use crate::{config::SecurityMode, headers::*, DatabasePool, Session, SessionData, SessionStore};
use axum_core::{
    body::{self, BoxBody},
    response::Response,
    BoxError,
};
use bytes::Bytes;
use chrono::Utc;
#[cfg(feature = "key-store")]
use fastbloom_rs::Deletable;
use futures::future::BoxFuture;
use http::{self, Request};
use http_body::Body as HttpBody;
use std::{
    boxed::Box,
    convert::Infallible,
    fmt::{self, Debug, Formatter},
    marker::{Send, Sync},
    task::{Context, Poll},
};
use tower_service::Service;

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
        let store = self.session_store.clone();
        let not_ready_inner = self.inner.clone();
        let mut ready_inner = std::mem::replace(&mut self.inner, not_ready_inner);

        Box::pin(async move {
            #[cfg(not(feature = "rest_mode"))]
            let cookies = get_cookies(req.headers());

            #[cfg(not(feature = "rest_mode"))]
            let (mut session_key, session_uuid, storable) =
                get_headers_and_key(&store, cookies).await;

            #[cfg(feature = "rest_mode")]
            let headers = get_headers(&store, req.headers());

            #[cfg(feature = "rest_mode")]
            let (mut session_key, session_uuid, storable) =
                get_headers_and_key(&store, headers).await;

            let (mut session, is_new) = Session::new(store, session_uuid).await;

            // Check if the session id exists if not lets check if it exists in the database or generate a new session.
            // If manual mode is enabled then do not check for a Session unless the UUID is not new.
            let check_database: bool = if is_new && !session.store.config.session_mode.is_manual() {
                let sess = SessionData::new(session.id.0, storable, &session.store.config);
                session.store.inner.insert(session.id.inner(), sess);
                false
            } else if !is_new || !session.store.config.session_mode.is_manual() {
                !session.store.service_session_data(&session)
            } else {
                false
            };

            if check_database {
                let mut sess = session
                    .store
                    .load_session(session.id.inner())
                    .await
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| {
                        SessionData::new(session.id.0, storable, &session.store.config)
                    });

                sess.autoremove = Utc::now() + session.store.config.memory_lifespan;
                sess.store = storable;
                sess.update = true;
                sess.requests = 1;
                session.store.inner.insert(session.id.inner(), sess);
            }

            let (last_sweep, last_database_sweep) = {
                let timers = session.store.timers.read().await;
                (timers.last_expiry_sweep, timers.last_database_expiry_sweep)
            };

            // This branch runs less often, and we already have write access,
            // let's check if any sessions expired. We don't want to hog memory
            // forever by abandoned sessions (e.g. when a client lost their cookie)
            // throttle by memory lifespan - e.g. sweep every hour
            let current_time = Utc::now();

            if last_sweep <= current_time && !session.store.config.memory_lifespan.is_zero() {
                // Only unload these from filter if the Client is None as this means no database.
                // Otherwise only unload from the filter if removed from the Database.
                #[cfg(feature = "key-store")]
                if session.store.is_persistent()
                    && session.store.auto_handles_expiry()
                    && session.store.config.use_bloom_filters
                {
                    let mut filter = session.store.filter.write().await;
                    session
                        .store
                        .inner
                        .iter()
                        .filter(|r| r.autoremove < current_time)
                        .for_each(|r| filter.remove(r.key().as_bytes()));

                    session
                        .store
                        .keys
                        .iter()
                        .filter(|r| r.autoremove < current_time)
                        .for_each(|r| filter.remove(r.key().as_bytes()));
                }

                session
                    .store
                    .inner
                    .retain(|_k, v| v.autoremove > current_time);
                session
                    .store
                    .keys
                    .retain(|_k, v| v.autoremove > current_time);
                session.store.timers.write().await.last_expiry_sweep =
                    Utc::now() + session.store.config.purge_update;
            }

            // Throttle by database lifespan - e.g. sweep every 6 hours
            if last_database_sweep <= current_time && session.store.is_persistent() {
                //Remove any old keys that expired and Remove them from our loaded filter.
                #[cfg(feature = "key-store")]
                let expired = session.store.cleanup().await.unwrap();
                #[cfg(not(feature = "key-store"))]
                let _ = session.store.cleanup().await.unwrap();

                #[cfg(feature = "key-store")]
                if !session.store.auto_handles_expiry() {
                    let mut filter = session.store.filter.write().await;
                    expired.iter().for_each(|id| filter.remove(id.as_bytes()));
                }

                session
                    .store
                    .timers
                    .write()
                    .await
                    .last_database_expiry_sweep =
                    Utc::now() + session.store.config.purge_database_update;
            }

            // Sets a clone of the Store in the Extensions for Direct usage and sets the Session for Direct usage
            //req.extensions_mut().insert(store.clone());
            req.extensions_mut().insert(session.clone());

            let mut response = ready_inner.call(req).await?.map(body::boxed);

            let (renew, storable, renew_key, destroy, loaded) =
                if let Some(session_data) = session.store.inner.get(&session.id.inner()) {
                    (
                        session_data.renew,
                        session_data.store,
                        session_data.renew_key,
                        session_data.destroy,
                        true,
                    )
                } else {
                    (false, false, false, false, false)
                };

            if !destroy && (!session.store.config.session_mode.is_manual() || loaded) {
                if renew {
                    // Lets change the Session ID and destory the old Session from the database.
                    let session_id = Session::generate_uuid(&session.store).await;

                    // Lets remove it from the database first.
                    if session.store.is_persistent() {
                        session
                            .store
                            .database_remove_session(session.id.inner())
                            .await
                            .unwrap();
                    }

                    //lets remove it from the filter. if the bottom fails just means it did not exist or was already unloaded.
                    #[cfg(feature = "key-store")]
                    if session.store.config.use_bloom_filters {
                        let mut filter = session.store.filter.write().await;
                        filter.remove(session.id.inner().as_bytes());
                    }

                    // Lets remove update and reinsert.
                    if let Some((_, mut session_data)) =
                        session.store.inner.remove(&session.id.inner())
                    {
                        session_data.id = session_id.0;
                        session_data.renew = false;
                        session.id = session_id;
                        session.store.inner.insert(session.id.inner(), session_data);
                    }
                }

                if renew_key && session.store.config.security_mode == SecurityMode::PerSession {
                    // Lets remove it from the database first.
                    if session.store.is_persistent() {
                        session
                            .store
                            .database_remove_session(session_key.id.inner())
                            .await
                            .unwrap();
                    }

                    // Lets remove update and reinsert.
                    #[cfg(feature = "key-store")]
                    let old_id = session_key.renew(&session.store).await.unwrap();

                    #[cfg(not(feature = "key-store"))]
                    let _ = session_key.renew(&session.store).await.unwrap();

                    #[cfg(feature = "key-store")]
                    if session.store.config.use_bloom_filters {
                        let mut filter = session.store.filter.write().await;
                        filter.remove(old_id.as_bytes());
                    }
                }
            }

            // Add the Session ID so it can link back to a Session if one exists.
            if (!session.store.config.session_mode.is_opt_in() || storable)
                && session.store.is_persistent()
                && !destroy
            {
                let clone_session =
                    if let Some(mut sess) = session.store.inner.get_mut(&session.id.inner()) {
                        // Check if Database needs to be updated or not. TODO: Make updatable based on a timer for in memory only.
                        if session.store.config.always_save || sess.update || !sess.validate() {
                            if sess.longterm {
                                sess.expires = Utc::now() + session.store.config.max_lifespan;
                            } else {
                                sess.expires = Utc::now() + session.store.config.lifespan;
                            };

                            sess.update = false;

                            Some(sess.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                if let Some(sess) = clone_session {
                    session.store.store_session(&sess).await.unwrap();

                    if session.store.config.security_mode == SecurityMode::PerSession {
                        session
                            .store
                            .store_key(&session_key, sess.expires.timestamp())
                            .await
                            .unwrap();
                    }
                }
            }

            //lets tell the system we can unload this request now.
            //If there are still more left the bottom wont unload anything.
            session.remove_request();

            if ((session.store.config.session_mode.is_opt_in() && !storable) || destroy)
                && !session.is_parallel()
            {
                if session.store.config.security_mode == SecurityMode::PerSession {
                    #[cfg(feature = "key-store")]
                    if session.store.config.use_bloom_filters {
                        let mut filter = session.store.filter.write().await;
                        filter.remove(session_key.id.inner().as_bytes());
                    }

                    let _ = session.store.keys.remove(&session_key.id.inner());

                    if session.store.is_persistent() {
                        session
                            .store
                            .database_remove_session(session_key.id.inner())
                            .await
                            .unwrap();
                    }
                }

                #[cfg(feature = "key-store")]
                if session.store.config.use_bloom_filters {
                    let mut filter = session.store.filter.write().await;
                    filter.remove(session.id.inner().as_bytes());
                }

                let _ = session.store.inner.remove(&session.id.inner());

                if session.store.is_persistent() {
                    session
                        .store
                        .database_remove_session(session.id.inner())
                        .await
                        .unwrap();
                }
            }

            // We will Deleted the data in memory as it should be stored in the database instead.
            // if user is using this without a database then it will only work as a per request data store.
            if session.store.config.memory_lifespan.is_zero() && !session.is_parallel() {
                #[cfg(feature = "key-store")]
                if !session.store.is_persistent() && session.store.config.use_bloom_filters {
                    let mut filter = session.store.filter.write().await;
                    filter.remove(session.id.inner().as_bytes());
                    filter.remove(session_key.id.inner().as_bytes());
                }

                session.store.inner.remove(&session.id.inner());
                session.store.keys.remove(&session_key.id.inner());
            }

            set_headers(
                &session,
                &session_key,
                response.headers_mut(),
                destroy,
                storable,
            );

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
