use crate::{AxumSession, AxumSessionConfig, AxumSessionData, AxumSessionStore};
use axum_core::{
    body::{self, BoxBody},
    response::Response,
    BoxError,
};
use bytes::Bytes;
use chrono::Utc;
use cookie::{Cookie, CookieJar, Key};
use futures::future::BoxFuture;
use http::{
    self,
    header::{COOKIE, SET_COOKIE},
    HeaderMap, Request,
};
use http_body::Body as HttpBody;
use std::collections::HashMap;
use std::{
    boxed::Box,
    convert::Infallible,
    fmt,
    task::{Context, Poll},
};
use tokio::sync::Mutex;
use tower_service::Service;

enum CookieType {
    Storable,
    Data,
}

impl CookieType {
    pub(crate) fn get_name(&self, config: &AxumSessionConfig) -> String {
        match self {
            CookieType::Data => config.cookie_name.to_string(),
            CookieType::Storable => config.storable_cookie_name.to_string(),
        }
    }

    pub(crate) fn get_age(&self, config: &AxumSessionConfig) -> Option<chrono::Duration> {
        match self {
            CookieType::Data => config.cookie_max_age,
            CookieType::Storable => config.storable_cookie_max_age,
        }
    }
}

#[derive(Clone)]
pub struct AxumSessionService<S> {
    pub(crate) session_store: AxumSessionStore,
    pub(crate) inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for AxumSessionService<S>
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
{
    type Response = Response<BoxBody>;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let store = self.session_store.clone();
        let not_ready_inner = self.inner.clone();
        let mut ready_inner = std::mem::replace(&mut self.inner, not_ready_inner);

        Box::pin(async move {
            let mut cookies = get_cookies(&req);
            let session = AxumSession::new(&store, &cookies).await;
            let accepted = cookies
                .get_cookie(&store.config.storable_cookie_name, &store.config.key)
                .map_or(false, |c| c.value().parse().unwrap_or(false));

            // check if the session id exists if not lets check if it exists in the database or generate a new session.
            if !store.service_session_data(&session).await {
                let mut sess = store
                    .load_session(session.id.inner())
                    .await
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| AxumSessionData::new(session.id.0, accepted, &store.config));

                if !sess.validate() || sess.destroy {
                    sess.destroy = false;
                    sess.data = HashMap::new();
                    sess.autoremove = Utc::now() + store.config.memory_lifespan;
                }

                store
                    .inner
                    .write()
                    .await
                    .insert(session.id.inner(), Mutex::new(sess));
            }

            let (last_sweep, last_database_sweep) = {
                let timers = store.timers.read().await;
                (timers.last_expiry_sweep, timers.last_database_expiry_sweep)
            };

            // This branch runs less often, and we already have write access,
            // let's check if any sessions expired. We don't want to hog memory
            // forever by abandoned sessions (e.g. when a client lost their cookie)
            // Throttle by memory lifespan - e.g. sweep every hour
            if last_sweep <= Utc::now() {
                store.inner.write().await.retain(|_k, v| {
                    v.try_lock()
                        .map(|data| data.autoremove > Utc::now())
                        .unwrap_or(true)
                });
                store.timers.write().await.last_expiry_sweep =
                    Utc::now() + store.config.memory_lifespan;
            }

            // Throttle by database lifespan - e.g. sweep every 6 hours
            if last_database_sweep <= Utc::now() && store.is_persistent() {
                store.cleanup().await.unwrap();
                store.timers.write().await.last_database_expiry_sweep =
                    Utc::now() + store.config.lifespan;
            }

            //Sets a clone of the Store in the Extensions for Direct usage and sets the Session for Direct usage
            req.extensions_mut().insert(store.clone());
            req.extensions_mut().insert(session.clone());

            let mut response = ready_inner.call(req).await?.map(body::boxed);

            let storable = if let Some(session_data) =
                session.store.inner.read().await.get(&session.id.inner())
            {
                session_data.lock().await.storable
            } else {
                false
            };

            // Add the Storable Cookie so we can keep track if they can store the session.
            // Todo: Maybe add a way to store expiration times and such for accepted or not accept via json.
            cookies.add_cookie(
                create_cookie(&store.config, storable.to_string(), CookieType::Storable),
                &store.config.key,
            );

            // Add the Session ID so it can link back to a Session if one exists.
            cookies.add_cookie(
                create_cookie(&store.config, session.id.inner(), CookieType::Data),
                &store.config.key,
            );

            if !store.config.session_mode.is_storable() || accepted {
                // run this After a response has returned so we save the most updated data to sql.
                if store.is_persistent() {
                    if let Some(session_data) =
                        session.store.inner.read().await.get(&session.id.inner())
                    {
                        let mut sess = session_data.lock().await;

                        if sess.longterm {
                            sess.expires = Utc::now() + store.config.max_lifespan;
                        } else {
                            sess.expires = Utc::now() + store.config.lifespan;
                        }

                        session.store.store_session(&sess).await.unwrap()
                    }
                }
            }

            if store.config.session_mode.is_storable() && !accepted {
                store.inner.write().await.remove(&session.id.inner());

                // Also run this just in case it was stored in the database and they rejected storability.
                if store.is_persistent() {
                    session
                        .store
                        .destroy_session(&session.id.inner())
                        .await
                        .unwrap();
                }
            }

            set_cookies(cookies, response.headers_mut());

            Ok(response)
        })
    }
}

impl<S> fmt::Debug for AxumSessionService<S>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AxumSessionService")
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

fn create_cookie<'a>(
    config: &AxumSessionConfig,
    value: String,
    cookie_type: CookieType,
) -> Cookie<'a> {
    let mut cookie_builder = Cookie::build(cookie_type.get_name(config), value)
        .path(config.cookie_path.clone())
        .secure(config.cookie_secure)
        .http_only(config.cookie_http_only);

    if let Some(domain) = &config.cookie_domain {
        cookie_builder = cookie_builder
            .domain(domain.clone())
            .same_site(config.cookie_same_site);
    }

    if let Some(max_age) = cookie_type.get_age(config) {
        let time_duration = max_age.to_std().expect("Max Age out of bounds");
        cookie_builder =
            cookie_builder.max_age(time_duration.try_into().expect("Max Age out of bounds"));
    }

    cookie_builder.finish()
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
