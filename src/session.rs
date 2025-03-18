use crate::{DatabasePool, SessionData, SessionError, SessionStore};
use axum::extract::FromRequestParts;

#[cfg(feature = "key-store")]
use fastbloom_rs::Membership;
use http::{request::Parts, StatusCode};
use serde::Serialize;
use std::fmt::Debug;

/// A Session Store.
///
/// Provides a Storage Handler to SessionStore and contains the ID of the current session.
///
/// This is Auto generated by the Session Layer Upon Service Execution.
#[derive(Debug, Clone)]
pub struct Session<T>
where
    T: DatabasePool + Clone + Debug + Sync + Send + 'static,
{
    /// The SessionStore that holds all the Sessions.
    pub(crate) store: SessionStore<T>,
    /// The Sessions current ID for looking up its store.
    pub(crate) id: String,
}

/// Adds `FromRequestParts<B>` for Session
///
/// Returns the Session from Axum's request extensions state
impl<T, S> FromRequestParts<S> for Session<T>
where
    T: DatabasePool + Clone + Debug + Sync + Send + 'static,
    S: Send + Sync,
{
    type Rejection = (http::StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts.extensions.get::<Session<T>>().cloned().ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Can't extract Axum `Session`. Is `SessionLayer` enabled?",
        ))
    }
}

impl<S> Session<S>
where
    S: DatabasePool + Clone + Debug + Sync + Send + 'static,
{
    #[allow(clippy::needless_pass_by_ref_mut)]
    pub(crate) async fn new(
        store: SessionStore<S>,
        value: Option<String>,
    ) -> Result<(Self, bool), SessionError> {
        let (id, is_new) = match value {
            Some(v) => (v, false),
            None => (Self::generate_id(&store).await?, true),
        };

        #[cfg(feature = "key-store")]
        if store.config.memory.use_bloom_filters {
            let contained = {
                let filter = store.filter.read().await;
                filter.contains(id.as_bytes())
            };

            if !contained {
                let mut filter = store.filter.write().await;
                filter.add(id.as_bytes());
            }
        }

        Ok((Self { id, store }, is_new))
    }

    #[cfg(feature = "key-store")]
    pub(crate) async fn generate_id(store: &SessionStore<S>) -> Result<String, SessionError> {
        loop {
            let token = store.config.id_generator.generate();

            if (!store.config.memory.use_bloom_filters || store.auto_handles_expiry())
                && !store.inner.contains_key(&token)
            {
                //This fixes an already used but in database issue.
                if let Some(client) = &store.client {
                    // Unwrap should be safe to use as we would want it to crash if there was a major database error.
                    // This would mean the database no longer is online or the table missing etc.
                    if !client
                        .exists(&token.to_string(), &store.config.database.table_name)
                        .await?
                    {
                        return Ok(token);
                    }
                } else {
                    return Ok(token);
                }
            } else {
                let filter = store.filter.read().await;

                if !filter.contains(token.to_string().as_bytes()) {
                    return Ok(token);
                }
            }
        }
    }

    #[cfg(not(feature = "key-store"))]
    pub(crate) async fn generate_id(store: &SessionStore<S>) -> Result<String, SessionError> {
        loop {
            let token = store.config.id_generator.generate();

            if !store.inner.contains_key(&token) {
                //This fixes an already used but in database issue.
                if let Some(client) = &store.client {
                    // Unwrap should be safe to use as we would want it to crash if there was a major database error.
                    // This would mean the database no longer is online or the table missing etc.
                    if !client
                        .exists(&token.to_string(), &store.config.database.table_name)
                        .await?
                    {
                        return Ok(token);
                    }
                } else {
                    return Ok(token);
                }
            }
        }
    }
    /// Sets the Session to create the SessionData based on the current Session ID.
    /// You can only use this if SessionMode::Manual is set or it will Panic.
    /// This will also set the store to true similar to session.set_store(true);
    ///
    /// # Examples
    /// ```rust ignore
    /// session.create_data();
    /// ```
    ///
    #[inline]
    pub fn create_data(&self) {
        if !self.store.config.session_mode.is_manual() {
            panic!(
                "Session must be set to SessionMode::Manual in order to use create_data,
                as the Session data is created already."
            );
        }
        let session_data = SessionData::new(self.id.clone(), true, &self.store.config);
        self.store.inner.insert(self.id.clone(), session_data);
    }

    /// Checks if the SessionData was created or not.
    ///
    /// # Examples
    /// ```rust ignore
    /// if session.data_exists() {
    ///     println!("data Exists");
    /// }
    /// ```
    ///
    #[inline]
    pub fn data_exists(&self) -> bool {
        self.store.inner.contains_key(&self.id)
    }

    /// Set session flags to renew/regenerate the ID.
    /// This deletes data from the database keyed with the old ID.
    /// This helps to enhance security when logging into secure
    /// areas on a website. The current session's data will be
    /// stored with the new ID.
    ///
    /// # Examples
    /// ```rust ignore
    /// session.renew();
    /// ```
    ///
    #[inline]
    pub fn renew(&self) {
        self.store.renew(self.id.clone());
    }

    /// Sets the Session to force update the database.
    /// This will increase the Timer on the sessions store
    /// making the session live longer in the persistent database.
    ///
    /// # Examples
    /// ```rust ignore
    /// session.renew();
    /// ```
    ///
    #[inline]
    pub fn update(&self) {
        self.store.update(self.id.clone());
    }

    /// Sets the Current Session to be Destroyed.
    /// This will Deleted the Session and Cookies upon Response Phase.
    ///
    /// # Examples
    /// ```rust ignore
    /// session.destroy();
    /// ```
    ///
    #[inline]
    pub fn destroy(&self) {
        self.store.destroy(self.id.clone());
    }

    /// Sets the Current Session to a long term expiration. Useful for Remember Me setups.
    /// This will also update the database on Response Phase.
    ///
    /// # Examples
    /// ```rust ignore
    /// session.set_longterm(true);
    /// ```
    ///
    #[inline]
    pub fn set_longterm(&self, longterm: bool) {
        self.store.set_longterm(self.id.clone(), longterm);
    }

    /// Allows the Current Session to store.
    /// This will also update the database on Response Phase.
    ///
    /// This is only used when `SessionMode` is Manual or OptIn.
    /// This will allow the Session to be stored if true.
    /// This will delete and not allow a session to be stored if false.
    ///
    /// # Examples
    /// ```rust ignore
    /// session.set_store(true);
    /// ```
    ///
    #[inline]
    pub fn set_store(&self, can_store: bool) {
        self.store.set_store(self.id.clone(), can_store);
    }

    /// Gets data from the Session's HashMap
    ///
    /// Provides an `Option<T>` that returns the requested data from the Sessions store.
    /// Returns None if Key does not exist or if serde_json failed to deserialize.
    ///
    /// # Examples
    /// ```rust ignore
    /// let id = session.get("user-id").unwrap_or(0);
    /// ```
    ///
    ///Used to get data stored within SessionData's hashmap from a key value.
    ///
    #[inline]
    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.store.get(self.id.clone(), key)
    }

    /// Removes a Key from the Current Session's HashMap returning it.
    ///
    /// Provides an `Option<T> `that returns the requested data from the Sessions store.
    /// Returns None if Key does not exist or if serde_json failed to deserialize.
    ///
    /// # Examples
    /// ```rust ignore
    /// let id = session.get_remove("user-id").unwrap_or(0);
    /// ```
    ///
    /// Used to get data stored within SessionData's hashmap from a key value.
    ///
    #[inline]
    pub fn get_remove<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.store.get_remove(self.id.clone(), key)
    }

    /// Sets data to the Current Session's HashMap.
    /// This will also update the database on Response Phase.
    ///
    /// # Examples
    /// ```rust ignore
    /// session.set("user-id", 1);
    /// ```
    ///
    #[inline]
    pub fn set(&self, key: &str, value: impl Serialize) {
        self.store.set(self.id.clone(), key, value);
    }

    /// Removes a Key from the Current Session's HashMap.
    /// Does not process the String into a Type, Just removes it.
    /// This will also update the database on Response Phase.
    ///
    /// # Examples
    /// ```rust ignore
    /// let _ = session.remove("user-id");
    /// ```
    ///
    #[inline]
    pub fn remove(&self, key: &str) {
        self.store.remove(self.id.clone(), key);
    }

    /// Clears all data from the Current Session's HashMap instantly.
    /// This will also update the database on Response Phase.
    ///
    /// # Examples
    /// ```rust ignore
    /// session.clear();
    /// ```
    ///
    #[inline]
    pub fn clear(&self) {
        self.store.clear_session_data(self.id.clone());
    }

    /// Returns a i64 count of how many Sessions exist.
    ///
    /// If the Session is persistent it will return all sessions within the database.
    /// If the Session is not persistent it will return a count within SessionStore.
    ///
    /// # Examples
    /// ```rust ignore
    /// let count = session.count().await;
    /// ```
    ///
    #[inline]
    pub async fn count(&self) -> i64 {
        self.store.count_sessions().await
    }

    /// Returns the SessionID for this Session.
    ///
    /// # Examples
    /// ```rust ignore
    /// let session_id = session.get_session_id();
    /// ```
    ///
    #[inline]
    pub fn get_session_id(&self) -> String {
        self.id.clone()
    }

    /// Returns the store for this Session.
    ///
    /// The store contains everything that all sessions need.
    ///
    /// # Examples
    /// ```rust ignore
    /// let store = session.get_store();
    /// ```
    ///
    #[inline]
    pub fn get_store(&self) -> &SessionStore<S> {
        &self.store
    }

    /// Returns a mutable store for this Session.
    ///
    /// The store contains everything that all sessions need.
    ///
    /// # Examples
    /// ```rust ignore
    /// let store = session.get_store_mut();
    /// ```
    ///
    #[inline]
    pub fn get_mut_store(&mut self) -> &mut SessionStore<S> {
        &mut self.store
    }

    /// Removes a Request from the request counter
    /// used to determine if parallel requests exist.
    /// prevents data deletion until requests == 0.
    ///
    /// # Examples
    /// ```rust ignore
    /// session.remove_request();
    /// ```
    ///
    #[inline]
    pub(crate) fn remove_request(&self) {
        self.store.remove_session_request(self.id.clone());
    }

    /// Removes a Request from the request counter
    /// used to determine if parallel requests exist.
    /// prevents data deletion until requests == 0.
    ///
    /// # Examples
    /// ```rust ignore
    /// session.set_request();
    /// ```
    ///
    #[inline]
    pub(crate) fn set_request(&self) {
        self.store.set_session_request(self.id.clone());
    }

    /// checks if a session has more than one request.
    ///
    /// # Examples
    /// ```rust ignore
    /// session.is_parallel();
    /// ```
    ///
    #[inline]
    pub(crate) fn is_parallel(&self) -> bool {
        self.store.is_session_parallel(self.id.clone())
    }

    /// checks if a session exists and if it is outdated.
    ///
    /// # Examples
    /// ```rust ignore
    /// session.verify();
    /// ```
    ///
    #[cfg(feature = "advanced")]
    #[cfg_attr(docsrs, doc(cfg(feature = "advanced")))]
    #[inline]
    pub fn verify(&self) -> Result<(), SessionError> {
        self.store.verify(self.id)
    }

    /// Updates the sessions stored database expire time.
    /// Use this before forcing a update to the database store.
    /// will update the database expires based on
    /// if the session is longterm then configs max_lifespan.
    /// if not then configs lifespan.
    ///
    /// THIS WILL NOT UPDATE THE DATABASE SIDE.
    ///
    /// # Examples
    /// ```rust ignore
    /// session.update_database_expires();
    /// ```
    ///
    #[cfg(feature = "advanced")]
    #[cfg_attr(docsrs, doc(cfg(feature = "advanced")))]
    #[inline]
    pub fn update_database_expires(&self) -> Result<(), SessionError> {
        self.store.update_database_expires(self.id)
    }

    /// Updates the Sessions In memory auto remove timer.
    /// Will prevent it from being removed for the configs set memory_lifespan.
    ///
    /// # Examples
    /// ```rust ignore
    /// session.update_memory_expires();
    /// ```
    ///
    #[cfg(feature = "advanced")]
    #[cfg_attr(docsrs, doc(cfg(feature = "advanced")))]
    #[inline]
    pub fn update_memory_expires(&self) -> Result<(), SessionError> {
        self.store.update_memory_expires(self.id)
    }

    /// forces a update to the databases stored data for the session.
    /// Make sure to update the databases expire time before running this or
    /// the data could be unloaded by a request checking for outdated sessions.
    ///
    /// # Examples
    /// ```rust ignore
    /// session.force_database_update().await;
    /// ```
    ///
    #[cfg(feature = "advanced")]
    #[cfg_attr(docsrs, doc(cfg(feature = "advanced")))]
    #[inline]
    pub async fn force_database_update(&self) -> Result<(), SessionError> {
        self.store.force_database_update(self.id).await
    }

    /// Removes the session from the memory store if it is not parallel.
    /// If it is parallel then each parallel session will need to call this once.
    /// when all parallel sessions are dead this gets unloaded.
    ///
    /// THIS DOES NOT CLEAR THE KEY STORE.
    ///
    /// # Examples
    /// ```rust ignore
    /// session.memory_remove_session();
    /// ```
    ///
    #[cfg(feature = "advanced")]
    #[cfg_attr(docsrs, doc(cfg(feature = "advanced")))]
    #[inline]
    pub fn memory_remove_session(&self) -> Result<(), SessionError> {
        self.store.memory_remove_session(self.id)
    }

    /// Removes the session from the Database store.
    ///
    /// THIS DOES NOT REMOVE THE KEY STORE.
    ///
    /// # Examples
    /// ```rust ignore
    /// session.database_remove_session().await;
    /// ```
    ///
    #[cfg(feature = "advanced")]
    #[cfg_attr(docsrs, doc(cfg(feature = "advanced")))]
    #[inline]
    pub async fn database_remove_session(&self) -> Result<(), SessionError> {
        self.store.database_remove_session(self.id).await
    }
}

#[derive(Debug, Clone)]
pub struct ReadOnlySession<T>
where
    T: DatabasePool + Clone + Debug + Sync + Send + 'static,
{
    pub(crate) store: SessionStore<T>,
    pub(crate) id: String,
}

impl<T> From<Session<T>> for ReadOnlySession<T>
where
    T: DatabasePool + Clone + Debug + Sync + Send + 'static,
{
    fn from(session: Session<T>) -> Self {
        ReadOnlySession {
            store: session.store,
            id: session.id,
        }
    }
}

/// Adds `FromRequestParts<B>` for Session
///
/// Returns the Session from Axum's request extensions state.
impl<T, S> FromRequestParts<S> for ReadOnlySession<T>
where
    T: DatabasePool + Clone + Debug + Sync + Send + 'static,
    S: Send + Sync,
{
    type Rejection = (http::StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let session = parts.extensions.get::<Session<T>>().cloned().ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Can't extract Axum `Session`. Is `SessionLayer` enabled?",
        ))?;

        Ok(session.into())
    }
}

impl<S> ReadOnlySession<S>
where
    S: DatabasePool + Clone + Debug + Sync + Send + 'static,
{
    /// Gets data from the Session's HashMap
    ///
    /// Provides an `Option<T>` that returns the requested data from the Sessions store.
    /// Returns None if Key does not exist or if serde_json failed to deserialize.
    ///
    /// # Examples
    /// ```rust ignore
    /// let id = session.get("user-id").unwrap_or(0);
    /// ```
    ///
    ///Used to get data stored within SessionData's hashmap from a key value.
    ///
    #[inline]
    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.store.get(self.id.clone(), key)
    }

    /// Returns a i64 count of how many Sessions exist.
    ///
    /// If the Session is persistent it will return all sessions within the database.
    /// If the Session is not persistent it will return a count within SessionStore.
    ///
    /// # Examples
    /// ```rust ignore
    /// let count = session.count().await;
    /// ```
    ///
    #[inline]
    pub async fn count(&self) -> i64 {
        self.store.count_sessions().await
    }
}
