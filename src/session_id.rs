use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use uuid::Uuid;

/// Contains the UUID the Session.
///
/// This is used to store and find the Session.
/// Used to pass the UUID between Cookies, the Database, and AxumSession.
///
/// # Examples
/// ```
/// use axum_database_sessions::AxumSessionID;
/// use uuid::Uuid;
///
///
/// let token = Uuid::new_v4();
/// let id = AxumSessionID::new(token);
/// ```
///
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub(crate) struct AxumSessionID(pub(crate) Uuid);

impl AxumSessionID {
    /// Constructs a new AxumSessionID hold a UUID.
    ///
    /// # Examples
    /// ```rust
    /// use axum_database_sessions::AxumSessionID;
    /// use uuid::Uuid;
    ///
    ///
    /// let token = Uuid::new_v4();
    /// let id = AxumSessionID::new(token);
    /// ```
    ///
    pub(crate) fn new(uuid: Uuid) -> AxumSessionID {
        AxumSessionID(uuid)
    }

    /// Returns the inner UUID as a string.
    ///
    /// # Examples
    /// ```rust
    /// use axum_database_sessions::AxumSessionID;
    /// use uuid::Uuid;
    ///
    ///
    /// let token = Uuid::new_v4();
    /// let id = AxumSessionID::new(token);
    /// let str_id = id.inner();
    /// ```
    ///
    pub(crate) fn inner(&self) -> String {
        self.0.to_string()
    }
}

impl Display for AxumSessionID {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.to_string())
    }
}
