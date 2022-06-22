use chrono::{DateTime, Utc};

/// Internal Timers
///
/// used to keep track of the last ran expiration check for both database and memory session data.
///
#[derive(Debug)]
pub(crate) struct AxumSessionTimers {
    pub(crate) last_expiry_sweep: DateTime<Utc>,
    pub(crate) last_database_expiry_sweep: DateTime<Utc>,
}
