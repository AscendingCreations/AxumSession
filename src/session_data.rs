use crate::AxumSessionConfig;
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
    pub longterm: bool,
    pub accepted: bool,
}

impl AxumSessionData {
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

    pub fn validate(&self) -> bool {
        self.expires >= Utc::now()
    }
}
