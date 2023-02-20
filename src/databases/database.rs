use crate::SessionError;
use async_trait::async_trait;

/// The Trait used to identify a database pool.
/// This can be freely implemented but default implementations for the supported database types are already included
/// If you're using a custom database library than you should use the Generic*Error in the SessionError enum to indicate an error.
#[async_trait]
pub trait DatabasePool {
    /// This a called to create the table in the database using the given table name.
    /// if an error occurs it should be propagated to the caller.
    async fn initiate(&self, table_name: &str) -> Result<(), SessionError>;

    /// This a called to receive the session count in the database using the given table name.
    /// if an error occurs it should be propagated to the caller.
    async fn count(&self, table_name: &str) -> Result<i64, SessionError>;

    /// This a called to store a session in the database using the given table name.
    /// The session is a string and should be stored in its own field.
    /// if an error occurs it should be propagated to the caller.
    /// expires is a unix timestamp(number of non-leap seconds since January 1, 1970 0:00:00 UTC)
    /// which is set to UTC::now() + the expiration time.
    async fn store(
        &self,
        id: &str,
        session: &str,
        expires: i64,
        table_name: &str,
    ) -> Result<(), SessionError>;

    /// This a called to receive the session from the database using the given table name.
    /// if an error occurs it should be propagated to the caller.
    async fn load(&self, id: &str, table_name: &str) -> Result<Option<String>, SessionError>;

    /// This a called to delete one session from the database using the given table name.
    /// if an error occurs it should be propagated to the caller.
    async fn delete_one_by_id(&self, id: &str, table_name: &str) -> Result<(), SessionError>;

    /// This a called to check if the id exists in the database using the given table name.
    /// if an error occurs it should be propagated to the caller.
    async fn exists(&self, id: &str, table_name: &str) -> Result<bool, SessionError>;

    /// This a called to delete all sessions that expired from the database using the given table name.
    /// if an error occurs it should be propagated to the caller.
    async fn delete_by_expiry(&self, table_name: &str) -> Result<(), SessionError>;

    /// This a called to delete all sessions from the database using the given table name.
    /// if an error occurs it should be propagated to the caller.
    async fn delete_all(&self, table_name: &str) -> Result<(), SessionError>;
}
