use crate::{AxumSession, AxumSessionConfig, AxumSessionData, AxumSessionID, AxumSessionStore};
use axum_core::{
    body::{self, BoxBody},
    response::Response,
    BoxError,
};
use bytes::Bytes;
use chrono::Utc;
use cookie::{Cookie, CookieJar};
use futures::future::BoxFuture;
use http::{
    self,
    header::{COOKIE, SET_COOKIE},
    HeaderMap, Request, StatusCode,
};
use http_body::{Body as HttpBody, Full};
use std::collections::HashMap;
use std::{
    boxed::Box,
    convert::Infallible,
    fmt,
    task::{Context, Poll},
};
use tokio::sync::Mutex;
use tower_service::Service;
use uuid::Uuid;

enum CookieType {
    Accepted,
    Data,
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
            let config = store.config.clone();
            let mut cookies = get_cookies(&req);
            let mut is_new_id = false;

            let session = AxumSession {
                id: {
                    if let Some(cookie) = cookies.get(&store.config.cookie_name) {
                        AxumSessionID(match Uuid::parse_str(cookie.value()) {
                            Ok(v) => v,
                            Err(_) => {
                                tracing::warn!("Uuid \" {} \" is invalid", cookie.value());
                                return Ok(Response::builder()
                                    .status(StatusCode::UNAUTHORIZED)
                                    .body(body::boxed(Full::from("401 Unauthorized")))
                                    .unwrap());
                            }
                        })
                    } else {
                        let store_ug = store.inner.read().await;
                        let new_id = loop {
                            let token = Uuid::new_v4();

                            if !store_ug.contains_key(&token.to_string()) {
                                break token;
                            }
                        };

                        is_new_id = true;
                        AxumSessionID(new_id)
                    }
                },
                store: store.clone(),
            };

            let accepted = if let Some(cookie) = cookies.get(&store.config.accepted_cookie_name) {
                cookie.value().parse().unwrap_or(false)
            } else {
                false
            };

            // If a cookie did have an AxumSessionID then lets check if it still exists in the hash or Database
            // If not make a new Session using the ID.
            if !is_new_id {
                let mut no_store = true;

                {
                    if let Some(m) = store.inner.read().await.get(&session.id.inner()) {
                        let mut inner = m.lock().await;

                        if inner.expires < Utc::now() || inner.destroy {
                            inner.longterm = false;
                            inner.data = HashMap::new();
                        }

                        inner.autoremove = Utc::now() + store.config.memory_lifespan;
                        no_store = false;
                    }
                }
                if no_store {
                    let mut sess = store
                        .load_session(session.id.inner())
                        .await
                        .ok()
                        .flatten()
                        .unwrap_or_else(|| {
                            AxumSessionData::new(session.id.0, accepted, &store.config)
                        });

                    if !sess.validate() || sess.destroy {
                        sess.data = HashMap::new();
                        sess.autoremove = Utc::now() + store.config.memory_lifespan;
                    }

                    store
                        .inner
                        .write()
                        .await
                        .insert(session.id.inner(), Mutex::new(sess));
                }
            } else {
                store.inner.write().await.insert(
                    session.id.inner(),
                    Mutex::new(AxumSessionData::new(session.id.0, accepted, &store.config)),
                );
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
                    if let Ok(data) = v.try_lock() {
                        data.autoremove > Utc::now()
                    } else {
                        //the lock is busy so rather than just killing
                        //everything lets ignore it till next time.
                        true
                    }
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

            let (remove, accepted) = if let Some(session_data) =
                session.store.inner.read().await.get(&session.id.inner())
            {
                let sess = session_data.lock().await;

                (
                    if (!store.config.disable_gdpr && sess.accepted) || store.config.disable_gdpr {
                        cookies.add(create_cookie(
                            config.clone(),
                            sess.id.to_string(),
                            CookieType::Data,
                        ));
                        false
                    } else {
                        true
                    },
                    sess.accepted,
                )
            } else {
                (false, false)
            };

            //Add the Accepted Cookie so we cna keep track if they accepted it or not.
            cookies.add(create_cookie(
                config,
                accepted.to_string(),
                CookieType::Accepted,
            ));

            //run this After a response has returned so we save the most updated data to sql.
            if store.is_persistent() && !remove {
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

            if remove {
                store.inner.write().await.remove(&session.id.inner());
                //Also run this just in case it was stored int he database and they rejected the cookies.
                session
                    .store
                    .destroy_session(&session.id.inner())
                    .await
                    .unwrap();
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

fn create_cookie<'a>(
    config: AxumSessionConfig,
    value: String,
    cookie_type: CookieType,
) -> Cookie<'a> {
    let mut cookie_builder = Cookie::build(
        match cookie_type {
            CookieType::Data => config.cookie_name,
            CookieType::Accepted => config.accepted_cookie_name,
        },
        value,
    )
    .path(config.cookie_path)
    .secure(config.cookie_secure)
    .http_only(config.cookie_http_only);

    if let Some(domain) = config.cookie_domain {
        cookie_builder = cookie_builder
            .domain(domain)
            .same_site(config.cookie_same_site);
    }

    let cookie_max_age = match cookie_type {
        CookieType::Data => config.cookie_max_age,
        CookieType::Accepted => config.accepted_cookie_max_age,
    };

    if let Some(max_age) = cookie_max_age {
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
