use crate::{DatabasePool, Session, SessionError, SessionID, SessionStore};
use aes_gcm::aead::{generic_array::GenericArray, Aead, AeadInPlace, KeyInit, Payload};
use aes_gcm::Aes256Gcm;
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Duration, Utc};
pub use cookie::Key;
use rand::RngCore;
use std::fmt::{self, Debug, Formatter};
use uuid::Uuid;

// Keep these in sync, and keep the key len synced with the `private` docs as
// well as the `KEYS_INFO` const in secure::Key. from cookie-rs
pub(crate) const NONCE_LEN: usize = 12;
pub(crate) const TAG_LEN: usize = 16;
pub(crate) const KEY_LEN: usize = 32;

#[derive(Clone)]
pub struct SessionKey {
    pub(crate) id: SessionID,
    pub(crate) autoremove: DateTime<Utc>,
    pub(crate) key: Key,
}

impl Debug for SessionKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionKey").field("id", &self.id).finish()
    }
}

impl SessionKey {
    pub(crate) fn new() -> Self {
        Self {
            id: SessionID(Uuid::default()),
            autoremove: Utc::now(),
            key: Key::generate(),
        }
    }

    /// Uses the Cookie Value to check if the key Exists or not.
    /// If the key does not Exist in the inner memory table then we load it from the database.
    /// if neither work then we make a new key.
    pub(crate) async fn get_or_create<S>(store: &SessionStore<S>, value: Option<Uuid>) -> Self
    where
        S: DatabasePool + Clone + Debug + Sync + Send + 'static,
    {
        if let Some(v) = value {
            let id: SessionID = SessionID(v);

            if let Some(mut value) = store.keys.get_mut(&id.inner()) {
                value.autoremove = Utc::now() + store.config.memory_lifespan;
                return value.clone();
            }

            if let Ok(Some(value)) = store.load_key(id.inner()).await {
                store.keys.insert(id.inner(), value.clone());
                return value;
            }
        }

        let id = Session::generate_uuid(store).await;
        let key = Key::generate();

        let session_key = Self {
            id,
            autoremove: Utc::now() + store.config.memory_lifespan,
            key,
        };

        store
            .keys
            .insert(session_key.id.inner(), session_key.clone());
        session_key
    }

    /// Renews the KeyID and Key. This can make things more Secure since it rotates the old encryption keys out
    /// and rotates the old ID out so if anyone did get the Key or ID it will be worthless for them.
    pub(crate) async fn renew<S>(&mut self, store: &SessionStore<S>) -> Result<String, SessionError>
    where
        S: DatabasePool + Clone + Debug + Sync + Send + 'static,
    {
        let old_id = self.id.inner();

        let _ = store.keys.remove(&old_id);
        // When we renew a SessionID we also should renew the Key SessionID and Key for extra Security.
        // This is the best time to do this as it doesnt disturb the force and loss data.
        // Switching the config Key and config Database Key however will invalidate everything.
        self.id = Session::generate_uuid(store).await;
        self.key = Key::generate();

        store.keys.insert(self.id.inner(), self.clone());
        Ok(old_id)
    }

    ///Encrypts the Key for Database Storage using the master key.
    pub(crate) fn encrypt(&self, master_key: Key) -> String {
        let key = self.key.master();

        let mut data = vec![0; NONCE_LEN + key.len() + TAG_LEN];
        let (nonce, in_out) = data.split_at_mut(NONCE_LEN);
        let (in_out, tag) = in_out.split_at_mut(key.len());
        in_out.copy_from_slice(key);

        let mut rng = rand::thread_rng();
        rng.try_fill_bytes(nonce)
            .expect("couldn't random fill nonce");
        let nonce = GenericArray::clone_from_slice(nonce);

        // Use the UUID to preform actual cookie Sealing.
        let binding = self.id.inner();
        let aad = binding.as_bytes();
        let aead = Aes256Gcm::new(GenericArray::from_slice(master_key.encryption()));
        let aad_tag = aead
            .encrypt_in_place_detached(&nonce, aad, in_out)
            .expect("encryption failure!");

        tag.copy_from_slice(aad_tag.as_slice());

        general_purpose::STANDARD.encode(&data)
    }

    pub(crate) fn decrypt(
        name: SessionID,
        value: &str,
        key: Key,
        memory_life_span: Duration,
    ) -> Result<Self, SessionError> {
        let data = general_purpose::STANDARD.decode(value)?;
        if data.len() <= NONCE_LEN {
            return Err(SessionError::GenericNotSupportedError(
                "length of decoded data is <= NONCE_LEN".to_owned(),
            ));
        }

        let (nonce, cipher) = data.split_at(NONCE_LEN);
        let binding = name.inner();
        let payload = Payload {
            msg: cipher,
            aad: binding.as_bytes(),
        };

        let aead = Aes256Gcm::new(GenericArray::from_slice(key.encryption()));
        let key = aead
            .decrypt(GenericArray::from_slice(nonce), payload)
            .map_err(|_| {
                SessionError::GenericNotSupportedError(
                    "invalid key/nonce/value: bad seal".to_owned(),
                )
            })?;

        Ok(Self {
            id: name,
            autoremove: Utc::now() + memory_life_span,
            key: Key::from(&key),
        })
    }
}
