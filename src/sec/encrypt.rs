use crate::SessionError;
use aes_gcm::aead::{generic_array::GenericArray, Aead, AeadInPlace, KeyInit, Payload};
use aes_gcm::Aes256Gcm;
use base64::{engine::general_purpose, Engine as _};
use cookie::Key;
use rand::RngCore;

pub(crate) const NONCE_LEN: usize = 12;
pub(crate) const TAG_LEN: usize = 16;
pub(crate) const KEY_LEN: usize = 32;

///Used to encrypt the database Values
pub(crate) fn encrypt(name: &str, value: &str, key: &Key) -> Result<String, &'static str> {
    let val = value.as_bytes();

    let mut data = vec![0; NONCE_LEN + val.len() + TAG_LEN];
    let (nonce, in_out) = data.split_at_mut(NONCE_LEN);
    let (in_out, tag) = in_out.split_at_mut(val.len());
    in_out.copy_from_slice(val);

    let mut rng = rand::thread_rng();
    let _ = rng
        .try_fill_bytes(nonce)
        .map_err(|_| "couldn't random fill nonce")?;

    let nonce = GenericArray::clone_from_slice(nonce);

    // Use the UUID to preform actual cookie Sealing.
    let aad = name.as_bytes();
    let aead = Aes256Gcm::new(GenericArray::from_slice(key.encryption()));
    let aad_tag = aead
        .encrypt_in_place_detached(&nonce, aad, in_out)
        .map_err(|_| "encryption failure!")?;

    tag.copy_from_slice(aad_tag.as_slice());

    Ok(general_purpose::STANDARD.encode(&data))
}

///Used to decrypt the database Values.
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
