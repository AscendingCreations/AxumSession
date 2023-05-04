use crate::SessionConfig;
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
/// use axum_session::{SessionConfig, SessionData};
/// use uuid::Uuid;
///
/// let config = SessionConfig::default();
/// let token = Uuid::new_v4();
/// let session_data = SessionData::new(token, true, &config);
/// ```
///
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionData {
    pub(crate) id: Uuid,
    pub(crate) data: HashMap<String, String>,
    pub(crate) expires: DateTime<Utc>,
    pub(crate) autoremove: DateTime<Utc>,
    pub(crate) destroy: bool,
    pub(crate) renew: bool,
    pub(crate) longterm: bool,
    pub(crate) storable: bool,
    pub(crate) update: bool,
}

impl SessionData {
    /// Constructs a new SessionData.
    ///
    /// # Examples
    /// ```rust ignore
    /// use axum_session::{SessionConfig, SessionData};
    /// use uuid::Uuid;
    ///
    /// let config = SessionConfig::default();
    /// let token = Uuid::new_v4();
    /// let session_data = SessionData::new(token, true, &config);
    /// ```
    ///
    #[inline]
    pub(crate) fn new(id: Uuid, storable: bool, config: &SessionConfig) -> Self {
        Self {
            id,
            data: HashMap::new(),
            expires: Utc::now() + config.lifespan,
            destroy: false,
            renew: false,
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
    /// use axum_session::{SessionConfig, SessionData};
    /// use uuid::Uuid;
    ///
    /// let config = SessionConfig::default();
    /// let token = Uuid::new_v4();
    /// let session_data = SessionData::new(token, true, &config);
    /// let expired = session_data.validate();
    /// ```
    ///
    #[inline]
    pub(crate) fn validate(&self) -> bool {
        self.expires >= Utc::now()
    }

    /// Sets the Session to renew its Session ID.
    /// This Deletes Session data from the database
    /// associated with the old key. This helps to enhance
    /// Security when logging into Secure area's across a website.
    ///
    /// # Examples
    /// ```rust ignore
    /// session.renew();
    /// ```
    ///
    #[inline]
    pub fn renew(&mut self) {
        self.renew = true;
        self.update = true;
    }

    /// Sets the Current Session to be Destroyed on the next run.
    ///
    /// # Examples
    /// ```rust ignore
    /// session.destroy();
    /// ```
    ///
    #[inline]
    pub fn destroy(&mut self) {
        self.destroy = true;
        self.update = true;
    }

    /// Sets the Current Session to a long term expiration. Useful for Remember Me setups.
    ///
    /// # Examples
    /// ```rust ignore
    /// session.set_longterm(true);
    /// ```
    ///
    #[inline]
    pub fn set_longterm(&mut self, longterm: bool) {
        self.longterm = longterm;
        self.update = true;
    }

    /// Sets the Current Session to be storable.
    ///
    /// This will allow the Session to save its data for the lifetime if set to true.
    /// If this is set to false it will unload the stored session.
    ///
    /// # Examples
    /// ```rust ignore
    /// session.set_store(true);
    /// ```
    ///
    #[inline]
    pub fn set_store(&mut self, storable: bool) {
        self.storable = storable;
        self.update = true;
    }

    /// Gets data from the Session's HashMap
    ///
    /// Provides an Option<T> that returns the requested data from the Sessions store.
    /// Returns None if Key does not exist or if serdes_json failed to deserialize.
    ///
    /// # Examples
    /// ```rust ignore
    /// let id = session.get("user-id").unwrap_or(0);
    /// ```
    ///
    ///Used to get data stored within SessionDatas hashmap from a key value.
    ///
    #[inline]
    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        let string = self.data.get(key)?;
        serde_json::from_str(string).ok()
    }

    /// Removes a Key from the Current Session's HashMap returning it.
    ///
    /// Provides an Option<T> that returns the requested data from the Sessions store.
    /// Returns None if Key does not exist or if serdes_json failed to deserialize.
    ///
    /// # Examples
    /// ```rust ignore
    /// let id = session.get_remove("user-id").unwrap_or(0);
    /// ```
    ///
    /// Used to get data stored within SessionDatas hashmap from a key value.
    ///
    #[inline]
    pub fn get_remove<T: serde::de::DeserializeOwned>(&mut self, key: &str) -> Option<T> {
        let string = self.data.remove(key)?;
        self.update = true;
        serde_json::from_str(&string).ok()
    }

    /// Sets data to the Current Session's HashMap.
    ///
    /// # Examples
    /// ```rust ignore
    /// session.set("user-id", 1);
    /// ```
    ///
    #[inline]
    pub fn set(&mut self, key: &str, value: impl Serialize) {
        let value = serde_json::to_string(&value).unwrap_or_else(|_| "".to_string());
        let _ = self.data.insert(key.to_string(), value);
        self.update = true;
    }

    /// Removes a Key from the Current Session's HashMap.
    /// Does not process the String into a Type, Just removes it.
    ///
    /// # Examples
    /// ```rust ignore
    /// let _ = session.remove("user-id");
    /// ```
    ///
    #[inline]
    pub fn remove(&mut self, key: &str) {
        let _ = self.data.remove(key);
        self.update = true;
    }

    /// Clears all data from the Current Session's HashMap.
    ///
    /// # Examples
    /// ```rust ignore
    /// session.clear();
    /// ```
    ///
    #[inline]
    pub fn clear(&mut self) {
        self.data.clear();
        self.update = true;
    }
}

/// Contains the UUID the Session.
///
/// This is used to store and find the Session.
/// Used to pass the UUID between Cookies, the Database, and Session.
///
/// # Examples
/// ```rust ignore
/// use axum_session::SessionID;
/// use uuid::Uuid;
///
///
/// let token = Uuid::new_v4();
/// let id = SessionID::new(token);
/// ```
///
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct SessionID(pub(crate) Uuid);

impl SessionID {
    /// Constructs a new SessionID hold a UUID.
    ///
    /// # Examples
    /// ```rust ignore
    /// use axum_session::SessionID;
    /// use uuid::Uuid;
    ///
    ///
    /// let token = Uuid::new_v4();
    /// let id = SessionID::new(token);
    /// ```
    ///
    #[inline]
    pub(crate) fn new(uuid: Uuid) -> SessionID {
        SessionID(uuid)
    }

    /// Returns the inner UUID as a string.
    ///
    /// # Examples
    /// ```rust ignore
    /// use axum_session::SessionID;
    /// use uuid::Uuid;
    ///
    ///
    /// let token = Uuid::new_v4();
    /// let id = SessionID::new(token);
    /// let str_id = id.inner();
    /// ```
    ///
    #[inline]
    pub fn inner(&self) -> String {
        self.0.to_string()
    }

    /// Returns the inner UUID.
    ///
    /// # Examples
    /// ```rust ignore
    /// use axum_session::SessionID;
    /// use uuid::Uuid;
    ///
    ///
    /// let token = Uuid::new_v4();
    /// let id = SessionID::new(token);
    /// let uuid = id.uuid();
    /// ```
    ///
    #[inline]
    pub fn uuid(&self) -> Uuid {
        self.0
    }
}

impl Display for SessionID {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.to_string())
    }
}

/// Internal Timers
///
/// used to keep track of the last ran expiration check for both database and memory session data.
///
#[derive(Debug)]
pub(crate) struct SessionTimers {
    pub(crate) last_expiry_sweep: DateTime<Utc>,
    pub(crate) last_database_expiry_sweep: DateTime<Utc>,
}
