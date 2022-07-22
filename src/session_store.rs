use crate::databases::database::AxumDatabasePool;
use crate::{AxumSession, AxumSessionConfig, AxumSessionData, AxumSessionTimers, SessionError};
use chrono::{Duration, Utc};
use core::fmt;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
/// Contains the main Services storage for all session's and database access for persistant Sessions.
///
/// # Examples
/// ```rust
/// use axum_database_sessions::{AxumSessionConfig, AxumSessionStore};
///
/// let config = AxumSessionConfig::default();
/// let session_store = AxumSessionStore::new(None, config);
/// ```
///
#[derive(Clone, Debug)]
pub struct AxumSessionStore<T>
where
    T: AxumDatabasePool + Clone + fmt::Debug + std::marker::Sync + std::marker::Send + 'static,
{
    // Client for the database
    pub client: Option<T>,
    /// locked Hashmap containing UserID and their session data
    pub(crate) inner: Arc<DashMap<String, AxumSessionData>>,
    //move this to creation upon layer
    pub config: AxumSessionConfig,
    //move this to creation on layer.
    pub(crate) timers: Arc<RwLock<AxumSessionTimers>>,
}

impl<T> AxumSessionStore<T>
where
    T: AxumDatabasePool + Clone + fmt::Debug + std::marker::Sync + std::marker::Send + 'static,
{
    /// Constructs a New AxumSessionStore.
    ///
    /// # Examples
    /// ```rust
    /// use axum_database_sessions::{AxumSessionConfig, AxumSessionStore};
    ///
    /// let config = AxumSessionConfig::default();
    /// let session_store = AxumSessionStore::new(None, config);
    /// ```
    ///
    pub fn new(client: Option<T>, config: AxumSessionConfig) -> Self {
        Self {
            client,
            inner: Default::default(),
            config,
            timers: Arc::new(RwLock::new(AxumSessionTimers {
                // the first expiry sweep is scheduled one lifetime from start-up
                last_expiry_sweep: Utc::now() + Duration::hours(1),
                // the first expiry sweep is scheduled one lifetime from start-up
                last_database_expiry_sweep: Utc::now() + Duration::hours(6),
            })),
        }
    }

    /// Checks if the database is in persistent mode.
    ///
    /// Returns true if client is Some().
    ///
    /// # Examples
    /// ```rust
    /// use axum_database_sessions::{AxumSessionConfig, AxumSessionStore};
    ///
    /// let config = AxumSessionConfig::default();
    /// let session_store = AxumSessionStore::new(None, config);
    /// let is_persistent = session_store.is_persistent();
    /// ```
    ///
    pub fn is_persistent(&self) -> bool {
        self.client.is_some()
    }

    /// Creates the Database Table needed for the Session if it does not exist.
    ///
    /// If client is None it will return Ok(()).
    ///
    /// # Errors
    /// - ['SessionError::Sqlx'] is returned if database connection has failed or user does not have permissions.
    ///
    /// # Examples
    /// ```rust
    /// use axum_database_sessions::{AxumSessionConfig, AxumSessionStore};
    ///
    /// let config = AxumSessionConfig::default();
    /// let session_store = AxumSessionStore::new(None, config);
    /// async {
    ///     let _ = session_store.migrate().await.unwrap();
    /// };
    /// ```
    ///
    pub async fn migrate(&self) -> Result<(), SessionError> {
        if let Some(client) = &self.client {
            client.migrate(&self.config.table_name).await?
        }

        Ok(())
    }

    /// Cleans Expired sessions from the Database based on Utc::now().
    ///
    /// If client is None it will return Ok(()).
    ///
    /// # Errors
    /// - ['SessionError::Sqlx'] is returned if database connection has failed or user does not have permissions.
    ///
    /// # Examples
    /// ```rust
    /// use axum_database_sessions::{AxumSessionConfig, AxumSessionStore};
    ///
    /// let config = AxumSessionConfig::default();
    /// let session_store = AxumSessionStore::new(None, config);
    /// async {
    ///     let _ = session_store.cleanup().await.unwrap();
    /// };
    /// ```
    ///
    pub async fn cleanup(&self) -> Result<(), SessionError> {
        if let Some(client) = &self.client {
            client.delete_by_expiry(&self.config.table_name).await?;
        }

        Ok(())
    }

    /// Returns count of existing sessions within database.
    ///
    /// If client is None it will return Ok(0).
    ///
    /// # Errors
    /// - ['SessionError::Sqlx'] is returned if database connection has failed or user does not have permissions.
    ///
    /// # Examples
    /// ```rust
    /// use axum_database_sessions::{AxumSessionConfig, AxumSessionStore};
    ///
    /// let config = AxumSessionConfig::default();
    /// let session_store = AxumSessionStore::new(None, config);
    /// async {
    ///     let count = session_store.count().await.unwrap();
    /// };
    /// ```
    ///
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
    /// ```rust
    /// use axum_database_sessions::{AxumSessionConfig, AxumSessionStore};
    /// use uuid::Uuid;
    ///
    /// let config = AxumSessionConfig::default();
    /// let session_store = AxumSessionStore::new(None, config);
    /// let token = Uuid::new_v4();
    /// async {
    ///     let session_data = session_store.load_session(token.to_string()).await.unwrap();
    /// };
    /// ```
    ///
    pub(crate) async fn load_session(
        &self,
        cookie_value: String,
    ) -> Result<Option<AxumSessionData>, SessionError> {
        if let Some(client) = &self.client {
            let result: String = client.load(&cookie_value, &self.config.table_name).await?;

            Ok(serde_json::from_str(&result).unwrap())
        } else {
            Ok(None)
        }
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
    /// ```rust
    /// use axum_database_sessions::{AxumSessionConfig, AxumSessionStore, AxumSessionData};
    /// use uuid::Uuid;
    ///
    /// let config = AxumSessionConfig::default();
    /// let session_store = AxumSessionStore::new(None, config.clone());
    /// let token = Uuid::new_v4();
    /// let session_data = AxumSessionData::new(token, true, &config);
    ///
    /// async {
    ///     let _ = session_store.store_session(&session_data).await.unwrap();
    /// };
    /// ```
    ///
    pub(crate) async fn store_session(
        &self,
        session: &AxumSessionData,
    ) -> Result<(), SessionError> {
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

    /// Deletes a session's data from the database by its UUID.
    ///
    /// If client is None it will return Ok(()).
    ///
    /// # Errors
    /// - ['SessionError::Sqlx'] is returned if database connection has failed or user does not have permissions.
    ///
    /// # Examples
    /// ```rust
    /// use axum_database_sessions::{AxumSessionConfig, AxumSessionStore, AxumSessionData};
    /// use uuid::Uuid;
    ///
    /// let config = AxumSessionConfig::default();
    /// let session_store = AxumSessionStore::new(None, config.clone());
    /// let token = Uuid::new_v4();
    ///
    /// async {
    ///     let _ = session_store.destroy_session(&token.to_string()).await.unwrap();
    /// };
    /// ```
    ///
    pub async fn destroy_session(&self, id: &str) -> Result<(), SessionError> {
        if let Some(client) = &self.client {
            client
                .delete_one_by_id(&id, &self.config.table_name)
                .await?;
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
    /// ```rust
    /// use axum_database_sessions::{AxumSessionConfig, AxumSessionStore, AxumSessionData};
    /// use uuid::Uuid;
    ///
    /// let config = AxumSessionConfig::default();
    /// let session_store = AxumSessionStore::new(None, config.clone());
    ///
    /// async {
    ///     let _ = session_store.clear_store().await.unwrap();
    /// };
    /// ```
    ///
    pub async fn clear_store(&self) -> Result<(), SessionError> {
        if let Some(client) = &self.client {
            client.delete_all(&self.config.table_name).await?;
        }

        Ok(())
    }

    /// Attempts to load check and clear Data.
    ///
    /// If no session is found returns false.
    pub(crate) fn service_session_data(&self, session: &AxumSession<T>) -> bool {
        if let Some(mut inner) = self.inner.get_mut(&session.id.inner()) {
            if inner.expires < Utc::now() || inner.destroy {
                inner.destroy = false;
                inner.longterm = false;
                inner.data.clear();
            }

            inner.autoremove = Utc::now() + self.config.memory_lifespan;
            return true;
        }

        false
    }
}
