use crate::{AxumSession, AxumSessionConfig, AxumSessionData, AxumSessionID, AxumSessionStore};
use axum::{
    http::{Request, StatusCode},
    response::IntoResponse,
};
use axum_extra::middleware::Next;
use chrono::{Duration, Utc};
use std::collections::HashMap;
use tokio::sync::Mutex;
use tower_cookies::{Cookie, Cookies};
use uuid::Uuid;

///axum_session_runner Creates, Manages and Sets an AxumSession into the Request extensions.
///This will unload and load other Session data based on Access and a timer check.
///returns an Response when all the Futures After run.
pub async fn axum_session_runner<B>(
    mut req: Request<B>,
    next: Next<B>,
    store: AxumSessionStore,
) -> impl IntoResponse {
    let config = store.config.clone();
    // We Extract the Tower_Cookies Extensions Variable so we can add Cookies to it. Some reason can only be done here..?
    let cookies = match req.extensions().get::<Cookies>() {
        Some(cookies) => cookies,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let session = AxumSession {
        id: {
            let id = if let Some(cookie) = cookies.get(&store.config.cookie_name) {
                (
                    AxumSessionID(Uuid::parse_str(cookie.value()).expect("`Could not parse Uuid")),
                    true,
                )
            } else {
                let store_ug = store.inner.read().await;
                let new_id = loop {
                    let token = Uuid::new_v4();

                    if !store_ug.contains_key(&token.to_string()) {
                        break token;
                    }
                };

                (AxumSessionID(new_id), false)
            };

            // If a cookie did have an AxumSessionID then lets check if it still exists in the hash or Database
            // If not make a new Session using the ID.
            if id.1 {
                if let Some(m) = store.inner.read().await.get(&id.0.to_string()) {
                    let mut inner = m.lock().await;

                    if inner.expires < Utc::now() || inner.destroy {
                        // Database Session expired, reuse the ID but drop data.
                        inner.data = HashMap::new();
                    }

                    // Session is extended by making a request with valid ID
                    inner.expires = Utc::now() + store.config.lifespan;
                    inner.autoremove = Utc::now() + store.config.memory_lifespan;
                } else {
                    let mut sess = store
                        .load_session(id.0.to_string())
                        .await
                        .ok()
                        .flatten()
                        .unwrap_or(AxumSessionData {
                            id: id.0 .0,
                            data: HashMap::new(),
                            expires: Utc::now() + Duration::hours(6),
                            destroy: false,
                            autoremove: Utc::now() + store.config.memory_lifespan,
                        });

                    if !sess.validate() || sess.destroy {
                        sess.data = HashMap::new();
                        sess.expires = Utc::now() + Duration::hours(6);
                        sess.autoremove = Utc::now() + store.config.memory_lifespan;
                    }

                    cookies.add(create_cookie(config, id.0 .0.to_string()));
                    store
                        .inner
                        .write()
                        .await
                        .insert(id.0 .0.to_string(), Mutex::new(sess));
                }
            } else {
                // --- New ID was generated Lets make a session for it ---
                cookies.add(create_cookie(config, id.0 .0.to_string()));

                let sess = AxumSessionData {
                    id: id.0 .0,
                    data: HashMap::new(),
                    expires: Utc::now() + Duration::hours(6),
                    destroy: false,
                    autoremove: Utc::now() + store.config.memory_lifespan,
                };

                store
                    .inner
                    .write()
                    .await
                    .insert(id.0 .0.to_string(), Mutex::new(sess));
            }

            id.0
        },
        store: store.clone(),
    };

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
        store.timers.write().await.last_expiry_sweep = Utc::now() + store.config.memory_lifespan;
    }

    // Throttle by database lifespan - e.g. sweep every 6 hours
    if last_database_sweep <= Utc::now() {
        store.cleanup().await.unwrap();
        store.timers.write().await.last_database_expiry_sweep = Utc::now() + store.config.lifespan;
    }

    //Sets a clone of the Store in the Extensions for Direct usage and sets the Session for Direct usage
    req.extensions_mut().insert(store.clone());
    req.extensions_mut().insert(session.clone());

    if let Some(session_data) = session
        .store
        .inner
        .read()
        .await
        .get(&session.id.0.to_string())
    {
        session
            .store
            .store_session(session_data.lock().await.clone())
            .await
            .unwrap()
    }

    Ok(next.run(req).await)
}

fn create_cookie<'a>(config: AxumSessionConfig, value: String) -> Cookie<'a> {
    let mut cookie_builder = Cookie::build(config.cookie_name, value)
        .path(config.cookie_path)
        .secure(config.cookie_secure)
        .http_only(config.cookie_http_only);

    if let Some(domain) = config.cookie_domain {
        cookie_builder = cookie_builder
            .domain(domain)
            .same_site(config.cookie_same_site);
    }

    if let Some(max_age) = config.cookie_max_age {
        let time_duration = max_age.to_std().expect("Max Age out of bounds");
        cookie_builder =
            cookie_builder.max_age(time_duration.try_into().expect("Max Age out of bounds"));
    }

    cookie_builder.finish()
}
