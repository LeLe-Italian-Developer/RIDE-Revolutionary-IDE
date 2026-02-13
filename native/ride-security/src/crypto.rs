/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! AES-256-GCM encryption and decryption for secure settings and credential storage.

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, AeadCore, Key, Nonce,
};
use napi::bindgen_prelude::*;
use napi_derive::napi;

/// Result of an encryption operation.
/// Contains the ciphertext and the nonce (IV) needed for decryption.
#[napi(object)]
pub struct EncryptResult {
    /// Hex-encoded ciphertext (includes authentication tag)
    pub ciphertext: String,
    /// Hex-encoded 96-bit nonce/IV
    pub nonce: String,
}

/// Generate a random 256-bit encryption key.
/// Returns a 64-character hex string.
#[napi]
pub fn generate_key() -> String {
    let key = Aes256Gcm::generate_key(OsRng);
    hex::encode(key)
}

/// Encrypt plaintext using AES-256-GCM.
///
/// # Arguments
/// * `plaintext` - The data to encrypt (UTF-8 string)
/// * `key_hex` - 64-character hex-encoded 256-bit key
///
/// # Returns
/// An `EncryptResult` containing the hex-encoded ciphertext and nonce
#[napi]
pub fn encrypt(plaintext: String, key_hex: String) -> Result<EncryptResult> {
    let key_bytes = hex::decode(&key_hex)
        .map_err(|e| Error::from_reason(format!("Invalid key hex: {}", e)))?;

    if key_bytes.len() != 32 {
        return Err(Error::from_reason("Key must be 256 bits (64 hex chars)"));
    }

    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| Error::from_reason(format!("Encryption failed: {}", e)))?;

    Ok(EncryptResult {
        ciphertext: hex::encode(ciphertext),
        nonce: hex::encode(nonce),
    })
}

/// Decrypt ciphertext using AES-256-GCM.
///
/// # Arguments
/// * `ciphertext_hex` - Hex-encoded ciphertext (from `encrypt`)
/// * `nonce_hex` - Hex-encoded nonce (from `encrypt`)
/// * `key_hex` - 64-character hex-encoded 256-bit key
///
/// # Returns
/// The original plaintext string
#[napi]
pub fn decrypt(ciphertext_hex: String, nonce_hex: String, key_hex: String) -> Result<String> {
    let key_bytes = hex::decode(&key_hex)
        .map_err(|e| Error::from_reason(format!("Invalid key hex: {}", e)))?;
    let ciphertext = hex::decode(&ciphertext_hex)
        .map_err(|e| Error::from_reason(format!("Invalid ciphertext hex: {}", e)))?;
    let nonce_bytes = hex::decode(&nonce_hex)
        .map_err(|e| Error::from_reason(format!("Invalid nonce hex: {}", e)))?;

    if key_bytes.len() != 32 {
        return Err(Error::from_reason("Key must be 256 bits (64 hex chars)"));
    }
    if nonce_bytes.len() != 12 {
        return Err(Error::from_reason("Nonce must be 96 bits (24 hex chars)"));
    }

    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| Error::from_reason("Decryption failed: invalid key, nonce, or tampered data"))?;

    String::from_utf8(plaintext)
        .map_err(|e| Error::from_reason(format!("Decrypted data is not valid UTF-8: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = generate_key();
        let plaintext = "Hello, RIDE! This is a secret.".to_string();

        let result = encrypt(plaintext.clone(), key.clone()).unwrap();
        let decrypted = decrypt(result.ciphertext, result.nonce, key).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = generate_key();
        let key2 = generate_key();
        let plaintext = "secret data".to_string();

        let result = encrypt(plaintext, key1).unwrap();
        let decrypted = decrypt(result.ciphertext, result.nonce, key2);

        assert!(decrypted.is_err());
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let key = generate_key();
        let plaintext = "secret data".to_string();

        let result = encrypt(plaintext, key.clone()).unwrap();

        // Tamper with ciphertext
        let mut tampered = hex::decode(&result.ciphertext).unwrap();
        if let Some(byte) = tampered.first_mut() {
            *byte ^= 0xff;
        }
        let tampered_hex = hex::encode(tampered);

        let decrypted = decrypt(tampered_hex, result.nonce, key);
        assert!(decrypted.is_err());
    }
}
