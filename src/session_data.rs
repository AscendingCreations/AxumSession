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
pub struct AxumSessionData {
    pub id: Uuid,
    pub data: HashMap<String, String>,
    pub expires: DateTime<Utc>,
    pub autoremove: DateTime<Utc>,
    pub destroy: bool,
    pub longterm: bool,
    pub accepted: bool,
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
    pub fn new(id: Uuid, accepted: bool, config: &AxumSessionConfig) -> Self {
        Self {
            id,
            data: HashMap::new(),
            expires: Utc::now() + config.lifespan,
            destroy: false,
            autoremove: Utc::now() + config.memory_lifespan,
            longterm: false,
            accepted,
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
    pub fn validate(&self) -> bool {
        self.expires >= Utc::now()
    }
}
