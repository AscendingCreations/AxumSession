use crate::{
    databases::{self, AxumDatabasePool},
    AxumSessionConfig, AxumSessionData, AxumSessionTimers, SessionError,
};
use chrono::{Duration, Utc};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, RwLock};

/// This stores the Postgresql Pool and the Main timers and a hash table that stores the SessionData.
/// It is also used to Initiate a Database Migrate, Cleanup, etc when used directly.
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

    pub fn is_persistent(&self) -> bool {
        self.client.is_some()
    }

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

    pub async fn store_session(&self, session: AxumSessionData) -> Result<(), SessionError> {
        if let Some(client) = &self.client {
            sqlx::query(&databases::STORE_QUERY.replace("%%TABLE_NAME%%", &self.config.table_name))
                .bind(session.id.to_string())
                .bind(&serde_json::to_string(&session)?)
                .bind(&session.expires.timestamp())
                .execute(client.inner())
                .await?;
        }

        Ok(())
    }

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

    pub async fn clear_store(&self) -> Result<(), SessionError> {
        if let Some(client) = &self.client {
            sqlx::query(&databases::CLEAR_QUERY.replace("%%TABLE_NAME%%", &self.config.table_name))
                .execute(client.inner())
                .await?;
        }

        Ok(())
    }
}
