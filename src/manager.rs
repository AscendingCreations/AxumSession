use crate::{AxumSession, AxumSessionData, AxumSessionID, AxumSessionStore};
use axum::{
    http::{Request, StatusCode},
    response::IntoResponse,
};
use axum_extra::middleware::Next;
use chrono::{Duration, Utc};
//use parking_lot::{Mutex, RwLockUpgradableReadGuard};
use std::collections::HashMap;
use tokio::sync::Mutex;
use tower_cookies::{Cookie, Cookies};
use uuid::Uuid;

///This manages the other services that can be seen in inner and gives access to the store.
/// the store is cloneable hence per each SqlxSession we clone it as we use thread Read write locks
/// to control any data that needs to be accessed across threads that cant be cloned.

pub async fn axum_session_runner<B>(
    mut req: Request<B>,
    next: Next<B>,
    store: AxumSessionStore,
) -> impl IntoResponse {
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
                    false,
                )
            } else {
                let store_ug = store.inner.read().await;
                let new_id = loop {
                    let token = Uuid::new_v4();

                    if !store_ug.contains_key(&token.to_string()) {
                        break token;
                    }
                };

                (AxumSessionID(new_id), true)
            };

            let no_store = {
                let store_ug = store.inner.read().await;

                if let Some(m) = store_ug.get(&id.0.to_string()) {
                    let mut inner = m.lock().await;

                    if inner.expires < Utc::now() || inner.destroy {
                        // Database Session expired, reuse the ID but drop data.
                        inner.data = HashMap::new();
                    }

                    // Session is extended by making a request with valid ID
                    inner.expires = Utc::now() + store.config.lifespan;
                    inner.autoremove = Utc::now() + store.config.memory_lifespan;
                    false
                } else {
                    true
                }
            };

            if !id.1 {
                if no_store {
                    let mut store_wg = store.inner.write().await;

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

                    let mut cookie =
                        Cookie::new(store.config.cookie_name.clone(), id.0 .0.to_string());

                    cookie.make_permanent();

                    cookies.add(cookie);
                    store_wg.insert(id.0 .0.to_string(), Mutex::new(sess));
                }
            } else {
                // --- New ID was generated Lets make a session for it ---
                // Get exclusive write access to the map
                let mut store_wg = store.inner.write().await;

                let (last_sweep, last_database_sweep) = {
                    let timers = store.timers.read().await;
                    (timers.last_expiry_sweep, timers.last_database_expiry_sweep)
                };
                // This branch runs less often, and we already have write access,
                // let's check if any sessions expired. We don't want to hog memory
                // forever by abandoned sessions (e.g. when a client lost their cookie)
                {
                    // Throttle by memory lifespan - e.g. sweep every hour
                    if last_sweep <= Utc::now() {
                        let mut timers = store.timers.write().await;
                        store_wg.retain(|_k, v| v.try_lock().unwrap().autoremove > Utc::now());
                        timers.last_expiry_sweep = Utc::now() + store.config.memory_lifespan;
                    }
                }

                {
                    // Throttle by database lifespan - e.g. sweep every 6 hours
                    if last_database_sweep <= Utc::now() {
                        let mut timers = store.timers.write().await;
                        store_wg.retain(|_k, v| v.try_lock().unwrap().autoremove > Utc::now());
                        store.cleanup().await.unwrap();
                        timers.last_database_expiry_sweep = Utc::now() + store.config.lifespan;
                    }
                }

                let mut cookie = Cookie::new(store.config.cookie_name.clone(), id.0 .0.to_string());
                cookie.make_permanent();
                cookies.add(cookie);

                let sess = AxumSessionData {
                    id: id.0 .0,
                    data: HashMap::new(),
                    expires: Utc::now() + Duration::hours(6),
                    destroy: false,
                    autoremove: Utc::now() + store.config.memory_lifespan,
                };

                store_wg.insert(id.0 .0.to_string(), Mutex::new(sess));
            }

            id.0
        },
        store: store.clone(),
    };

    //Sets a clone of the Store in the Extensions for Direct usage and sets the Session for Direct usage
    req.extensions_mut().insert(store.clone());
    req.extensions_mut().insert(session.clone());

    let session_data = {
        let sess = session.store.inner.read().await;
        let data = sess.get(&session.id.0.to_string());

        if let Some(session_data) = data {
            Some(session_data.lock().await.clone())
        } else {
            None
        }
    };

    if let Some(data) = session_data {
        session.store.store_session(data).await.unwrap();
    }

    Ok(next.run(req).await)
}
