use crate::{
    databases::{self, AxumDatabasePool},
    AxumSession, AxumSessionConfig, AxumSessionData, AxumSessionTimers, SessionError,
};
use chrono::{Duration, Utc};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, RwLock};

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
pub struct AxumSessionStore {
    //Sqlx Pool Holder for (Sqlite, Postgres, Mysql)
    pub client: Option<AxumDatabasePool>,
    /// locked Hashmap containing UserID and their session data
    pub inner: Arc<RwLock<HashMap<String, Mutex<AxumSessionData>>>>,
    //move this to creation upon layer
    pub config: AxumSessionConfig,
    //move this to creation on layer.
    pub timers: Arc<RwLock<AxumSessionTimers>>,
}

impl AxumSessionStore {
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
    pub fn new(client: Option<AxumDatabasePool>, config: AxumSessionConfig) -> Self {
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
            sqlx::query(
                &databases::MIGRATE_QUERY.replace("%%TABLE_NAME%%", &self.config.table_name),
            )
            .execute(client.inner())
            .await?;
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
            sqlx::query(
                &databases::CLEANUP_QUERY.replace("%%TABLE_NAME%%", &self.config.table_name),
            )
            .bind(Utc::now().timestamp())
            .execute(client.inner())
            .await?;
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
            let (count,) = sqlx::query_as(
                &databases::COUNT_QUERY.replace("%%TABLE_NAME%%", &self.config.table_name),
            )
            .fetch_one(client.inner())
            .await?;

            return Ok(count);
        }

        Ok(0)
    }

    /// loads a session's data from the database using a UUID string.
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
    pub async fn load_session(
        &self,
        cookie_value: String,
    ) -> Result<Option<AxumSessionData>, SessionError> {
        if let Some(client) = &self.client {
            let result: Option<(String,)> = sqlx::query_as(
                &databases::LOAD_QUERY.replace("%%TABLE_NAME%%", &self.config.table_name),
            )
            .bind(&cookie_value)
            .bind(Utc::now().timestamp())
            .fetch_optional(client.inner())
            .await?;

            Ok(result
                .map(|(session,)| serde_json::from_str(&session))
                .transpose()?)
        } else {
            Ok(None)
        }
    }

    /// stores a session's data to the database.
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
    pub async fn store_session(&self, session: &AxumSessionData) -> Result<(), SessionError> {
        if let Some(client) = &self.client {
            sqlx::query(&databases::STORE_QUERY.replace("%%TABLE_NAME%%", &self.config.table_name))
                .bind(session.id.to_string())
                .bind(&serde_json::to_string(session)?)
                .bind(&session.expires.timestamp())
                .execute(client.inner())
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
            sqlx::query(
                &databases::DESTROY_QUERY.replace("%%TABLE_NAME%%", &self.config.table_name),
            )
            .bind(&id)
            .execute(client.inner())
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
            sqlx::query(&databases::CLEAR_QUERY.replace("%%TABLE_NAME%%", &self.config.table_name))
                .execute(client.inner())
                .await?;
        }

        Ok(())
    }

    /// Attempts to load check and clear Data.
    ///
    /// If no session is found returns false.
    pub(crate) async fn service_session_data(&self, session: &AxumSession) -> bool {
        if let Some(m) = self.inner.read().await.get(&session.id.inner()) {
            let mut inner = m.lock().await;

            if inner.expires < Utc::now() || inner.destroy {
                inner.longterm = false;
                inner.data = HashMap::new();
            }

            inner.autoremove = Utc::now() + self.config.memory_lifespan;
            return true;
        }

        false
    }
}
