use chrono::Duration;
pub use cookie::SameSite;
use std::borrow::Cow;

/// Mode at which the Session will function As.
///
/// # Examples
/// ```rust
/// use axum_database_sessions::{AxumSessionConfig, AxumSessionMode};
///
/// let config = AxumSessionConfig::default().with_mode(AxumSessionMode::Always);
/// ```
///
#[derive(Debug, Clone)]
pub enum AxumSessionMode {
    /// Deletes Session Data if session.storable is false, if session.storable is true saves data.
    Storable,
    /// Always in Memory and Database. regardless of if storable.
    Always,
}

impl AxumSessionMode {
    /// Checks if the Mode is set to only if Storable.
    ///
    pub fn is_storable(&self) -> bool {
        matches!(self, AxumSessionMode::Storable)
    }
}

/// Configuration for how the Session and Cookies are used.
///
/// # Examples
/// ```rust
/// use axum_database_sessions::AxumSessionConfig;
///
/// let config = AxumSessionConfig::default();
/// ```
///
#[derive(Debug, Clone)]
pub struct AxumSessionConfig {
    /// The acepted cookies max age None means the browser deletes cookie on close
    pub(crate) storable_cookie_max_age: Option<Duration>,
    /// The cookie name that contains a boolean for session saving.
    pub(crate) storable_cookie_name: Cow<'static, str>,
    /// Session cookie domain
    pub(crate) cookie_domain: Option<Cow<'static, str>>,
    /// Session cookie http only flag
    pub(crate) cookie_http_only: bool,
    /// Session cookie max age None means the browser deletes cookie on close
    pub(crate) cookie_max_age: Option<Duration>,
    /// Session cookie name
    pub(crate) cookie_name: Cow<'static, str>,
    /// Session cookie path
    pub(crate) cookie_path: Cow<'static, str>,
    /// Resticts how Cookies are sent cross-site. Default is `SameSite::None`
    /// Only works if domain is also set.
    pub(crate) cookie_same_site: SameSite,
    /// Session cookie secure flag
    pub(crate) cookie_secure: bool,
    /// Disables the need to avoid session saving.
    pub(crate) session_mode: AxumSessionMode,
    /// Sessions lifespan within the Database.
    pub(crate) lifespan: Duration,
    /// Session Database Max Poll Connections. Can not be 0
    pub(crate) max_connections: u32,
    /// This is the long term lifespan for things like Remember Me.
    /// Deturmines Database unload.
    pub(crate) max_lifespan: Duration,
    /// Session Memory lifespan, deturmines when to unload it from memory
    /// this works fine since the data can stay in the database till its needed
    /// if not yet expired.
    pub(crate) memory_lifespan: Duration,
    /// Session Database table name default is async_sessions
    pub(crate) table_name: Cow<'static, str>,
}

impl AxumSessionConfig {
    /// Sets the sessions database pool's max connection's limit.
    ///
    /// # Examples
    /// ```rust
    /// use axum_database_sessions::AxumSessionConfig;
    ///
    /// let config = AxumSessionConfig::default().set_max_connections(5);
    /// ```
    ///
    #[must_use]
    pub fn set_max_connections(mut self, max: u32) -> Self {
        let max = std::cmp::max(max, 1);
        self.max_connections = max;
        self
    }

    /// Set the session's storable cookie name.
    ///
    /// # Examples
    /// ```rust
    /// use axum_database_sessions::AxumSessionConfig;
    ///
    /// let config = AxumSessionConfig::default().with_accepted_cookie_name("my_accepted_cookie");
    /// ```
    ///
    #[must_use]
    pub fn with_storable_cookie_name(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.storable_cookie_name = name.into();
        self
    }

    /// Set's the session's storable cookies max_age (expiration time).
    ///
    /// If this is set to None then the storable Cookie will be unloaded on browser Close.
    /// Set this to be the duration of max_lifespan or longer to prevent session drops.
    ///
    /// # Examples
    /// ```rust
    /// use axum_database_sessions::AxumSessionConfig;
    /// use chrono::Duration;
    ///
    /// let config = AxumSessionConfig::default().with_accepted_max_age(Some(Duration::days(64)));
    /// ```
    ///
    #[must_use]
    pub fn with_storable_max_age(mut self, time: Option<Duration>) -> Self {
        self.storable_cookie_max_age = time;
        self
    }

    /// Set's the session's cookie's domain name.
    ///
    /// # Examples
    /// ```rust
    /// use axum_database_sessions::AxumSessionConfig;
    ///
    /// let config = AxumSessionConfig::default().with_cookie_domain(Some("www.helpme.com".to_string()));
    /// ```
    ///
    #[must_use]
    pub fn with_cookie_domain(mut self, name: impl Into<Option<Cow<'static, str>>>) -> Self {
        self.cookie_domain = name.into();
        self
    }

    /// Set's the session's cookie's name.
    ///
    /// # Examples
    /// ```rust
    /// use axum_database_sessions::AxumSessionConfig;
    ///
    /// let config = AxumSessionConfig::default().with_cookie_name("my_cookie");
    /// ```
    ///
    #[must_use]
    pub fn with_cookie_name(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.cookie_name = name.into();
        self
    }

    /// Set's the session's cookie's path.
    ///
    /// This is used to deturmine when the cookie takes effect within the website path.
    /// Leave as default ("/") for cookie to be used site wide.
    ///
    /// # Examples
    /// ```rust
    /// use axum_database_sessions::AxumSessionConfig;
    ///
    /// let config = AxumSessionConfig::default().with_cookie_path("/");
    /// ```
    ///
    #[must_use]
    pub fn with_cookie_path(mut self, path: impl Into<Cow<'static, str>>) -> Self {
        self.cookie_path = path.into();
        self
    }

    /// Set's the session's cookie's Same Site Setting for Cross-Site restrictions.
    ///
    /// Only works if Domain is also set to restrict it to that domain only.
    ///
    /// # Examples
    /// ```rust
    /// use axum_database_sessions::AxumSessionConfig;
    /// use cookie::SameSite;
    ///
    /// let config = AxumSessionConfig::default().with_cookie_same_site(SameSite::Strict);
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
    /// use axum_database_sessions::AxumSessionConfig;
    /// use cookie::SameSite;
    ///
    /// let config = AxumSessionConfig::default().with_mode(AxumSessionMode::Always);
    /// ```
    ///
    #[must_use]
    pub fn with_mode(mut self, mode: AxumSessionMode) -> Self {
        self.session_mode = mode;
        self
    }

    /// Set's the session's cookie's to http only.
    ///
    /// # Examples
    /// ```rust
    /// use axum_database_sessions::AxumSessionConfig;
    ///
    /// let config = AxumSessionConfig::default().with_http_only(false);
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
    /// use axum_database_sessions::AxumSessionConfig;
    /// use chrono::Duration;
    ///
    /// let config = AxumSessionConfig::default().with_lifetime(Duration::days(32));
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
    ///
    /// # Examples
    /// ```rust
    /// use axum_database_sessions::AxumSessionConfig;
    /// use chrono::Duration;
    ///
    /// let config = AxumSessionConfig::default().with_max_age(Some(Duration::days(64)));
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
    /// use axum_database_sessions::AxumSessionConfig;
    /// use chrono::Duration;
    ///
    /// let config = AxumSessionConfig::default().with_max_lifetime(Duration::days(32));
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
    /// use axum_database_sessions::AxumSessionConfig;
    /// use chrono::Duration;
    ///
    /// let config = AxumSessionConfig::default().with_memory_lifetime(Duration::days(32));
    /// ```
    ///
    #[must_use]
    pub fn with_memory_lifetime(mut self, time: Duration) -> Self {
        self.memory_lifespan = time;
        self
    }

    /// Set's the session's secure flag for if it gets sent over https.
    ///
    /// # Examples
    /// ```rust
    /// use axum_database_sessions::AxumSessionConfig;
    ///
    /// let config = AxumSessionConfig::default().with_secure(true);
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
    /// use axum_database_sessions::AxumSessionConfig;
    ///
    /// let config = AxumSessionConfig::default().with_table_name("my_table");
    /// ```
    ///
    #[must_use]
    pub fn with_table_name(mut self, table_name: impl Into<Cow<'static, str>>) -> Self {
        self.table_name = table_name.into();
        self
    }
}

impl Default for AxumSessionConfig {
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
            cookie_same_site: SameSite::None,
            storable_cookie_name: "session_acceptance".into(),
            storable_cookie_max_age: Some(Duration::days(100)),
            table_name: "async_sessions".into(),
            max_connections: 5,
            /// Unload memory after 60 minutes if it has not been accessed.
            memory_lifespan: Duration::minutes(60),
            /// Unload long term session after 60 days if it has not been accessed.
            max_lifespan: Duration::days(60),
            session_mode: AxumSessionMode::Always,
        }
    }
}
