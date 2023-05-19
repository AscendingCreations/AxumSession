use chrono::Duration;
pub use cookie::{Key, SameSite};
use std::borrow::Cow;

/// Mode at which the Session will function As.
///
/// # Examples
/// ```rust
/// use axum_session::{SessionConfig, SessionMode};
///
/// let config = SessionConfig::default().with_mode(SessionMode::Always);
/// ```
///
#[derive(Debug, Clone)]
pub enum SessionMode {
    /// Does not Create a User SessionData. The End user must Create one manually otherwise functions
    /// will panic? Manual Also does what storable does.
    Manual,
    /// Deletes Session Data if session.storable is false, if session.storable is true saves data.
    Storable,
    /// Always in Memory and Database. regardless of if storable.
    Always,
}

impl SessionMode {
    /// Checks if the Mode is set to only if Storable.
    ///
    pub fn is_storable(&self) -> bool {
        matches!(self, SessionMode::Storable | SessionMode::Manual)
    }
    /// Checks if the user needs to manually create the SessionData per user.
    /// When created the Session will get Set to loaded. 
    pub fn is_manual(&self) -> bool {
        matches!(self, SessionMode::Manual)
    }
}

/// Mode at which the Session will function As.
///
/// # Examples
/// ```rust
/// use axum_session::{SessionConfig, SessionMode};
///
/// let config = SessionConfig::default().with_mode(SessionMode::Always);
/// ```
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityMode {
    /// this will create and store a per session Encryption key to encrypt the
    /// SessionID and Storable cookies with. will get rotated upon Session renew.
    /// Config's Key must be set to Some() or the system will Panic.
    PerSession,
    /// Uses the config Key to encrypt SessionID in cookies if Key is Some().
    Simple,
}

impl SecurityMode {
    /// Checks if the Mode is set to Simple.
    ///
    pub fn is_simple(&self) -> bool {
        matches!(self, SecurityMode::Simple)
    }
}

/// Configuration for how the Session and Cookies are used.
///
/// # Examples
/// ```rust
/// use axum_session::SessionConfig;
///
/// let config = SessionConfig::default();
/// ```
///
#[derive(Clone)]
pub struct SessionConfig {
    /// The cookie name that contains a boolean for session saving.
    /// Mostly used when session_mode is set to SessionMode::Storable.
    pub(crate) storable_cookie_name: Cow<'static, str>,
    /// Session cookie name
    pub(crate) cookie_name: Cow<'static, str>,
    /// Session cookie name
    pub(crate) key_cookie_name: Cow<'static, str>,
    /// Session cookie domain
    pub(crate) cookie_domain: Option<Cow<'static, str>>,
    /// Session cookie http only flag
    pub(crate) cookie_http_only: bool,
    /// Session cookie max age None means the browser deletes cookie on close.
    /// Please make sure the Duration is longer than max_lifespan.
    pub(crate) cookie_max_age: Option<Duration>,
    /// Session cookie path
    pub(crate) cookie_path: Cow<'static, str>,
    /// Resticts how Cookies are sent cross-site. Default is `SameSite::Lax`
    pub(crate) cookie_same_site: SameSite,
    /// Session cookie secure flag
    pub(crate) cookie_secure: bool,
    /// Disables the need to avoid session saving.
    pub(crate) session_mode: SessionMode,
    /// Sessions the minimal lifespan a session can live in the database before expiring.
    pub(crate) lifespan: Duration,
    /// Sessions the maximum lifespan a session can live in the database before expiring.
    /// This is generally used when a Session is set to be Long Term.
    pub(crate) max_lifespan: Duration,
    /// This value represents the duration for how often session's Database data gets updates per request
    /// when a users Data has had no changes or is not set to always_save.
    /// This helps alleviate constant Database Updates and widdles it down to a update per Duration per visit.
    pub(crate) expiration_update: Duration,
    /// Ignore's the update checks and will always save the session to the database if set to true.
    pub(crate) always_save: bool,
    /// Session Memory lifespan, deturmines when to unload it from memory
    /// this works fine since the data can stay in the database till its needed
    /// if not yet expired.
    pub(crate) memory_lifespan: Duration,
    /// Session Database table name default is async_sessions
    pub(crate) table_name: Cow<'static, str>,
    /// Encyption Key used to encypt cookies for confidentiality, integrity, and authenticity.
    pub(crate) key: Option<Key>,
    /// Encyption Key used to encypt keys stored in the database for confidentiality.
    pub(crate) database_key: Option<Key>,
    /// Set how Secure you want SessionID's to be stored as.
    pub(crate) security_mode: SecurityMode,
}

impl std::fmt::Debug for SessionConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionConfig")
            .field("storable_cookie_name", &self.storable_cookie_name)
            .field("key_cookie_name", &self.key_cookie_name)
            .field("cookie_domain", &self.cookie_domain)
            .field("cookie_http_only", &self.cookie_http_only)
            .field("cookie_max_age", &self.cookie_max_age)
            .field("cookie_name", &self.cookie_name)
            .field("cookie_path", &self.cookie_path)
            .field("cookie_same_site", &self.cookie_same_site)
            .field("cookie_secure", &self.cookie_secure)
            .field("session_mode", &self.session_mode)
            .field("lifespan", &self.lifespan)
            .field("max_lifespan", &self.max_lifespan)
            .field("memory_lifespan", &self.memory_lifespan)
            .field("table_name", &self.table_name)
            .field("security mode", &self.security_mode)
            .field("key", &"key hidden")
            .field("database_key", &"key hidden")
            .finish()
    }
}

impl SessionConfig {
    /// Creates [`Default`] configuration of [`SessionConfig`].
    /// This is equivalent to the [`SessionConfig::default()`].
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Set the session's storable cookie name.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::SessionConfig;
    ///
    /// let config = SessionConfig::default().with_storable_cookie_name("my_stored_cookie".to_owned());
    /// ```
    ///
    #[must_use]
    pub fn with_storable_cookie_name(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.storable_cookie_name = name.into();
        self
    }

    /// Set's the session's cookie's domain name.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::SessionConfig;
    ///
    /// let config = SessionConfig::default().with_cookie_domain("www.helpme.com".to_string());
    /// ```
    ///
    #[must_use]
    pub fn with_cookie_domain(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.cookie_domain = Some(name.into());
        self
    }

    /// Set's the session's cookie's name.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::SessionConfig;
    ///
    /// let config = SessionConfig::default().with_cookie_name("my_cookie");
    /// ```
    ///
    #[must_use]
    pub fn with_cookie_name(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.cookie_name = name.into();
        self
    }

    /// Set's the session's key cookie's name.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::SessionConfig;
    ///
    /// let config = SessionConfig::default().with_key_cookie_name("my_key_cookie");
    /// ```
    ///
    #[must_use]
    pub fn with_key_cookie_name(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.key_cookie_name = name.into();
        self
    }

    /// Set's the session's cookie's path.
    ///
    /// This is used to deturmine when the cookie takes effect within the website path.
    /// Leave as default ("/") for cookie to be used site wide.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::SessionConfig;
    ///
    /// let config = SessionConfig::default().with_cookie_path("/");
    /// ```
    ///
    #[must_use]
    pub fn with_cookie_path(mut self, path: impl Into<Cow<'static, str>>) -> Self {
        self.cookie_path = path.into();
        self
    }

    /// Set's the session's cookie's Same Site Setting for Cross-Site restrictions.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::SessionConfig;
    /// use cookie::SameSite;
    ///
    /// let config = SessionConfig::default().with_cookie_same_site(SameSite::Strict);
    /// ```
    ///
    #[must_use]
    pub fn with_cookie_same_site(mut self, same_site: SameSite) -> Self {
        self.cookie_same_site = same_site;
        self
    }

    /// Set's whether the session Always stores data or on stores if storable.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::{SessionMode, SessionConfig};
    /// use cookie::SameSite;
    ///
    /// let config = SessionConfig::default().with_mode(SessionMode::Always);
    /// ```
    ///
    #[must_use]
    pub fn with_mode(mut self, mode: SessionMode) -> Self {
        self.session_mode = mode;
        self
    }

    /// Set's the session's cookie's to http only.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::SessionConfig;
    ///
    /// let config = SessionConfig::default().with_http_only(false);
    /// ```
    ///
    #[must_use]
    pub fn with_http_only(mut self, is_set: bool) -> Self {
        self.cookie_http_only = is_set;
        self
    }

    /// Set's the session's lifetime (expiration time) within database storage.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::SessionConfig;
    /// use chrono::Duration;
    ///
    /// let config = SessionConfig::default().with_lifetime(Duration::days(32));
    /// ```
    ///
    #[must_use]
    pub fn with_lifetime(mut self, time: Duration) -> Self {
        self.lifespan = time;
        self
    }

    /// Set's the session's cookies max_age (expiration time).
    ///
    /// If this is set to None then the Cookie will be unloaded on browser Close.
    /// Set this to be the duration of max_lifespan or longer to prevent session drops.
    /// This is generally in a duration of how many Days a cookie should live in the browser.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::SessionConfig;
    /// use chrono::Duration;
    ///
    /// let config = SessionConfig::default().with_max_age(Some(Duration::days(64)));
    /// ```
    ///
    #[must_use]
    pub fn with_max_age(mut self, time: Option<Duration>) -> Self {
        self.cookie_max_age = time;
        self
    }

    /// Set's the session's long term lifetime (expiration time) within database storage.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::SessionConfig;
    /// use chrono::Duration;
    ///
    /// let config = SessionConfig::default().with_max_lifetime(Duration::days(32));
    /// ```
    ///
    #[must_use]
    pub fn with_max_lifetime(mut self, time: Duration) -> Self {
        self.max_lifespan = time;
        self
    }

    /// Set's the session's lifetime (expiration time) within memory storage.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::SessionConfig;
    /// use chrono::Duration;
    ///
    /// let config = SessionConfig::default().with_memory_lifetime(Duration::days(32));
    /// ```
    ///
    #[must_use]
    pub fn with_memory_lifetime(mut self, time: Duration) -> Self {
        self.memory_lifespan = time;
        self
    }

    /// This value represents the offset duration for how often session data gets updated in
    /// the database regardless of getting changed or not.
    /// This is leftover_expiration_duration <= expiration_update.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::SessionConfig;
    /// use chrono::Duration;
    ///
    /// let config = SessionConfig::default().with_expiration_update(Duration::days(320));
    /// ```
    ///
    #[must_use]
    pub fn with_expiration_update(mut self, duration: Duration) -> Self {
        self.expiration_update = duration;
        self
    }

    /// This value represents if the database should check for updates to save or
    /// to just save the data regardless of updates. When set to true it will disable the
    /// update checks.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::SessionConfig;
    /// use chrono::Duration;
    ///
    /// let config = SessionConfig::default().with_expiration_update(Duration::days(320));
    /// ```
    ///
    #[must_use]
    pub fn with_always_save(mut self, always_save: bool) -> Self {
        self.always_save = always_save;
        self
    }

    /// Set's the session's secure flag for if it gets sent over https.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::SessionConfig;
    ///
    /// let config = SessionConfig::default().with_secure(true);
    /// ```
    ///
    #[must_use]
    pub fn with_secure(mut self, is_set: bool) -> Self {
        self.cookie_secure = is_set;
        self
    }

    /// Set's the session's database table name.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::SessionConfig;
    ///
    /// let config = SessionConfig::default().with_table_name("my_table");
    /// ```
    ///
    #[must_use]
    pub fn with_table_name(mut self, table_name: impl Into<Cow<'static, str>>) -> Self {
        self.table_name = table_name.into();
        self
    }

    /// Set's the session's cookie encyption key enabling private cookies.
    ///
    /// When Set it will enforce Private cookies across all Sessions.
    /// If you use Key::generate() it will make a new key each server reboot.
    /// To prevent this make and save a key to a config file for long term usage.
    /// For Extra Security Regenerate the key every so many months to a year.
    /// A new key will invalidate all old Sessions so it be wise to run session_store.clear_store() on reboot.
    ///
    /// Must be Set to Some() in order to use Security::PerSession.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::{Key, SessionConfig};
    ///
    /// let config = SessionConfig::default().with_key(Key::generate());
    /// ```
    ///
    #[must_use]
    pub fn with_key(mut self, key: Key) -> Self {
        self.key = Some(key);
        self
    }

    /// Set's the session's database encyption key for per session key storage.
    ///
    /// Must be Set to Some() in order to use Security::PerSession or will panic if not.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::{Key, SessionConfig};
    ///
    /// let config = SessionConfig::default().with_key(Key::generate());
    /// ```
    ///
    #[must_use]
    pub fn with_database_key(mut self, key: Key) -> Self {
        self.database_key = Some(key);
        self
    }

    /// Set's the session's security mode.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::{SecurityMode, SessionConfig};
    ///
    /// let config = SessionConfig::default().with_security_mode(SecurityMode::PerSession);
    /// ```
    ///
    #[must_use]
    pub fn with_security_mode(mut self, mode: SecurityMode) -> Self {
        self.security_mode = mode;
        self
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            /// Set to a 6 hour default in Database Session stores unloading.
            lifespan: Duration::hours(6),
            cookie_name: "sqlx_session".into(),
            cookie_path: "/".into(),
            cookie_max_age: Some(Duration::days(100)),
            cookie_http_only: true,
            cookie_secure: false,
            cookie_domain: None,
            cookie_same_site: SameSite::Lax,
            storable_cookie_name: "session_acceptance".into(),
            table_name: "async_sessions".into(),
            /// Unload memory after 60 minutes if it has not been accessed.
            memory_lifespan: Duration::minutes(60),
            /// Unload long term session after 60 days if it has not been accessed.
            max_lifespan: Duration::days(60),
            /// Default to update the database every hour if the session is still being requested.
            expiration_update: Duration::hours(5),
            always_save: false,
            session_mode: SessionMode::Always,
            /// Key is set to None so Private cookies are not used by default. Please set this if you want to use private cookies.
            key: None,
            /// Database key is set to None it will panic if you attempt to use SecurityMode::PerSession.
            database_key: None,
            /// Default cookie name for the Key Id.
            key_cookie_name: "session_key".into(),
            /// Simple is the Default mode for compatibilty with older versions of the crate.
            security_mode: SecurityMode::Simple,
        }
    }
}
