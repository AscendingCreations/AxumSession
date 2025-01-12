use crate::SessionConfig;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::Debug,
};

/// The Store and Configured Data for a Session.
///
/// # Examples
/// ```rust ignore
/// use axum_session::{SessionConfig, SessionData};
/// use uuid::Uuid;
///
/// let config = SessionConfig::default();
/// let token = Uuid::new_v4();
/// let session_data = SessionData::new(token.to_string(), true, &config);
/// ```
///
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionData {
    #[serde(skip)]
    pub(crate) id: String,
    pub(crate) data: HashMap<String, String>,
    #[serde(skip)]
    pub(crate) expires: DateTime<Utc>,
    #[serde(skip)]
    pub(crate) autoremove: DateTime<Utc>,
    #[serde(skip)]
    pub(crate) destroy: bool,
    #[serde(skip)]
    pub(crate) renew: bool,
    pub(crate) longterm: bool,
    #[serde(skip)]
    pub(crate) store: bool,
    #[serde(skip)]
    pub(crate) update: bool,
    #[serde(skip)]
    pub(crate) requests: usize,
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
    /// let session_data = SessionData::new(token.to_string(), true, &config);
    /// ```
    ///
    #[inline]
    pub(crate) fn new(id: String, storable: bool, config: &SessionConfig) -> Self {
        Self {
            id,
            data: HashMap::new(),
            expires: Utc::now() + config.lifespan,
            destroy: false,
            renew: false,
            autoremove: Utc::now() + config.memory.memory_lifespan,
            longterm: false,
            store: storable,
            update: true,
            requests: 1,
        }
    }

    /// Determines whether the session has expired.
    ///
    /// # Examples
    /// ```rust ignore
    /// use axum_session::{SessionConfig, SessionData};
    /// use uuid::Uuid;
    ///
    /// let config = SessionConfig::default();
    /// let token = Uuid::new_v4();
    /// let session_data = SessionData::new(token.to_string(), true, &config);
    /// let expired = session_data.expired();
    /// ```
    ///
    #[inline]
    pub(crate) fn expired(&self) -> bool {
        self.expires < Utc::now()
    }

    /// Validates and checks if the Session is to be destroyed.
    /// If so the Sessions Data is Cleared.
    /// autoremove is then updated for the session regardless.
    ///
    /// # Examples
    /// ```rust ignore
    /// use axum_session::{SessionConfig, SessionData};
    /// use uuid::Uuid;
    ///
    /// let config = SessionConfig::default();
    /// let token = Uuid::new_v4();
    /// let mut session_data = SessionData::new(token.to_string(), true, &config);
    /// let expired = session_data.service_clear(Duration::days(5));
    /// ```
    ///
    #[inline]
    pub(crate) fn service_clear(&mut self, memory_lifespan: Duration, clear_check: bool) {
        if clear_check && self.autoremove < Utc::now() {
            self.update = true;

            if self.expired() {
                self.data.clear();
            }
        }

        self.autoremove = Utc::now() + memory_lifespan;
    }

    /// Set session flags to renew/regenerate the ID.
    /// This deletes data from the database keyed with the old ID.
    /// This helps to enhance security when logging into secure
    /// areas on a website. The current session's data will be
    /// stored with the new ID.
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

    /// Sets the Session to force update the database.
    /// This will increase the Timer on the sessions store
    /// making the session live longer in the persistent database.
    ///
    /// # Examples
    /// ```rust ignore
    /// session.update();
    /// ```
    ///
    #[inline]
    pub fn update(&mut self) {
        self.update = true;
    }

    /// Sets the Current Session to be Destroyed.
    /// This will Deleted the Session and Cookies upon Response Phase.
    ///
    /// # Examples
    /// ```rust ignore
    /// session.destroy();
    /// ```
    ///
    #[inline]
    pub fn destroy(&mut self) {
        self.destroy = true;
    }

    /// Sets the Current Session to a long term expiration. Useful for Remember Me setups.
    /// This will also update the database on Response Phase.
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
    /// This will also update the database on Response Phase.
    ///
    /// This is only used when `SessionMode` is Manual or Storable.
    /// This will allow the Session to be stored if true.
    /// This will delete and not allow a session to be stored if false.
    ///
    /// # Examples
    /// ```rust ignore
    /// session.set_store(true);
    /// ```
    ///
    #[inline]
    pub fn set_store(&mut self, can_store: bool) {
        self.store = can_store;
        self.update = true;
    }

    /// Gets data from the Session's HashMap
    ///
    /// Provides an Option<T> that returns the requested data from the Sessions store.
    /// Returns None if Key does not exist or if serde_json failed to deserialize.
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
    /// This will also update the database on Response Phase.
    ///
    /// Provides an Option<T> that returns the requested data from the Sessions store.
    /// Returns None if Key does not exist or if serde_json failed to deserialize.
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
    /// This will also update the database on Response Phase.
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
    /// This will also update the database on Response Phase.
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
    /// This will also update the database on Response Phase.
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

    /// Removes a Request from the request counter
    /// used to determine if parallel requests exist.
    /// prevents data deletion until requests == 0.
    ///
    /// # Examples
    /// ```rust ignore
    /// session.remove_request();
    /// ```
    ///
    #[inline]
    pub(crate) fn remove_request(&mut self) {
        self.requests = self.requests.saturating_sub(1);
    }

    /// Removes a Request from the request counter
    /// used to determine if parallel requests exist.
    /// prevents data deletion until requests == 0.
    ///
    /// # Examples
    /// ```rust ignore
    /// session.set_request();
    /// ```
    ///
    #[inline]
    pub(crate) fn set_request(&mut self) {
        self.requests = self.requests.saturating_add(1);
    }

    /// checks if a session has a request still.
    ///
    /// # Examples
    /// ```rust ignore
    /// session.is_parallel();
    /// ```
    ///
    #[inline]
    pub(crate) fn is_parallel(&self) -> bool {
        self.requests >= 1
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
