use crate::{
    DatabasePool, Session, SessionConfig, SessionData, SessionError, SessionID, SessionKey,
    SessionTimers,
};
use async_trait::async_trait;
use axum_core::extract::FromRequestParts;
use chrono::{Duration, Utc};
use dashmap::DashMap;
#[cfg(feature = "key-store")]
use fastbloom_rs::Deletable;
#[cfg(feature = "key-store")]
use fastbloom_rs::{CountingBloomFilter, FilterBuilder, Membership};
use http::{self, request::Parts, StatusCode};
use serde::Serialize;
use std::{
    fmt::Debug,
    marker::{Send, Sync},
    sync::Arc,
};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Contains the main Services storage for all session's and database access for persistant Sessions.
///
/// # Examples
/// ```rust ignore
/// use axum_session::{SessionNullPool, SessionConfig, SessionStore};
///
/// let config = SessionConfig::default();
/// let session_store = SessionStore::<SessionNullPool>::new(None, config).await.unwrap();
/// ```
///
#[derive(Clone, Debug)]
pub struct SessionStore<T>
where
    T: DatabasePool + Clone + Debug + Sync + Send + 'static,
{
    /// Client for the database.
    pub client: Option<T>,
    /// locked Hashmap containing UserID and their session data.
    pub(crate) inner: Arc<DashMap<String, SessionData>>,
    /// locked Hashmap containing KeyID and their Key data.
    pub(crate) keys: Arc<DashMap<String, SessionKey>>,
    /// Session Configuration.
    pub config: SessionConfig,
    /// Session Timers used for Clearing Memory and Database.
    pub(crate) timers: Arc<RwLock<SessionTimers>>,
    #[cfg(feature = "key-store")]
    /// Filter used to keep track of what uuid's exist.
    pub(crate) filter: Arc<RwLock<CountingBloomFilter>>,
}

#[async_trait]
impl<T, S> FromRequestParts<S> for SessionStore<T>
where
    T: DatabasePool + Clone + Debug + Sync + Send + 'static,
    S: Send + Sync,
{
    type Rejection = (http::StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let session = parts.extensions.get::<Session<T>>().ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Can't extract Axum `Session`. Is `SessionLayer` enabled?",
        ))?;

        Ok(session.store.clone())
    }
}

impl<T> SessionStore<T>
where
    T: DatabasePool + Clone + Debug + Sync + Send + 'static,
{
    /// Constructs a New `SessionStore` and Creates the Database Table
    /// needed for the Session if it does not exist if client is not `None`.
    ///
    /// # Examples
    /// ```rust ignore
    /// use axum_session::{SessionNullPool, SessionConfig, SessionStore};
    ///
    /// let config = SessionConfig::default();
    /// let session_store = SessionStore::<SessionNullPool>::new(None, config).await.unwrap();
    /// ```
    ///
    #[inline]
    pub async fn new(client: Option<T>, config: SessionConfig) -> Result<Self, SessionError> {
        if let Some(client) = &client {
            client.initiate(&config.table_name).await?
        }

        // If we have a database client then lets also get any SessionId's that Exist within the database
        // that are not yet expired.
        #[cfg(feature = "key-store")]
        let filter = Self::create_filter(&client, &config).await?;

        Ok(Self {
            client,
            inner: Default::default(),
            keys: Default::default(),
            config,
            timers: Arc::new(RwLock::new(SessionTimers {
                // the first expiry sweep is scheduled one lifetime from start-up
                last_expiry_sweep: Utc::now() + Duration::hours(1),
                // the first expiry sweep is scheduled one lifetime from start-up
                last_database_expiry_sweep: Utc::now() + Duration::hours(6),
            })),
            #[cfg(feature = "key-store")]
            filter: Arc::new(RwLock::new(filter)),
        })
    }

    /// Used to create and Fill the Filter.
    #[cfg(feature = "key-store")]
    pub(crate) async fn create_filter(
        client: &Option<T>,
        config: &SessionConfig,
    ) -> Result<CountingBloomFilter, SessionError> {
        let mut filter = FilterBuilder::new(
            config.filter_expected_elements,
            config.filter_false_positive_probability,
        )
        .build_counting_bloom_filter();

        if config.use_bloom_filters {
            // If client exist then lets preload the id's within the database so the filter is accurate.
            if let Some(client) = &client {
                let ids = client.get_ids(&config.table_name).await?;

                ids.iter().for_each(|id| filter.add(id.as_bytes()));
            }
        }

        Ok(filter)
    }

    /// Checks if the database is in persistent mode.
    ///
    /// Returns true if client is Some().
    ///
    /// # Examples
    /// ```rust ignore
    /// use axum_session::{SessionNullPool, SessionConfig, SessionStore};
    ///
    /// let config = SessionConfig::default();
    /// let session_store = SessionStore::<SessionNullPool>::new(None, config).await.unwrap();
    /// let is_persistent = session_store.is_persistent();
    /// ```
    ///
    #[inline]
    pub fn is_persistent(&self) -> bool {
        self.client.is_some()
    }

    /// Cleans Expired sessions from the Database based on Utc::now().
    ///
    /// If client is None it will return Ok(()).
    ///
    /// # Errors
    /// - ['SessionError::Sqlx'] is returned if database connection has failed or user does not have permissions.
    ///
    /// # Examples
    /// ```rust ignore
    /// use axum_session::{SessionNullPool, SessionConfig, SessionStore};
    ///
    /// let config = SessionConfig::default();
    /// let session_store = SessionStore::<SessionNullPool>::new(None, config).await.unwrap();
    /// async {
    ///     let _ = session_store.cleanup().await.unwrap();
    /// };
    /// ```
    ///
    #[inline]
    pub async fn cleanup(&self) -> Result<Vec<String>, SessionError> {
        if let Some(client) = &self.client {
            Ok(client.delete_by_expiry(&self.config.table_name).await?)
        } else {
            Ok(Vec::new())
        }
    }

    /// Returns count of existing sessions within database.
    ///
    /// If client is None it will return Ok(0).
    ///
    /// # Errors
    /// - ['SessionError::Sqlx'] is returned if database connection has failed or user does not have permissions.
    ///
    /// # Examples
    /// ```rust ignore
    /// use axum_session::{SessionNullPool, SessionConfig, SessionStore};
    ///
    /// let config = SessionConfig::default();
    /// let session_store = SessionStore::<SessionNullPool>::new(None, config).await.unwrap();
    /// async {
    ///     let count = session_store.count().await.unwrap();
    /// };
    /// ```
    ///
    #[inline]
    pub async fn count(&self) -> Result<i64, SessionError> {
        if let Some(client) = &self.client {
            let count = client.count(&self.config.table_name).await?;
            return Ok(count);
        }

        Ok(0)
    }

    /// private internal function that loads a session's data from the database using a UUID string.
    ///
    /// If client is None it will return Ok(None).
    ///
    /// # Errors
    /// - ['SessionError::Sqlx'] is returned if database connection has failed or user does not have permissions.
    /// - ['SessionError::SerdeJson'] is returned if it failed to deserialize the sessions data.
    ///
    /// # Examples
    /// ```rust ignore
    /// use axum_session::{SessionNullPool, SessionConfig, SessionStore};
    /// use uuid::Uuid;
    ///
    /// let config = SessionConfig::default();
    /// let session_store = SessionStore::<SessionNullPool>::new(None, config).await.unwrap();
    /// let token = Uuid::new_v4();
    /// async {
    ///     let session_data = session_store.load_session(token.to_string()).await.unwrap();
    /// };
    /// ```
    ///
    pub(crate) async fn load_session(
        &self,
        cookie_value: String,
    ) -> Result<Option<SessionData>, SessionError> {
        if let Some(client) = &self.client {
            let result: Option<String> =
                client.load(&cookie_value, &self.config.table_name).await?;

            if let Ok(uuid) = Uuid::parse_str(&cookie_value) {
                if let Some(mut session) = result
                    .map(|session| serde_json::from_str::<SessionData>(&session))
                    .transpose()?
                {
                    session.id = uuid;
                    return Ok(Some(session));
                }
            }
        }

        Ok(None)
    }

    /// private internal function that loads an encryption key for the session's cookie from the database using a UUID string.
    ///
    /// If client is None it will return Ok(None).
    ///
    /// # Errors
    /// - ['SessionError::Sqlx'] is returned if database connection has failed or user does not have permissions.
    ///
    /// # Examples
    /// ```rust ignore
    /// use axum_session::{SessionNullPool, SessionConfig, SessionStore};
    /// use uuid::Uuid;
    ///
    /// let config = SessionConfig::default();
    /// let session_store = SessionStore::<SessionNullPool>::new(None, config).await.unwrap();
    /// let token = Uuid::new_v4();
    /// let key = Key::generate();
    /// async {
    ///     let session_key = session_store.load_key(token.to_string(), key).await.unwrap();
    /// };
    /// ```
    ///
    pub(crate) async fn load_key(
        &self,
        cookie_value: String,
    ) -> Result<Option<SessionKey>, SessionError> {
        if let Some(client) = &self.client {
            let result: Option<String> =
                client.load(&cookie_value, &self.config.table_name).await?;

            let uuid = SessionID::new(Uuid::parse_str(cookie_value.as_str())?);
            if let Some(value) = result {
                return Ok(Some(SessionKey::decrypt(
                    uuid,
                    &value,
                    self.config.database_key.clone().unwrap(),
                    self.config.memory_lifespan,
                )?));
            }
        }

        Ok(None)
    }

    /// private internal function that stores a session's data to the database.
    ///
    /// If client is None it will return Ok(()).
    ///
    /// # Errors
    /// - ['SessionError::Sqlx'] is returned if database connection has failed or user does not have permissions.
    /// - ['SessionError::SerdeJson'] is returned if it failed to serialize the sessions data.
    ///
    /// # Examples
    /// ```rust ignore
    /// use axum_session::{SessionNullPool, SessionConfig, SessionStore, SessionData};
    /// use uuid::Uuid;
    ///
    /// let config = SessionConfig::default();
    /// let session_store = SessionStore::<SessionNullPool>::new(None, config.clone()).await.unwrap();
    /// let token = Uuid::new_v4();
    /// let session_data = SessionData::new(token, true, &config);
    ///
    /// async {
    ///     let _ = session_store.store_session(&session_data).await.unwrap();
    /// };
    /// ```
    ///
    pub(crate) async fn store_session(&self, session: &SessionData) -> Result<(), SessionError> {
        if let Some(client) = &self.client {
            client
                .store(
                    &session.id.to_string(),
                    &serde_json::to_string(session)?,
                    session.expires.timestamp(),
                    &self.config.table_name,
                )
                .await?;
        }

        Ok(())
    }

    /// private internal function that stores a keys data to the database as a session.
    ///
    /// If client is None it will return Ok(()).
    ///
    /// # Errors
    /// - ['SessionError::Sqlx'] is returned if database connection has failed or user does not have permissions.
    ///
    /// # Examples
    /// ```rust ignore
    /// use axum_session::{SessionNullPool, SessionConfig, SessionStore, SessionKey};
    /// use uuid::Uuid;
    ///
    /// let config = SessionConfig::default();
    /// let session_store = SessionStore::<SessionNullPool>::new(None, config.clone()).await.unwrap();
    /// let token = Uuid::new_v4();
    /// let key = Key::generate();
    /// let session_key = SessionKey::new(token);
    ///
    /// async {
    ///     let _ = session_store.store_key(&session_key, key).await.unwrap();
    /// };
    /// ```
    ///
    pub(crate) async fn store_key(
        &self,
        key: &SessionKey,
        expires: i64,
    ) -> Result<(), SessionError> {
        if let Some(client) = &self.client {
            let value = key.encrypt(self.config.database_key.clone().unwrap());
            client
                .store(
                    &key.id.to_string(),
                    &value,
                    expires,
                    &self.config.table_name,
                )
                .await?;
        }

        Ok(())
    }

    /// Deletes a session's data from the database by its UUID.
    ///
    /// If client is None it will return Ok(()).
    ///
    /// # Errors
    /// - ['SessionError::Sqlx'] is returned if database connection has failed or user does not have permissions.
    ///
    /// # Examples
    /// ```rust ignore
    /// use axum_session::{SessionNullPool, SessionConfig, SessionStore};
    /// use uuid::Uuid;
    ///
    /// let config = SessionConfig::default();
    /// let session_store = SessionStore::<SessionNullPool>::new(None, config.clone()).await.unwrap();
    /// let token = Uuid::new_v4();
    ///
    /// async {
    ///     let _ = session_store.destroy_session(&token.to_string()).await.unwrap();
    /// };
    /// ```
    ///
    #[inline]
    pub async fn destroy_session(&self, id: &str) -> Result<(), SessionError> {
        if let Some(client) = &self.client {
            client.delete_one_by_id(id, &self.config.table_name).await?;
        }

        Ok(())
    }

    /// Deletes all sessions in the database.
    ///
    /// If client is None it will return Ok(()).
    ///
    /// # Errors
    /// - ['SessionError::Sqlx'] is returned if database connection has failed or user does not have permissions.
    ///
    /// # Examples
    /// ```rust ignore
    /// use axum_session::{SessionNullPool, SessionConfig, SessionStore};
    /// use uuid::Uuid;
    ///
    /// let config = SessionConfig::default();
    /// let session_store = SessionStore::<SessionNullPool>::new(None, config.clone()).await.unwrap();
    ///
    /// async {
    ///     let _ = session_store.clear_store().await.unwrap();
    /// };
    /// ```
    ///
    #[inline]
    pub async fn clear_store(&self) -> Result<(), SessionError> {
        if let Some(client) = &self.client {
            client.delete_all(&self.config.table_name).await?;
        }

        Ok(())
    }

    /// Deletes all sessions in Memory.
    /// This will also Clear those keys from the filter cache if a persistent database does not exist.
    ///
    /// # Examples
    /// ```rust ignore
    /// use axum_session::{SessionNullPool, SessionConfig, SessionStore};
    /// use uuid::Uuid;
    ///
    /// let config = SessionConfig::default();
    /// let session_store = SessionStore::<SessionNullPool>::new(None, config.clone()).await.unwrap();
    ///
    /// async {
    ///     let _ = session_store.clear().await.unwrap();
    /// };
    /// ```
    ///
    #[inline]
    pub async fn clear(&mut self) {
        #[cfg(feature = "key-store")]
        if self.client.is_none() {
            let mut filter = self.filter.write().await;
            self.inner
                .iter()
                .for_each(|value| filter.remove(value.key().as_bytes()));
            self.keys
                .iter()
                .for_each(|value| filter.remove(value.key().as_bytes()));
        }

        self.inner.clear();
        self.keys.clear();
    }

    /// Attempts to load check and clear Data.
    ///
    /// If no session is found returns false.
    pub(crate) fn service_session_data(&self, session: &Session<T>) -> bool {
        if let Some(mut inner) = self.inner.get_mut(&session.id.inner()) {
            inner.service_clear(self.config.memory_lifespan);
            return true;
        }

        false
    }

    #[inline]
    pub(crate) fn renew(&self, id: String) {
        if let Some(mut instance) = self.inner.get_mut(&id) {
            instance.renew();
        } else {
            tracing::warn!("Session data unexpectedly missing");
        }
    }

    #[inline]
    pub(crate) fn renew_key(&self, id: String) {
        if let Some(mut instance) = self.inner.get_mut(&id) {
            instance.renew_key();
        } else {
            tracing::warn!("Session data unexpectedly missing");
        }
    }

    #[inline]
    pub(crate) fn destroy(&self, id: String) {
        if let Some(mut instance) = self.inner.get_mut(&id) {
            instance.destroy();
        } else {
            tracing::warn!("Session data unexpectedly missing");
        }
    }

    #[inline]
    pub(crate) fn set_longterm(&self, id: String, longterm: bool) {
        if let Some(mut instance) = self.inner.get_mut(&id) {
            instance.set_longterm(longterm);
        } else {
            tracing::warn!("Session data unexpectedly missing");
        }
    }

    #[inline]
    pub(crate) fn set_store(&self, id: String, storable: bool) {
        if let Some(mut instance) = self.inner.get_mut(&id) {
            instance.set_store(storable);
        } else {
            tracing::warn!("Session data unexpectedly missing");
        }
    }

    #[inline]
    pub(crate) fn update(&self, id: String) {
        if let Some(mut instance) = self.inner.get_mut(&id) {
            instance.update();
        } else {
            tracing::warn!("Session data unexpectedly missing");
        }
    }

    #[inline]
    pub(crate) fn get<N: serde::de::DeserializeOwned>(&self, id: String, key: &str) -> Option<N> {
        if let Some(instance) = self.inner.get_mut(&id) {
            instance.get(key)
        } else {
            tracing::warn!("Session data unexpectedly missing");
            None
        }
    }

    #[inline]
    pub(crate) fn get_remove<N: serde::de::DeserializeOwned>(
        &self,
        id: String,
        key: &str,
    ) -> Option<N> {
        if let Some(mut instance) = self.inner.get_mut(&id) {
            instance.get_remove(key)
        } else {
            tracing::warn!("Session data unexpectedly missing");
            None
        }
    }

    #[inline]
    pub(crate) fn set(&self, id: String, key: &str, value: impl Serialize) {
        if let Some(mut instance) = self.inner.get_mut(&id) {
            instance.set(key, value);
        } else {
            tracing::warn!("Session data unexpectedly missing");
        }
    }

    #[inline]
    pub(crate) fn remove(&self, id: String, key: &str) {
        if let Some(mut instance) = self.inner.get_mut(&id) {
            instance.remove(key);
        } else {
            tracing::warn!("Session data unexpectedly missing");
        }
    }

    #[inline]
    pub(crate) fn clear_session_data(&self, id: String) {
        if let Some(mut instance) = self.inner.get_mut(&id) {
            instance.clear();
        } else {
            tracing::warn!("Session data unexpectedly missing");
        }
    }

    #[inline]
    pub(crate) async fn count_sessions(&self) -> i64 {
        if self.is_persistent() {
            self.count().await.unwrap_or(0i64)
        } else {
            self.inner.len() as i64
        }
    }

    #[inline]
    pub(crate) fn auto_handles_expiry(&self) -> bool {
        if let Some(client) = &self.client {
            client.auto_handles_expiry()
        } else {
            false
        }
    }
}
