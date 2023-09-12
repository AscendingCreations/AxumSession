use crate::{
    config::SecurityMode, DatabasePool, Session, SessionConfig, SessionData, SessionKey,
    SessionStore,
};
use axum_core::{
    body::{self, BoxBody},
    response::Response,
    BoxError,
};
use bytes::Bytes;
use chrono::Utc;
use cookie::{Cookie, CookieJar, Key};
#[cfg(feature = "key-store")]
use fastbloom_rs::Deletable;
use futures::future::BoxFuture;
use http::{
    self,
    header::{COOKIE, SET_COOKIE},
    HeaderMap, Request,
};
use http_body::Body as HttpBody;
use std::{
    boxed::Box,
    convert::Infallible,
    fmt::{self, Debug, Formatter},
    marker::{Send, Sync},
    task::{Context, Poll},
};
use tower_service::Service;

enum CookieType {
    Storable,
    Data,
    Key,
}

impl CookieType {
    #[inline]
    pub(crate) fn get_name(&self, config: &SessionConfig) -> String {
        match self {
            CookieType::Data => config.cookie_name.to_string(),
            CookieType::Storable => config.storable_cookie_name.to_string(),
            CookieType::Key => config.key_cookie_name.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct SessionService<S, T>
where
    T: DatabasePool + Clone + Debug + Sync + Send + 'static,
{
    pub(crate) session_store: SessionStore<T>,
    pub(crate) inner: S,
}

impl<S, T, ReqBody, ResBody> Service<Request<ReqBody>> for SessionService<S, T>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>, Error = Infallible>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
    Infallible: From<<S as Service<Request<ReqBody>>>::Error>,
    ResBody: HttpBody<Data = Bytes> + Send + 'static,
    ResBody::Error: Into<BoxError>,
    T: DatabasePool + Clone + Debug + Sync + Send + 'static,
{
    type Response = Response<BoxBody>;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let mut store = self.session_store.clone();
        let not_ready_inner = self.inner.clone();
        let mut ready_inner = std::mem::replace(&mut self.inner, not_ready_inner);

        Box::pin(async move {
            let cookies = get_cookies(&req);
            let mut session_key = match store.config.security_mode {
                SecurityMode::PerSession => SessionKey::get_or_create(&store, &cookies).await,
                SecurityMode::Simple => SessionKey::new(),
            };
            let (mut session, is_new) = Session::new(&mut store, &cookies, &session_key).await;
            let storable = cookies
                .get_cookie(&store.config.storable_cookie_name, &store.config.key)
                .map_or(false, |c| c.value().parse().unwrap_or(false));

            // Check if the session id exists if not lets check if it exists in the database or generate a new session.
            // If manual mode is enabled then do not check for a Session unless the UUID is not new.
            let check_database: bool = if is_new && !store.config.session_mode.is_manual() {
                let sess = SessionData::new(session.id.0, storable, &store.config);
                store.inner.insert(session.id.inner(), sess);
                false
            } else if !is_new || !store.config.session_mode.is_manual() {
                !store.service_session_data(&session)
            } else {
                false
            };

            if check_database {
                let mut sess = store
                    .load_session(session.id.inner())
                    .await
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| SessionData::new(session.id.0, storable, &store.config));

                sess.service_clear(store.config.memory_lifespan);
                sess.update();

                store.inner.insert(session.id.inner(), sess);
            }

            let (last_sweep, last_database_sweep) = {
                let timers = store.timers.read().await;
                (timers.last_expiry_sweep, timers.last_database_expiry_sweep)
            };

            // This branch runs less often, and we already have write access,
            // let's check if any sessions expired. We don't want to hog memory
            // forever by abandoned sessions (e.g. when a client lost their cookie)
            // throttle by memory lifespan - e.g. sweep every hour
            if last_sweep <= Utc::now() {
                // Only unload these from filter if the Client is None as this means no database.
                // Otherwise only unload from the filter if removed from the Database.
                #[cfg(feature = "key-store")]
                if !store.auto_handles_expiry() && store.config.use_bloom_filters {
                    store
                        .inner
                        .iter()
                        .filter(|r| r.autoremove < Utc::now())
                        .for_each(|r| session.store.filter.remove(r.key().as_bytes()));

                    store
                        .keys
                        .iter()
                        .filter(|r| r.autoremove < Utc::now())
                        .for_each(|r| session.store.filter.remove(r.key().as_bytes()));
                }

                store.inner.retain(|_k, v| v.autoremove > Utc::now());
                store.keys.retain(|_k, v| v.autoremove > Utc::now());
                store.timers.write().await.last_expiry_sweep =
                    Utc::now() + store.config.purge_update;
            }

            // Throttle by database lifespan - e.g. sweep every 6 hours
            if last_database_sweep <= Utc::now() && store.is_persistent() {
                //Remove any old keys that expired and Remove them from our loaded filter.
                #[cfg(feature = "key-store")]
                let expired = store.cleanup().await.unwrap();
                #[cfg(not(feature = "key-store"))]
                let _ = store.cleanup().await.unwrap();

                #[cfg(feature = "key-store")]
                if !store.auto_handles_expiry() {
                    expired
                        .iter()
                        .for_each(|id| session.store.filter.remove(id.as_bytes()));
                }

                store.timers.write().await.last_database_expiry_sweep =
                    Utc::now() + store.config.purge_database_update;
            }

            // Sets a clone of the Store in the Extensions for Direct usage and sets the Session for Direct usage
            req.extensions_mut().insert(store.clone());
            req.extensions_mut().insert(session.clone());

            let mut response = ready_inner.call(req).await?.map(body::boxed);

            let (renew, storable, renew_key, destroy, loaded) =
                if let Some(session_data) = session.store.inner.get(&session.id.inner()) {
                    (
                        session_data.renew,
                        session_data.storable,
                        session_data.renew_key,
                        session_data.destroy,
                        true,
                    )
                } else {
                    (false, false, false, false, false)
                };

            if !destroy && (!store.config.session_mode.is_manual() || loaded) {
                if renew {
                    // Lets change the Session ID and destory the old Session from the database.
                    let session_id = Session::generate_uuid(&store).await;

                    // Lets remove it from the database first.
                    if store.is_persistent() {
                        session
                            .store
                            .destroy_session(&session.id.inner())
                            .await
                            .unwrap();
                    }

                    //lets remove it from the filter. if the bottom fails just means it did not exist or was already unloaded.
                    #[cfg(feature = "key-store")]
                    if !store.auto_handles_expiry() && store.config.use_bloom_filters {
                        session.store.filter.remove(session.id.inner().as_bytes());
                    }

                    // Lets remove update and reinsert.
                    if let Some((_, mut session_data)) =
                        session.store.inner.remove(&session.id.inner())
                    {
                        session_data.id = session_id.0;
                        session_data.renew = false;
                        session.id = session_id;
                        store.inner.insert(session.id.inner(), session_data);
                    }
                }

                if renew_key && store.config.security_mode == SecurityMode::PerSession {
                    // Lets remove it from the database first.
                    if store.is_persistent() {
                        session
                            .store
                            .destroy_session(&session_key.id.inner())
                            .await
                            .unwrap();
                    }

                    // Lets remove update and reinsert.
                    #[cfg(feature = "key-store")]
                    let old_id = session_key.renew(&store).await.unwrap();

                    #[cfg(not(feature = "key-store"))]
                    let _ = session_key.renew(&store).await.unwrap();

                    #[cfg(feature = "key-store")]
                    if !store.auto_handles_expiry() && store.config.use_bloom_filters {
                        session.store.filter.remove(old_id.as_bytes());
                    }
                }
            } else if loaded {
                // lets unload everything if the session data has been loaded and set to be destroyed.
                #[cfg(feature = "key-store")]
                if !store.auto_handles_expiry() && store.config.use_bloom_filters {
                    session.store.filter.remove(session.id.inner().as_bytes());
                }

                if store.config.security_mode == SecurityMode::PerSession {
                    store.keys.remove(&session_key.id.inner());

                    if store.is_persistent() {
                        session
                            .store
                            .destroy_session(&session_key.id.inner())
                            .await
                            .unwrap();
                    }
                }

                let _ = session.store.inner.remove(&session.id.inner());

                if store.is_persistent() {
                    session
                        .store
                        .destroy_session(&session.id.inner())
                        .await
                        .unwrap();
                }
            }

            // Lets make a new jar as we only want to add our cookies to the Response cookie header.
            let mut cookies = CookieJar::new();

            // Add Per-Session encryption KeyID
            let cookie_key = match store.config.security_mode {
                SecurityMode::PerSession => {
                    if store.config.session_mode.is_storable() && storable && !destroy
                        || !store.config.session_mode.is_storable() && !destroy
                    {
                        cookies.add_cookie(
                            create_cookie(&store.config, session_key.id.inner(), CookieType::Key),
                            &store.config.key,
                        );
                    } else {
                        //If not Storable we still remove the encryption key since there is no session.
                        cookies.add_cookie(
                            remove_cookie(&store.config, CookieType::Key),
                            &store.config.key,
                        );
                    }

                    Some(session_key.key.clone())
                }
                SecurityMode::Simple => {
                    cookies.add_cookie(
                        remove_cookie(&store.config, CookieType::Key),
                        &store.config.key,
                    );
                    store.config.key.clone()
                }
            };

            // Add SessionID
            if (store.config.session_mode.is_storable() && storable
                || !store.config.session_mode.is_storable())
                && !destroy
            {
                cookies.add_cookie(
                    create_cookie(&store.config, session.id.inner(), CookieType::Data),
                    &cookie_key,
                );
            } else {
                cookies.add_cookie(remove_cookie(&store.config, CookieType::Data), &cookie_key);
            }

            // Add Session Storable Boolean
            if store.config.session_mode.is_storable() && storable && !destroy {
                cookies.add_cookie(
                    create_cookie(&store.config, storable.to_string(), CookieType::Storable),
                    &cookie_key,
                );
            } else {
                cookies.add_cookie(
                    remove_cookie(&store.config, CookieType::Storable),
                    &cookie_key,
                );
            }

            // Add the Session ID so it can link back to a Session if one exists.
            if (!store.config.session_mode.is_storable() || storable)
                && store.is_persistent()
                && !destroy
            {
                let sess = if let Some(mut sess) = session.store.inner.get_mut(&session.id.inner())
                {
                    if store.config.always_save || sess.update || !sess.validate() {
                        if sess.longterm {
                            sess.expires = Utc::now() + store.config.max_lifespan;
                        } else {
                            sess.expires = Utc::now() + store.config.lifespan;
                        };

                        sess.update = false;
                        Some(sess.clone())
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some(sess) = sess {
                    session.store.store_session(&sess).await.unwrap();

                    if store.config.security_mode == SecurityMode::PerSession {
                        session
                            .store
                            .store_key(&session_key, sess.expires.timestamp())
                            .await
                            .unwrap();
                    }
                }
            }

            if store.config.session_mode.is_storable() && !storable && !destroy {
                #[cfg(feature = "key-store")]
                if !store.auto_handles_expiry() && store.config.use_bloom_filters {
                    session.store.filter.remove(session.id.inner().as_bytes());
                }

                store.inner.remove(&session.id.inner());

                if store.config.security_mode == SecurityMode::PerSession {
                    store.keys.remove(&session_key.id.inner());
                }

                // Also run this just in case it was stored in the database and they rejected storability.
                if store.is_persistent() {
                    session
                        .store
                        .destroy_session(&session.id.inner())
                        .await
                        .unwrap();

                    if store.config.security_mode == SecurityMode::PerSession {
                        session
                            .store
                            .destroy_session(&session_key.id.inner())
                            .await
                            .unwrap();
                    }
                }
            }

            set_cookies(cookies, response.headers_mut());

            Ok(response)
        })
    }
}

impl<S, T> Debug for SessionService<S, T>
where
    S: Debug,
    T: DatabasePool + Clone + Debug + Sync + Send + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionService")
            .field("session_store", &self.session_store)
            .field("inner", &self.inner)
            .finish()
    }
}

pub(crate) trait CookiesExt {
    fn get_cookie(&self, name: &str, key: &Option<Key>) -> Option<Cookie<'static>>;
    fn add_cookie(&mut self, cookie: Cookie<'static>, key: &Option<Key>);
}

impl CookiesExt for CookieJar {
    fn get_cookie(&self, name: &str, key: &Option<Key>) -> Option<Cookie<'static>> {
        if let Some(key) = key {
            self.private(key).get(name)
        } else {
            self.get(name).cloned()
        }
    }

    fn add_cookie(&mut self, cookie: Cookie<'static>, key: &Option<Key>) {
        if let Some(key) = key {
            self.private_mut(key).add(cookie)
        } else {
            self.add(cookie)
        }
    }
}

fn create_cookie<'a>(config: &SessionConfig, value: String, cookie_type: CookieType) -> Cookie<'a> {
    let mut cookie_builder = Cookie::build(cookie_type.get_name(config), value)
        .path(config.cookie_path.clone())
        .secure(config.cookie_secure)
        .http_only(config.cookie_http_only)
        .same_site(config.cookie_same_site);

    if let Some(domain) = &config.cookie_domain {
        cookie_builder = cookie_builder.domain(domain.clone());
    }

    if let Some(max_age) = config.cookie_max_age {
        let time_duration = max_age.to_std().expect("Max Age out of bounds");
        cookie_builder =
            cookie_builder.expires(Some((std::time::SystemTime::now() + time_duration).into()));
    }

    cookie_builder.finish()
}

fn remove_cookie<'a>(config: &SessionConfig, cookie_type: CookieType) -> Cookie<'a> {
    let mut cookie_builder = Cookie::build(cookie_type.get_name(config), "")
        .path(config.cookie_path.clone())
        .http_only(config.cookie_http_only);

    if let Some(domain) = &config.cookie_domain {
        cookie_builder = cookie_builder.domain(domain.clone());
    }

    if let Some(domain) = &config.cookie_domain {
        cookie_builder = cookie_builder.domain(domain.clone());
    }

    let mut cookie = cookie_builder.finish();
    cookie.make_removal();
    cookie
}

fn get_cookies<ReqBody>(req: &Request<ReqBody>) -> CookieJar {
    let mut jar = CookieJar::new();

    let cookie_iter = req
        .headers()
        .get_all(COOKIE)
        .into_iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|cookie| Cookie::parse_encoded(cookie.to_owned()).ok());

    for cookie in cookie_iter {
        jar.add_original(cookie);
    }

    jar
}

fn set_cookies(jar: CookieJar, headers: &mut HeaderMap) {
    for cookie in jar.delta() {
        if let Ok(header_value) = cookie.encoded().to_string().parse() {
            headers.append(SET_COOKIE, header_value);
        }
    }
}
