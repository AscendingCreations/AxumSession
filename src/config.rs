use chrono::Duration;
pub use cookie::{Key, SameSite};
use std::borrow::Cow;

/// Mode at which the Session will function As.
///
/// # Examples
/// ```rust
/// use axum_session::{SessionConfig, SessionMode};
///
/// let config = SessionConfig::default().with_mode(SessionMode::Persistent);
/// ```
///
#[derive(Debug, Clone)]
pub enum SessionMode {
    /// Creates a SessionID Without SessionData.The End user must run `session.create_data()`
    /// to create the SessionData.
    /// Functions will emit a Info warning and do nothing until Created.
    /// Deletes SessionData and cookie if `session.store` is false.
    /// if SessionData.store is true retains SessionData and Syncs with Database.
    /// You can Set SessionData.store to true using 'session.set_store(true)'
    Manual,
    /// Always Creates a Session.
    /// Deletes SessionData and cookie if `session.store` is false.
    /// if SessionData.store is true retains SessionData and Syncs with Database.
    /// You can Set SessionData.store to true using 'session.set_store(true)'
    OptIn,
    /// Always Creates a Session
    /// Always retains in Memory and syncs with Database.
    Persistent,
}

impl SessionMode {
    /// Checks if the Mode is set to only if OptIn or Manual.
    ///
    pub fn is_opt_in(&self) -> bool {
        matches!(self, SessionMode::OptIn | SessionMode::Manual)
    }
    /// Checks if the user needs to manually create the SessionData per user.
    /// When created the Session will get Set to loaded.
    pub fn is_manual(&self) -> bool {
        matches!(self, SessionMode::Manual)
    }
}

#[derive(Clone)]
pub struct CookieAndHeaderConfig {
    /// The Cookie or Header name that contains a boolean for session saving.
    /// only used when session_mode is set to SessionMode::OptIn or Manual.
    pub(crate) store_name: Cow<'static, str>,
    /// Session Cookie or Header name.
    pub(crate) session_name: Cow<'static, str>,
    /// Session cookie domain.
    pub(crate) cookie_domain: Option<Cow<'static, str>>,
    /// Session cookie http only flag.
    pub(crate) cookie_http_only: bool,
    /// Session cookie max age None means the browser deletes cookie on close.
    /// Please make sure the Duration is longer than max_lifespan.
    pub(crate) cookie_max_age: Option<Duration>,
    /// Session cookie path.
    pub(crate) cookie_path: Cow<'static, str>,
    /// Resticts how Cookies are sent cross-site. Default is `SameSite::Lax`.
    pub(crate) cookie_same_site: SameSite,
    /// Session cookie secure flag.
    pub(crate) cookie_secure: bool,
    /// Encyption Key used to sign cookies and header for integrity, and authenticity.
    pub(crate) key: Option<Key>,
    /// This is used to append __Host- to the front of all Cookie names to prevent sub domain usage.
    /// This will not append to Headers only Cookies. It is enabled by default.
    pub(crate) prefix_with_host: bool,
}

impl std::fmt::Debug for CookieAndHeaderConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CookieAndHeaderConfig")
            .field("store_name", &self.store_name)
            .field("cookie_domain", &self.cookie_domain)
            .field("cookie_http_only", &self.cookie_http_only)
            .field("cookie_max_age", &self.cookie_max_age)
            .field("session_name", &self.session_name)
            .field("cookie_path", &self.cookie_path)
            .field("cookie_same_site", &self.cookie_same_site)
            .field("cookie_secure", &self.cookie_secure)
            .field("prefix_with_host", &self.prefix_with_host)
            .field("key", &"key hidden")
            .finish()
    }
}

#[derive(Clone)]
pub struct DatabaseConfig {
    /// Encyption Key used to encypt Session data stored in the database for confidentiality.
    pub(crate) database_key: Option<Key>,
    /// Session Database table name default is sessions.
    pub(crate) table_name: Cow<'static, str>,
    /// This value represents the duration for how often session's data gets purged from the database per request.
    pub(crate) purge_database_update: Duration,
    /// Ignore's the update checks and will always save the session to the database if set to true.
    pub(crate) always_save: bool,
}

impl std::fmt::Debug for DatabaseConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DatabaseConfig")
            .field("table_name", &self.table_name)
            .field("purge_database_update", &self.purge_database_update)
            .field("always_save", &self.always_save)
            .field("database_key", &"key hidden")
            .finish()
    }
}

#[derive(Clone)]
pub struct MemoryConfig {
    /// This value represents the duration for how often session's data gets purged from memory per request.
    pub(crate) purge_update: Duration,
    /// Session Memory lifespan, deturmines when to unload it from memory
    /// this works fine since the data can stay in the database till its needed
    /// if not yet expired.
    pub(crate) memory_lifespan: Duration,
    /// How many Elements could we see at one time in the Table?
    /// So if you have 1000 unique visitors a second and each generate a UUID.
    /// That would be 1000 * 60(secs) * 60(mins) * 24(hours) to get 1 days worth of visitors.
    pub(crate) filter_expected_elements: u64,
    /// The probability of how many allowable false positives you want to have based on the expected elements.
    /// 0.01 is a good starting point.
    pub(crate) filter_false_positive_probability: f64,
    /// This enabled using a counting bloom filter. If this is taking to much Memory or is to slow or you just dont want
    /// the false positives it can give you can disable it by setting it to false. This will reduce memory usage.
    /// By default this is enabled unless the specific database cant function with it then disabled.
    pub(crate) use_bloom_filters: bool,
}

impl std::fmt::Debug for MemoryConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryConfig")
            .field("memory_lifespan", &self.memory_lifespan)
            .field("filter_expected_elements", &self.filter_expected_elements)
            .field("use_bloom_filters", &self.use_bloom_filters)
            .field("purge_update", &self.purge_update)
            .field(
                "filter_false_positive_probability",
                &self.filter_false_positive_probability,
            )
            .finish()
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
    /// Disables the need to avoid session saving.
    pub(crate) session_mode: SessionMode,
    /// Minimal lifespan of database store and cookie before expiring.
    /// This is set to the Cookie before sending and to the database before updating/inserting.
    pub(crate) lifespan: Duration,
    /// Maximum lifespan of database store and cookie before expiring.
    /// This is set to the Cookie before sending and to the database before updating/inserting.
    /// Only Set when Long Term is true.
    pub(crate) max_lifespan: Duration,
    /// This is to be used when your handling multiple Parallel Sessions to prevent the next one from unloaded data.
    pub(crate) clear_check_on_load: bool,
    /// where All Database Storage options exist.
    pub(crate) database_config: DatabaseConfig,
    /// where All In Memory Storage options exist.
    pub(crate) memory_config: MemoryConfig,
    /// where All the cookie and header Options Exist.
    pub(crate) cookie_and_header_config: CookieAndHeaderConfig,
}

impl std::fmt::Debug for SessionConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionConfig")
            .field("database_config", &self.database_config)
            .field("memory_config", &self.memory_config)
            .field("cookie_and_header_config", &self.cookie_and_header_config)
            .field("session_mode", &self.session_mode)
            .field("lifespan", &self.lifespan)
            .field("max_lifespan", &self.max_lifespan)
            .field("clear_check_on_load", &self.clear_check_on_load)
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

    /// Set the session's store Cookie or Header name.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::SessionConfig;
    ///
    /// let config = SessionConfig::default().with_store_name("my_stored_cookie".to_owned());
    /// ```
    ///
    #[must_use]
    pub fn with_store_name(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.cookie_and_header_config.store_name = name.into();
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
        self.cookie_and_header_config.cookie_domain = Some(name.into());
        self
    }

    /// Set's the session's Cookie or Header name.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::SessionConfig;
    ///
    /// let config = SessionConfig::default().with_session_name("my_cookie");
    /// ```
    ///
    #[must_use]
    pub fn with_session_name(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.cookie_and_header_config.session_name = name.into();
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
        self.cookie_and_header_config.cookie_path = path.into();
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
        self.cookie_and_header_config.cookie_same_site = same_site;
        self
    }

    /// Set's whether the session Persistantly stores data or on stores if storable.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::{SessionMode, SessionConfig};
    /// use cookie::SameSite;
    ///
    /// let config = SessionConfig::default().with_mode(SessionMode::Persistent);
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
        self.cookie_and_header_config.cookie_http_only = is_set;
        self
    }

    /// Set's the session's lifetime (expiration time) within database storage.
    /// This should be equal too or less than the Cookies Expiration time.
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
    /// Please Ensure the Duration is greater or equal to max_lifespan for proper storage.
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
        self.cookie_and_header_config.cookie_max_age = time;
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
    /// This setting should be Less than lifespan and max_lifespan. This is to
    /// Unload the data from memory and allow it to stay stored in the database.
    ///
    /// Set this to Duration::zero() if you dont want it to stay in memory.
    /// Warning: This will cause it to be loaded from the database each request.
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
        self.memory_config.memory_lifespan = time;
        self
    }

    /// This value represents the offset duration for how often session purge for memory is ran.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::SessionConfig;
    /// use chrono::Duration;
    ///
    /// let config = SessionConfig::default().with_purge_update(Duration::hours(1));
    /// ```
    ///
    #[must_use]
    pub fn with_purge_update(mut self, duration: Duration) -> Self {
        self.memory_config.purge_update = duration;
        self
    }

    /// This value represents the offset duration for how often session purge for database is ran.
    /// If using Redis or any auto purge database this Setting will be ignored.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::SessionConfig;
    /// use chrono::Duration;
    ///
    /// let config = SessionConfig::default().with_purge_database_update(Duration::hours(5));
    /// ```
    ///
    #[must_use]
    pub fn with_purge_database_update(mut self, duration: Duration) -> Self {
        self.database_config.purge_database_update = duration;
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
    /// let config = SessionConfig::default().with_always_save(true);
    /// ```
    ///
    #[must_use]
    pub fn with_always_save(mut self, always_save: bool) -> Self {
        self.database_config.always_save = always_save;
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
        self.cookie_and_header_config.cookie_secure = is_set;
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
        self.database_config.table_name = table_name.into();
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
        self.cookie_and_header_config.key = Some(key);
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
        self.database_config.database_key = Some(key);
        self
    }

    /// Set's the session's filters expected elements.
    /// Please Set this by a daily value.
    /// Example: 1000 * 60(secs) * 60(mins) * 24(hours) to get 1 days worth of visitors.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::SessionConfig;
    ///
    /// let config = SessionConfig::default().with_filter_expected_elements(100_000);
    /// ```
    ///
    #[must_use]
    pub fn with_filter_expected_elements(mut self, elements: u64) -> Self {
        self.memory_config.filter_expected_elements = elements;
        self
    }

    /// Set's the session's filters False Posistive probability when creating and comparing UUID.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::SessionConfig;
    ///
    /// let config = SessionConfig::default().with_filter_false_positive_probability(0.01);
    /// ```
    ///
    #[must_use]
    pub fn with_filter_false_positive_probability(mut self, probability: f64) -> Self {
        self.memory_config.filter_false_positive_probability = probability;
        self
    }

    /// Set's the session's bloom filters to be disabled or enabled. By default they are enabled.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::SessionConfig;
    ///
    /// let config = SessionConfig::default().with_bloom_filter(true);
    /// ```
    ///
    #[must_use]
    pub fn with_bloom_filter(mut self, enable: bool) -> Self {
        self.memory_config.use_bloom_filters = enable;
        self
    }

    /// Get's the session's Cookie/Header name
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::SessionConfig;
    ///
    /// let name = SessionConfig::default().get_session_name();
    /// ```
    ///
    pub fn get_session_name(&self) -> String {
        self.cookie_and_header_config.session_name.to_string()
    }

    /// Get's the session's store booleans Cookie/Header name
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::SessionConfig;
    ///
    /// let name = SessionConfig::default().get_store_name();
    /// ```
    ///
    pub fn get_store_name(&self) -> String {
        self.cookie_and_header_config.store_name.to_string()
    }

    /// Set's the session's loading to either true: unload data if checks fail or false: bypass.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::SessionConfig;
    ///
    /// let config = SessionConfig::default().with_bloom_filter(true);
    /// ```
    ///
    #[must_use]
    pub fn with_clear_check_on_load(mut self, enable: bool) -> Self {
        self.clear_check_on_load = enable;
        self
    }

    /// Set's the session's prefix_with_host to either true: __Host- gets prefixed to the cookie names false: __Host- does not get prepended.
    ///
    /// __Host- prefix: Cookies with names starting with __Host- must be set with the secure flag, must be from a secure page (HTTPS),
    /// must not have a domain specified (and therefore, are not sent to subdomains), and the path must be /.
    ///
    /// # Examples
    /// ```rust
    /// use axum_session::SessionConfig;
    ///
    /// let config = SessionConfig::default().with_prefix_with_host(true);
    /// ```
    ///
    #[must_use]
    pub fn with_prefix_with_host(mut self, enable: bool) -> Self {
        self.cookie_and_header_config.prefix_with_host = enable;
        self
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            // Set to a 6 hour default in Database Session stores unloading.
            lifespan: Duration::hours(6),
            cookie_and_header_config: CookieAndHeaderConfig::default(),
            database_config: DatabaseConfig::default(),
            memory_config: MemoryConfig::default(),
            // Unload long term session after 60 days if it has not been accessed.
            max_lifespan: Duration::days(60),
            session_mode: SessionMode::Persistent,
            clear_check_on_load: true,
        }
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            // Unload memory after 60 minutes if it has not been accessed.
            memory_lifespan: Duration::minutes(60),
            // Default to purge old sessions every 5 hours.
            purge_update: Duration::hours(1),
            // Simple is the Default mode for compatibilty with older versions of the crate.
            filter_expected_elements: 100_000,
            // The probability of how many allowable false positives you want to have based on the expected elements.
            // 0.01 is a good starting point.
            filter_false_positive_probability: 0.01,
            // Always set to on.
            use_bloom_filters: true,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            // Set to a 6 hour default in Database Session stores unloading.
            table_name: "sessions".into(),
            // Default to purge old sessions in the database every 5 hours per request.
            purge_database_update: Duration::hours(5),
            always_save: false,
            // Database key is set to None it will panic if you attempt to use SecurityMode::PerSession.
            database_key: None,
        }
    }
}

impl Default for CookieAndHeaderConfig {
    fn default() -> Self {
        Self {
            session_name: "session".into(),
            cookie_path: "/".into(),
            cookie_max_age: Some(Duration::days(100)),
            cookie_http_only: true,
            cookie_secure: false,
            cookie_domain: None,
            cookie_same_site: SameSite::Lax,
            store_name: "store".into(),
            // Key is set to None so Private cookies are not used by default. Please set this if you want to use private cookies.
            key: None,
            prefix_with_host: false,
        }
    }
}
