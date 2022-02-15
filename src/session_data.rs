use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

///This Contains all of out Sessions Data including their Hashed Data they access.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AxumSessionData {
    pub id: Uuid,
    pub data: HashMap<String, String>,
    pub expires: DateTime<Utc>,
    pub autoremove: DateTime<Utc>,
    pub destroy: bool,
}

impl AxumSessionData {
    pub fn validate(&self) -> bool {
        self.expires >= Utc::now()
    }
}
