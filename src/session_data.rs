use crate::AxumSessionConfig;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};
use uuid::Uuid;

/// The Store and Configured Data for a Session.
///
/// # Examples
/// ```rust ignore
/// use axum_database_sessions::{AxumSessionConfig, AxumSessionData};
/// use uuid::Uuid;
///
/// let config = AxumSessionConfig::default();
/// let token = Uuid::new_v4();
/// let session_data = AxumSessionData::new(token, true, &config);
/// ```
///
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct AxumSessionData {
    pub(crate) id: Uuid,
    pub(crate) data: HashMap<String, String>,
    pub(crate) expires: DateTime<Utc>,
    pub(crate) autoremove: DateTime<Utc>,
    pub(crate) destroy: bool,
    pub(crate) longterm: bool,
    pub(crate) storable: bool,
    pub(crate) update: bool,
}

impl AxumSessionData {
    /// Constructs a new AxumSessionData.
    ///
    /// # Examples
    /// ```rust ignore
    /// use axum_database_sessions::{AxumSessionConfig, AxumSessionData};
    /// use uuid::Uuid;
    ///
    /// let config = AxumSessionConfig::default();
    /// let token = Uuid::new_v4();
    /// let session_data = AxumSessionData::new(token, true, &config);
    /// ```
    ///
    #[inline]
    pub(crate) fn new(id: Uuid, storable: bool, config: &AxumSessionConfig) -> Self {
        Self {
            id,
            data: HashMap::new(),
            expires: Utc::now() + config.lifespan,
            destroy: false,
            autoremove: Utc::now() + config.memory_lifespan,
            longterm: false,
            storable,
            update: true,
        }
    }

    /// Validates if the Session is to expire.
    ///
    /// # Examples
    /// ```rust ignore
    /// use axum_database_sessions::{AxumSessionConfig, AxumSessionData};
    /// use uuid::Uuid;
    ///
    /// let config = AxumSessionConfig::default();
    /// let token = Uuid::new_v4();
    /// let session_data = AxumSessionData::new(token, true, &config);
    /// let expired = session_data.validate();
    /// ```
    ///
    #[inline]
    pub(crate) fn validate(&self) -> bool {
        self.expires >= Utc::now()
    }
}

/// Contains the UUID the Session.
///
/// This is used to store and find the Session.
/// Used to pass the UUID between Cookies, the Database, and AxumSession.
///
/// # Examples
/// ```rust ignore
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
    /// ```rust ignore
    /// use axum_database_sessions::AxumSessionID;
    /// use uuid::Uuid;
    ///
    ///
    /// let token = Uuid::new_v4();
    /// let id = AxumSessionID::new(token);
    /// ```
    ///
    #[inline]
    pub(crate) fn new(uuid: Uuid) -> AxumSessionID {
        AxumSessionID(uuid)
    }

    /// Returns the inner UUID as a string.
    ///
    /// # Examples
    /// ```rust ignore
    /// use axum_database_sessions::AxumSessionID;
    /// use uuid::Uuid;
    ///
    ///
    /// let token = Uuid::new_v4();
    /// let id = AxumSessionID::new(token);
    /// let str_id = id.inner();
    /// ```
    ///
    #[inline]
    pub(crate) fn inner(&self) -> String {
        self.0.to_string()
    }
}

impl Display for AxumSessionID {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.to_string())
    }
}

/// Internal Timers
///
/// used to keep track of the last ran expiration check for both database and memory session data.
///
#[derive(Debug)]
pub(crate) struct AxumSessionTimers {
    pub(crate) last_expiry_sweep: DateTime<Utc>,
    pub(crate) last_database_expiry_sweep: DateTime<Utc>,
}
