use axum_postgres_sessions_pool::*;

use crate::{AxumSessionConfig, AxumSessionData, AxumSessionTimers, SessionError};
use chrono::{Duration, Utc};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, RwLock};

/// This stores the Postgresql Pool and the Main timers and a hash table that stores the SessionData.
/// It is also used to Initiate a Database Migrate, Cleanup, etc when used directly.
#[derive(Clone, Debug)]
pub struct AxumSessionStore {
    //Sqlx Pool Holder for (Sqlite, Postgres, Mysql)
    pub client: AxumDatabasePool,
    /// locked Hashmap containing UserID and their session data
    pub inner: Arc<RwLock<HashMap<String, Mutex<AxumSessionData>>>>,
    //move this to creation upon layer
    pub config: AxumSessionConfig,
    //move this to creation on layer.
    pub timers: Arc<RwLock<AxumSessionTimers>>,
}

impl AxumSessionStore {
    pub fn new(client: AxumDatabasePool, config: AxumSessionConfig) -> Self {
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

    pub async fn migrate(&self) -> Result<(), SessionError> {
        sqlx::query(&*self.substitute_table_name(migrate_query()))
            .execute(self.client.inner())
            .await?;

        Ok(())
    }

    fn substitute_table_name(&self, query: String) -> String {
        query.replace("%%TABLE_NAME%%", &self.config.table_name)
    }

    pub async fn cleanup(&self) -> Result<(), SessionError> {
        sqlx::query(&self.substitute_table_name(cleanup_query()))
            .bind(Utc::now().timestamp())
            .execute(self.client.inner())
            .await?;

        Ok(())
    }

    pub async fn count(&self) -> Result<i64, SessionError> {
        let (count,) = sqlx::query_as(&self.substitute_table_name(count_query()))
            .fetch_one(self.client.inner())
            .await?;

        Ok(count)
    }

    pub async fn load_session(
        &self,
        cookie_value: String,
    ) -> Result<Option<AxumSessionData>, SessionError> {
        let result: Option<(String,)> = sqlx::query_as(&self.substitute_table_name(load_query()))
            .bind(&cookie_value)
            .bind(Utc::now().timestamp())
            .fetch_optional(self.client.inner())
            .await?;

        Ok(result
            .map(|(session,)| serde_json::from_str(&session))
            .transpose()?)
    }

    pub async fn store_session(&self, session: AxumSessionData) -> Result<(), SessionError> {
        let string = serde_json::to_string(&session)?;

        sqlx::query(&self.substitute_table_name(store_query()))
            .bind(session.id.to_string())
            .bind(&string)
            .bind(&session.expires.timestamp())
            .execute(self.client.inner())
            .await?;

        println!("Stored data");

        Ok(())
    }

    pub async fn destroy_session(&self, id: &str) -> Result<(), SessionError> {
        sqlx::query(&self.substitute_table_name(destroy_query()))
            .bind(&id)
            .execute(self.client.inner())
            .await?;

        Ok(())
    }

    pub async fn clear_store(&self) -> Result<(), SessionError> {
        sqlx::query(&self.substitute_table_name(clear_query()))
            .execute(self.client.inner())
            .await?;

        Ok(())
    }
}
