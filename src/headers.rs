#[cfg(not(feature = "rest_mode"))]
use crate::CookiesAdditionJar;
use crate::{DatabasePool, Session, SessionConfig, SessionStore};
#[cfg(not(feature = "rest_mode"))]
use cookie::{Cookie, CookieJar, Key};
use forwarded_header_value::{ForwardedHeaderValue, Identifier};
#[cfg(not(feature = "rest_mode"))]
use http::header::{COOKIE, SET_COOKIE};
use http::{
    self,
    header::{FORWARDED, USER_AGENT},
    request::Request,
    HeaderMap,
};
#[cfg(feature = "rest_mode")]
use http::{header::HeaderName, HeaderValue};
#[cfg(feature = "rest_mode")]
use std::collections::HashMap;
use std::{
    fmt::Debug,
    marker::{Send, Sync},
    net::{IpAddr, SocketAddr},
};
use uuid::Uuid;

const X_REAL_IP: &str = "x-real-ip";
const X_FORWARDED_FOR: &str = "x-forwarded-for";

enum NameType {
    Store,
    Data,
}

impl NameType {
    #[inline]
    pub(crate) fn get_name(&self, config: &SessionConfig) -> String {
        let name = match self {
            NameType::Data => config.cookie_and_header.session_name.to_string(),
            NameType::Store => config.cookie_and_header.store_name.to_string(),
        };

        #[cfg(not(feature = "rest_mode"))]
        if config.cookie_and_header.prefix_with_host {
            let mut prefixed = "__Host-".to_owned();
            prefixed.push_str(&name);
            prefixed
        } else {
            name
        }

        #[cfg(feature = "rest_mode")]
        name
    }
}

#[cfg(not(feature = "rest_mode"))]
pub async fn get_headers_and_key<T>(
    store: &SessionStore<T>,
    cookies: CookieJar,
    ip_user_agent: &str,
) -> (Option<Uuid>, bool)
where
    T: DatabasePool + Clone + Debug + Sync + Send + 'static,
{
    let key = store.config.cookie_and_header.key.as_ref();

    let value = cookies
        .get_cookie(
            &store.config.cookie_and_header.session_name,
            key,
            ip_user_agent.to_owned(),
            false,
        )
        .and_then(|c| Uuid::parse_str(c.value()).ok());

    let storable = cookies
        .get_cookie(
            &store.config.cookie_and_header.store_name,
            key,
            ip_user_agent.to_owned(),
            true,
        )
        .map_or(false, |c| c.value().parse().unwrap_or(false));

    (value, storable)
}

#[cfg(feature = "rest_mode")]
pub async fn get_headers_and_key<T>(
    store: &SessionStore<T>,
    headers: HashMap<String, String>,
    ip_user_agent: &str,
) -> (Option<Uuid>, bool)
where
    T: DatabasePool + Clone + Debug + Sync + Send + 'static,
{
    use crate::sec::verify_header;
    let key = store.config.cookie_and_header.key.as_ref();

    let name = store.config.cookie_and_header.session_name.to_string();
    let value = headers
        .get(&name)
        .and_then(|c| {
            if let Some(key) = key {
                verify_header(c, key, ip_user_agent).ok()
            } else {
                Some(c.to_owned())
            }
        })
        .and_then(|c| Uuid::parse_str(&c).ok());

    let name = store.config.cookie_and_header.store_name.to_string();
    let storable = headers
        .get(&name)
        .and_then(|c| {
            if let Some(key) = key {
                verify_header(c, key, ip_user_agent).ok()
            } else {
                Some(c.to_owned())
            }
        })
        .map(|c| c.parse().unwrap_or(false));

    (value, storable.unwrap_or(false))
}

#[cfg(not(feature = "rest_mode"))]
pub(crate) trait CookiesExt {
    fn get_cookie(
        &self,
        name: &str,
        key: Option<&Key>,
        message: String,
        bypass: bool,
    ) -> Option<Cookie<'static>>;
    fn add_cookie(
        &mut self,
        cookie: Cookie<'static>,
        key: &Option<Key>,
        message: String,
        bypass: bool,
    );
}

#[cfg(not(feature = "rest_mode"))]
impl CookiesExt for CookieJar {
    fn get_cookie(
        &self,
        name: &str,
        key: Option<&Key>,
        message: String,
        bypass: bool,
    ) -> Option<Cookie<'static>> {
        if !bypass {
            if let Some(key) = key {
                return self.message_signed(key, message).get(name);
            }
        }

        self.get(name).cloned()
    }

    fn add_cookie(
        &mut self,
        cookie: Cookie<'static>,
        key: &Option<Key>,
        message: String,
        bypass: bool,
    ) {
        if !bypass {
            if let Some(key) = key {
                self.message_signed_mut(key, message).add(cookie);
                return;
            }
        }

        self.add(cookie);
    }
}

#[cfg(not(feature = "rest_mode"))]
fn create_cookie<'a>(config: &SessionConfig, value: String, cookie_type: NameType) -> Cookie<'a> {
    let mut cookie_builder = Cookie::build((cookie_type.get_name(config), value))
        .path(config.cookie_and_header.cookie_path.clone())
        .secure(config.cookie_and_header.cookie_secure)
        .http_only(config.cookie_and_header.cookie_http_only)
        .same_site(config.cookie_and_header.cookie_same_site);

    if let Some(domain) = &config.cookie_and_header.cookie_domain {
        cookie_builder = cookie_builder.domain(domain.clone());
    }

    if let Some(max_age) = config.cookie_and_header.cookie_max_age {
        let time_duration = max_age.to_std().expect("Max Age out of bounds");
        cookie_builder =
            cookie_builder.expires(Some((std::time::SystemTime::now() + time_duration).into()));
    }

    cookie_builder.build()
}

#[cfg(not(feature = "rest_mode"))]
fn remove_cookie<'a>(config: &SessionConfig, cookie_type: NameType) -> Cookie<'a> {
    let mut cookie_builder = Cookie::build((cookie_type.get_name(config), ""))
        .path(config.cookie_and_header.cookie_path.clone())
        .http_only(config.cookie_and_header.cookie_http_only)
        .same_site(cookie::SameSite::None);

    if let Some(domain) = &config.cookie_and_header.cookie_domain {
        cookie_builder = cookie_builder.domain(domain.clone());
    }

    if let Some(domain) = &config.cookie_and_header.cookie_domain {
        cookie_builder = cookie_builder.domain(domain.clone());
    }

    let mut cookie = cookie_builder.build();
    cookie.make_removal();
    cookie
}

#[cfg(not(feature = "rest_mode"))]
/// This will get a CookieJar from the Headers.
pub(crate) fn get_cookies(headers: &HeaderMap) -> CookieJar {
    let mut jar = CookieJar::new();

    let cookie_iter = headers
        .get_all(COOKIE)
        .into_iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|cookie| Cookie::parse_encoded(cookie.to_owned()).ok());

    for cookie in cookie_iter {
        jar.add_original(cookie);
    }

    jar
}

#[cfg(feature = "rest_mode")]
/// This will get a Hashmap of all the headers that Exist.
pub(crate) fn get_headers<T>(
    store: &SessionStore<T>,
    headers: &HeaderMap,
) -> HashMap<String, String>
where
    T: DatabasePool + Clone + Debug + Sync + Send + 'static,
{
    let mut map = HashMap::new();

    for name in [
        store.config.cookie_and_header.session_name.to_string(),
        store.config.cookie_and_header.store_name.to_string(),
    ] {
        if let Some(value) = headers.get(&name) {
            if let Ok(val) = value.to_str() {
                map.insert(name, val.to_owned());
            }
        }
    }

    map
}

#[cfg(not(feature = "rest_mode"))]
fn set_cookies(jar: CookieJar, headers: &mut HeaderMap) {
    for cookie in jar.delta() {
        if let Ok(header_value) = cookie.encoded().to_string().parse() {
            headers.append(SET_COOKIE, header_value);
        }
    }
}

/// Used to Set either the Header Values or the Cookie Values.
pub(crate) fn set_headers<T>(
    session: &Session<T>,
    headers: &mut HeaderMap,
    ip_user_agent: &str,
    destroy: bool,
    storable: bool,
) where
    T: DatabasePool + Clone + Debug + Sync + Send + 'static,
{
    // Lets make a new jar as we only want to add our cookies to the Response cookie header.\
    #[cfg(not(feature = "rest_mode"))]
    {
        let mut cookies = CookieJar::new();

        // Add SessionID
        if (storable || !session.store.config.session_mode.is_opt_in()) && !destroy {
            cookies.add_cookie(
                create_cookie(&session.store.config, session.id.inner(), NameType::Data),
                &session.store.config.cookie_and_header.key,
                ip_user_agent.to_owned(),
                false,
            );
        } else {
            cookies.add_cookie(
                remove_cookie(&session.store.config, NameType::Data),
                &session.store.config.cookie_and_header.key,
                ip_user_agent.to_owned(),
                false,
            );
        }

        // Add Session Store Boolean
        if session.store.config.session_mode.is_opt_in() && storable && !destroy {
            cookies.add_cookie(
                create_cookie(&session.store.config, storable.to_string(), NameType::Store),
                &session.store.config.cookie_and_header.key,
                ip_user_agent.to_owned(),
                true,
            );
        } else {
            cookies.add_cookie(
                remove_cookie(&session.store.config, NameType::Store),
                &session.store.config.cookie_and_header.key,
                ip_user_agent.to_owned(),
                true,
            );
        }

        set_cookies(cookies, headers);
    }
    #[cfg(feature = "rest_mode")]
    {
        use crate::sec::sign_header;
        // Add SessionID
        if (storable || !session.store.config.session_mode.is_opt_in()) && !destroy {
            let name = NameType::Data.get_name(&session.store.config);
            let value = if let Some(key) = session.store.config.cookie_and_header.key.as_ref() {
                match sign_header(&session.id.inner(), key, ip_user_agent) {
                    Ok(v) => v,
                    Err(err) => {
                        tracing::error!(err = %err, "Failed to sign Session ID so blank will be used.");
                        String::new()
                    }
                }
            } else {
                session.id.inner()
            };

            if let Ok(name) = HeaderName::from_bytes(name.as_bytes()) {
                if let Ok(value) = HeaderValue::from_str(&value) {
                    headers.insert(name, value);
                }
            }
        }

        // Add Session Store Boolean
        if session.store.config.session_mode.is_opt_in() && storable && !destroy {
            let name = NameType::Store.get_name(&session.store.config);
            //storable doesnt need signing or encryption.
            let value = storable.to_string();

            if let Ok(name) = HeaderName::from_bytes(name.as_bytes()) {
                if let Ok(value) = HeaderValue::from_str(&value) {
                    headers.insert(name, value);
                }
            }
        }
    }
}

///Rather than getting a single IP from the x_real, X forwarded and socket ip
///It is better to use all 3 to ensure none of them have changed. Setting the default
/// to be a empty String if it is not present. we will combine these together in a single Message String.
pub(crate) fn get_ips_hash<T, D>(req: &Request<T>, store: &SessionStore<D>) -> String
where
    D: DatabasePool + Clone + Debug + Sync + Send + 'static,
{
    if store.config.cookie_and_header.key.is_some()
        && store.config.cookie_and_header.with_ip_and_user_agent
    {
        let headers = req.headers();

        let ip = req
            .extensions()
            .get::<axum::extract::ConnectInfo<SocketAddr>>()
            .map(|addr| addr.ip().to_string())
            .unwrap_or_default();

        let x_forward_for_ip = headers
            .get(X_FORWARDED_FOR)
            .and_then(|hv| hv.to_str().ok())
            .and_then(|s| s.split(',').find_map(|s| s.trim().parse::<IpAddr>().ok()))
            .map(|ip| ip.to_string())
            .unwrap_or_default();

        let forwarded_ip = headers
            .get_all(FORWARDED)
            .iter()
            .find_map(|hv| {
                hv.to_str()
                    .ok()
                    .and_then(|s| ForwardedHeaderValue::from_forwarded(s).ok())
                    .and_then(|f| {
                        f.iter()
                            .filter_map(|fs| fs.forwarded_for.as_ref())
                            .find_map(|ff| match ff {
                                Identifier::SocketAddr(a) => Some(a.ip()),
                                Identifier::IpAddr(ip) => Some(*ip),
                                _ => None,
                            })
                    })
            })
            .map(|ip| ip.to_string())
            .unwrap_or_default();

        let real_ip = headers
            .get(X_REAL_IP)
            .and_then(|hv| hv.to_str().ok())
            .and_then(|s| s.parse::<IpAddr>().ok())
            .map(|ip| ip.to_string())
            .unwrap_or_default();

        let user_agent = headers
            .get(USER_AGENT)
            .and_then(|hv| hv.to_str().ok())
            .map(|useragent| useragent.to_string())
            .unwrap_or_default();

        format!(
            "{};{};{};{};{}",
            ip, x_forward_for_ip, forwarded_ip, real_ip, user_agent
        )
    } else {
        String::new()
    }
}
