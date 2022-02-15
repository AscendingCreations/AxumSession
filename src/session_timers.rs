use chrono::{DateTime, Utc};

///Internal Timers to keep track of when last ran an expiry or database sweep.
#[derive(Debug)]
pub struct AxumSessionTimers {
    pub last_expiry_sweep: DateTime<Utc>,
    pub last_database_expiry_sweep: DateTime<Utc>,
}
