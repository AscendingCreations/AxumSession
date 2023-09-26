use crate::{
    config::SecurityMode, DatabasePool, Session, SessionConfig, SessionError, SessionKey,
    SessionStore,
};
use aes_gcm::aead::{generic_array::GenericArray, Aead, AeadInPlace, KeyInit, Payload};
use aes_gcm::Aes256Gcm;
use base64::{engine::general_purpose, Engine as _};
use cookie::{Cookie, CookieJar, Key};
use http::{
    self,
    header::{COOKIE, SET_COOKIE},
    HeaderMap,
};
use rand::RngCore;
use std::{
    collections::HashMap,
    fmt::Debug,
    marker::{Send, Sync},
};
use uuid::Uuid;

// Keep these in sync, and keep the key len synced with the `private` docs as
// well as the `KEYS_INFO` const in secure::Key. from cookie-rs
pub(crate) const NONCE_LEN: usize = 12;
pub(crate) const TAG_LEN: usize = 16;
pub(crate) const KEY_LEN: usize = 32;

enum CookieType {
    Storable,
    Data,
    Key,
}

impl CookieType {
    #[inline]
    pub(crate) fn get_name(&self, config: &SessionConfig) -> String {
        match self {
            CookieType::Data => config.cookie_name.to_string(),
            CookieType::Storable => config.storable_cookie_name.to_string(),
            CookieType::Key => config.key_cookie_name.to_string(),
        }
    }
}

#[cfg(not(feature = "rest_mode"))]
pub async fn get_headers_and_key<T>(
    store: &SessionStore<T>,
    cookies: CookieJar,
) -> (SessionKey, Option<Uuid>, bool)
where
    T: DatabasePool + Clone + Debug + Sync + Send + 'static,
{
    let value = cookies
        .get_cookie(&store.config.key_cookie_name, &store.config.key)
        .and_then(|c| Uuid::parse_str(c.value()).ok());

    let session_key = match store.config.security_mode {
        SecurityMode::PerSession => SessionKey::get_or_create(&store, value).await,
        SecurityMode::Simple => SessionKey::new(),
    };

    let key = match store.config.security_mode {
        SecurityMode::PerSession => Some(&session_key.key),
        SecurityMode::Simple => store.config.key.as_ref(),
    };

    let value = cookies
        .get_cookie(&store.config.cookie_name, &key)
        .and_then(|c| Uuid::parse_str(c.value()).ok());

    let storable = cookies
        .get_cookie(&store.config.storable_cookie_name, &key)
        .map_or(false, |c| c.value().parse().unwrap_or(false));

    (session_key, value, storable)
}

pub async fn get_headers_and_key<T>(
    store: &SessionStore<T>,
    headers: HashMap<String, String>,
) -> (SessionKey, Option<Uuid>, bool)
where
    T: DatabasePool + Clone + Debug + Sync + Send + 'static,
{
    let name = store.config.key_cookie_name.to_string();
    let value = headers
        .get(&name)
        .and_then(|c| {
            if let Some(key) = &store.config.key {
                decrypt(&name, c, key).ok()
            } else {
                Some(c.to_owned())
            }
        })
        .and_then(|c| Uuid::parse_str(&c).ok());

    let session_key = match store.config.security_mode {
        SecurityMode::PerSession => SessionKey::get_or_create(&store, value).await,
        SecurityMode::Simple => SessionKey::new(),
    };

    let key = match store.config.security_mode {
        SecurityMode::PerSession => Some(&session_key.key),
        SecurityMode::Simple => store.config.key.as_ref(),
    };

    let name = store.config.cookie_name.to_string();
    let value = headers
        .get(&name)
        .and_then(|c| {
            if let Some(key) = key {
                decrypt(&name, c, key).ok()
            } else {
                Some(c.to_owned())
            }
        })
        .and_then(|c| Uuid::parse_str(&c).ok());

    let name = store.config.storable_cookie_name.to_string();
    let storable = headers
        .get(&name)
        .and_then(|c| {
            if let Some(key) = key {
                decrypt(&name, c, key).ok()
            } else {
                Some(c.to_owned())
            }
        })
        .and_then(|c| Some(c.parse().unwrap_or(false)));

    (session_key, value, storable.unwrap_or(false))
}

pub(crate) trait CookiesExt {
    fn get_cookie(&self, name: &str, key: Option<&Key>) -> Option<Cookie<'static>>;
    fn add_cookie(&mut self, cookie: Cookie<'static>, key: &Option<Key>);
}

impl CookiesExt for CookieJar {
    fn get_cookie(&self, name: &str, key: Option<&Key>) -> Option<Cookie<'static>> {
        if let Some(key) = key {
            self.private(key).get(name)
        } else {
            self.get(name).cloned()
        }
    }

    fn add_cookie(&mut self, cookie: Cookie<'static>, key: &Option<Key>) {
        if let Some(key) = key {
            self.private_mut(key).add(cookie)
        } else {
            self.add(cookie)
        }
    }
}

fn create_cookie<'a>(config: &SessionConfig, value: String, cookie_type: CookieType) -> Cookie<'a> {
    let mut cookie_builder = Cookie::build(cookie_type.get_name(config), value)
        .path(config.cookie_path.clone())
        .secure(config.cookie_secure)
        .http_only(config.cookie_http_only)
        .same_site(config.cookie_same_site);

    if let Some(domain) = &config.cookie_domain {
        cookie_builder = cookie_builder.domain(domain.clone());
    }

    if let Some(max_age) = config.cookie_max_age {
        let time_duration = max_age.to_std().expect("Max Age out of bounds");
        cookie_builder =
            cookie_builder.expires(Some((std::time::SystemTime::now() + time_duration).into()));
    }

    cookie_builder.finish()
}

fn remove_cookie<'a>(config: &SessionConfig, cookie_type: CookieType) -> Cookie<'a> {
    let mut cookie_builder = Cookie::build(cookie_type.get_name(config), "")
        .path(config.cookie_path.clone())
        .http_only(config.cookie_http_only)
        .same_site(cookie::SameSite::None);

    if let Some(domain) = &config.cookie_domain {
        cookie_builder = cookie_builder.domain(domain.clone());
    }

    if let Some(domain) = &config.cookie_domain {
        cookie_builder = cookie_builder.domain(domain.clone());
    }

    let mut cookie = cookie_builder.finish();
    cookie.make_removal();
    cookie
}

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
        store.config.key_cookie_name.to_string(),
        store.config.cookie_name.to_string(),
        store.config.storable_cookie_name.to_string(),
    ] {
        if let Some(value) = headers.get(&name) {
            if let Some(val) = value.to_str().ok() {
                map.insert(name, val.to_owned());
            }
        }
    }

    map
}

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
    session_key: &SessionKey,
    headers: &mut HeaderMap,
    destroy: bool,
    storable: bool,
) where
    T: DatabasePool + Clone + Debug + Sync + Send + 'static,
{
    // Lets make a new jar as we only want to add our cookies to the Response cookie header.
    let mut cookies = CookieJar::new();

    // Add Per-Session encryption KeyID
    let cookie_key = match session.store.config.security_mode {
        SecurityMode::PerSession => {
            if (storable || !session.store.config.session_mode.is_storable()) && !destroy {
                cookies.add_cookie(
                    create_cookie(
                        &session.store.config,
                        session_key.id.inner(),
                        CookieType::Key,
                    ),
                    &session.store.config.key,
                );
            } else {
                //If not Storable we still remove the encryption key since there is no session.
                cookies.add_cookie(
                    remove_cookie(&session.store.config, CookieType::Key),
                    &session.store.config.key,
                );
            }

            Some(session_key.key.clone())
        }
        SecurityMode::Simple => {
            cookies.add_cookie(
                remove_cookie(&session.store.config, CookieType::Key),
                &session.store.config.key,
            );
            session.store.config.key.clone()
        }
    };

    // Add SessionID
    if (storable || !session.store.config.session_mode.is_storable()) && !destroy {
        cookies.add_cookie(
            create_cookie(&session.store.config, session.id.inner(), CookieType::Data),
            &cookie_key,
        );
    } else {
        cookies.add_cookie(
            remove_cookie(&session.store.config, CookieType::Data),
            &cookie_key,
        );
    }

    // Add Session Storable Boolean
    if session.store.config.session_mode.is_storable() && storable && !destroy {
        cookies.add_cookie(
            create_cookie(
                &session.store.config,
                storable.to_string(),
                CookieType::Storable,
            ),
            &cookie_key,
        );
    } else {
        cookies.add_cookie(
            remove_cookie(&session.store.config, CookieType::Storable),
            &cookie_key,
        );
    }

    set_cookies(cookies, headers);
}

///Used to encrypt the Header Values and key values
pub(crate) fn encrypt(name: &str, value: &str, key: &Key) -> String {
    let val = value.as_bytes();

    let mut data = vec![0; NONCE_LEN + val.len() + TAG_LEN];
    let (nonce, in_out) = data.split_at_mut(NONCE_LEN);
    let (in_out, tag) = in_out.split_at_mut(val.len());
    in_out.copy_from_slice(val);

    let mut rng = rand::thread_rng();
    rng.try_fill_bytes(nonce)
        .expect("couldn't random fill nonce");
    let nonce = GenericArray::clone_from_slice(nonce);

    // Use the UUID to preform actual cookie Sealing.
    let aad = name.as_bytes();
    let aead = Aes256Gcm::new(GenericArray::from_slice(key.encryption()));
    let aad_tag = aead
        .encrypt_in_place_detached(&nonce, aad, in_out)
        .expect("encryption failure!");

    tag.copy_from_slice(aad_tag.as_slice());

    general_purpose::STANDARD.encode(&data)
}

///Used to deencrypt the Header Values and key values.
pub(crate) fn decrypt(name: &str, value: &str, key: &Key) -> Result<String, SessionError> {
    let data = general_purpose::STANDARD.decode(value)?;
    if data.len() <= NONCE_LEN {
        return Err(SessionError::GenericNotSupportedError(
            "length of decoded data is <= NONCE_LEN".to_owned(),
        ));
    }

    let (nonce, cipher) = data.split_at(NONCE_LEN);
    let payload = Payload {
        msg: cipher,
        aad: name.as_bytes(),
    };

    let aead = Aes256Gcm::new(GenericArray::from_slice(key.encryption()));
    Ok(String::from_utf8(
        aead.decrypt(GenericArray::from_slice(nonce), payload)
            .map_err(|_| {
                SessionError::GenericNotSupportedError(
                    "invalid key/nonce/value: bad seal".to_owned(),
                )
            })?,
    )?)
}
