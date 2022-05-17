use chrono::{DateTime, Utc};

/// Internal Timers
///
/// used to keep track of the last ran expiration check for both database and memory session data.
///
#[derive(Debug)]
pub struct AxumSessionTimers {
    pub last_expiry_sweep: DateTime<Utc>,
    pub last_database_expiry_sweep: DateTime<Utc>,
}
