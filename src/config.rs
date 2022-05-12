use chrono::Duration;
pub use cookie::SameSite;

///This is the Sessions Config it is used to Setup the SQL database and sets the hashmap saved Memory and Session life spans.
#[derive(Debug, Clone)]
pub struct AxumSessionConfig {
    /// Sessions lifespan
    pub(crate) lifespan: Duration,
    /// Session cookie name
    pub(crate) cookie_name: String,
    /// Session cookie path
    pub(crate) cookie_path: String,
    /// Session ID character length
    pub(crate) cookie_len: usize,
    /// Session cookie max age None means the browser deletes cookie on close
    pub(crate) cookie_max_age: Option<Duration>,
    /// Session cookie http only flag
    pub(crate) cookie_http_only: bool,
    /// Session cookie secure flag
    pub(crate) cookie_secure: bool,
    /// Session cookie domain
    pub(crate) cookie_domain: Option<String>,
    /// Resticts how Cookies are sent cross-site. Default is `SameSite::None`
    /// Only works if domain is also set.
    pub(crate) cookie_same_site: SameSite,
    /// Session Database table name default is async_sessions
    pub(crate) table_name: String,
    /// Session Database Max Poll Connections. Can not be 0
    pub(crate) max_connections: u32,
    /// Session Memory lifespan, deturmines when to unload it from memory
    /// this works fine since the data can stay in the database till its needed
    /// if not yet expired.
    pub(crate) memory_lifespan: Duration,
    /// This is the long term lifespan for things like Remember Me.
    pub(crate) max_lifespan: Duration,
}

impl AxumSessionConfig {
    /// Set session database pools max connections limit.
    #[must_use]
    pub fn set_max_connections(mut self, max: u32) -> Self {
        let max = std::cmp::max(max, 1);
        self.max_connections = max;
        self
    }

    /// Set session lifetime (expiration time) within database storage.
    #[must_use]
    pub fn with_lifetime(mut self, time: Duration) -> Self {
        self.lifespan = time;
        self
    }

    /// Set session's long term lifetime (expiration time) within database storage.
    #[must_use]
    pub fn with_max_lifetime(mut self, time: Duration) -> Self {
        self.max_lifespan = time;
        self
    }

    /// Set session lifetime (expiration time) within Memory storage.
    #[must_use]
    pub fn with_memory_lifetime(mut self, time: Duration) -> Self {
        self.memory_lifespan = time;
        self
    }

    /// Set session cookie max_age (expiration time) in browser.
    /// Set this to be the duration of max_lifespan or longer.
    #[must_use]
    pub fn with_max_age(mut self, time: Option<Duration>) -> Self {
        self.cookie_max_age = time;
        self
    }

    /// Set session cookie http_only flag.
    /// If set javascript has no access to the cookie.
    #[must_use]
    pub fn with_http_only(mut self, is_set: bool) -> Self {
        self.cookie_http_only = is_set;
        self
    }

    /// Set session cookie secure flag.
    /// If set the cookie will only be sent over https.
    #[must_use]
    pub fn with_secure(mut self, is_set: bool) -> Self {
        self.cookie_secure = is_set;
        self
    }

    /// Set session cookie domain name
    #[must_use]
    pub fn with_cookie_domain(mut self, name: Option<String>) -> Self {
        self.cookie_domain = name;
        self
    }

    /// Set session cookie Same Site Setting for Cross-Site restrictions
    /// Only works if Domain is also set.
    #[must_use]
    pub fn with_cookie_same_site(mut self, same_site: SameSite) -> Self {
        self.cookie_same_site = same_site;
        self
    }

    /// Set session cookie name
    #[must_use]
    pub fn with_cookie_name(mut self, name: &str) -> Self {
        self.cookie_name = name.into();
        self
    }

    /// Set session cookie length
    #[must_use]
    pub fn with_cookie_len(mut self, length: usize) -> Self {
        self.cookie_len = length;
        self
    }

    /// Set session cookie path
    #[must_use]
    pub fn with_cookie_path(mut self, path: &str) -> Self {
        self.cookie_path = path.into();
        self
    }

    /// Set session database table name
    #[must_use]
    pub fn with_table_name(mut self, table_name: &str) -> Self {
        self.table_name = table_name.into();
        self
    }
}

impl Default for AxumSessionConfig {
    fn default() -> Self {
        Self {
            /// Set to 6hour for default in Database Session stores.
            lifespan: Duration::hours(6),
            cookie_name: "sqlx_session".into(),
            cookie_path: "/".into(),
            cookie_len: 16,
            cookie_max_age: None,
            cookie_http_only: true,
            cookie_secure: false,
            cookie_domain: None,
            cookie_same_site: SameSite::None,
            table_name: "async_sessions".into(),
            max_connections: 5,
            /// Unload memory after 60mins if it has not been accessed.
            memory_lifespan: Duration::minutes(60),
            /// Unload session after 60days if it has not been accessed.
            max_lifespan: Duration::days(60),
        }
    }
}
