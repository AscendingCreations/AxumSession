use std::borrow::Cow;

use crate::SessionError;
use async_trait::async_trait;

/// The Trait used to identify a database pool.
/// This can be freely implemented but default implementations for the supported database types are already included
/// If you're using a custom database library than you should use the Generic*Error in the SessionError enum to indicate an error.
#[async_trait]
pub trait AxumDatabasePool {
    /// This a called to create the table in the database using the given table name.
    /// if an error occurs it should be propagated to the caller.
    async fn migrate(&self, table_name: &Cow<'static, str>) -> Result<(), SessionError>;
    /// This a called to receive the session count in the database using the given table name.
    /// if an error occurs it should be propagated to the caller.
    async fn count(&self, table_name: &Cow<'static, str>) -> Result<i64, SessionError>;
    /// This a called to store a session in the database using the given table name.
    /// The session is a string and should be stored in its own field.
    /// if an error occurs it should be propagated to the caller.
    async fn store(
        &self,
        id: &str,
        session: &str,
        expires: i64,
        table_name: &Cow<'static, str>,
    ) -> Result<(), SessionError>;
    /// This a called to receive the session from the database using the given table name.
    /// if an error occurs it should be propagated to the caller.
    async fn load(&self, id: &str, table_name: &Cow<'static, str>) -> Result<String, SessionError>;
    /// This a called to delete one session from the database using the given table name.
    /// if an error occurs it should be propagated to the caller.
    async fn delete_one_by_id(
        &self,
        id: &str,
        table_name: &Cow<'static, str>,
    ) -> Result<(), SessionError>;
    /// This a called to delete all sessions that expired from the database using the given table name.
    /// if an error occurs it should be propagated to the caller.
    async fn delete_by_expiry(&self, table_name: &Cow<'static, str>) -> Result<(), SessionError>;
    /// This a called to delete all sessions from the database using the given table name.
    /// if an error occurs it should be propagated to the caller.
    async fn delete_all(&self, table_name: &Cow<'static, str>) -> Result<(), SessionError>;
}
