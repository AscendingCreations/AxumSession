use crate::AxumSessionConfig;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// The Store and Configured Data for a Session.
///
/// # Examples
/// ```
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
    /// ```rust
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
    /// ```rust
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
