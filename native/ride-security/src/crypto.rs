/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! RIDE Secure Crypto Engine
//!
//! Provides enterprise-grade cryptographic primitives including:
//! - AES-256-GCM (Authenticated Encryption with Associated Data)
//! - Argon2id (State-of-the-art password hashing)
//! - Ed25519 (High-speed digital signatures)
//! - PBKDF2-HMAC-SHA256 (Robust key derivation)
//! - Secure Memory (Zeroization of sensitive data)

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng, AeadCore},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::RngCore, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use ed25519_dalek::{Signature, Signer, Verifier, SigningKey, VerifyingKey};
use hmac::Hmac;
use pbkdf2::pbkdf2;
use sha2::Sha256;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use zeroize::{Zeroize, ZeroizeOnDrop};
use subtle::ConstantTimeEq;

// ─── AES-GCM Encryption ───────────────────────────────────────────────────

#[napi(object)]
pub struct EncryptResult {
    pub ciphertext: String,
    pub nonce: String,
}

#[napi]
pub fn generate_key() -> String {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    let hex_key = hex::encode(key);
    key.zeroize();
    hex_key
}

#[napi]
pub fn encrypt(plaintext: String, key_hex: String, aad: Option<String>) -> Result<EncryptResult> {
    let mut key_bytes = hex::decode(&key_hex)
        .map_err(|e| Error::from_reason(format!("Invalid key hex: {}", e)))?;

    if key_bytes.len() != 32 {
        key_bytes.zeroize();
        return Err(Error::from_reason("Key must be 256 bits (64 hex chars)"));
    }

    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let payload = aes_gcm::aead::Payload {
        msg: plaintext.as_bytes(),
        aad: aad.as_ref().map(|s| s.as_bytes()).unwrap_or(&[]),
    };

    let ciphertext = cipher
        .encrypt(&nonce, payload)
        .map_err(|e| Error::from_reason(format!("Encryption failed: {}", e)))?;

    key_bytes.zeroize();

    Ok(EncryptResult {
        ciphertext: hex::encode(ciphertext),
        nonce: hex::encode(nonce),
    })
}

#[napi]
pub fn decrypt(ciphertext_hex: String, nonce_hex: String, key_hex: String, aad: Option<String>) -> Result<String> {
    let mut key_bytes = hex::decode(&key_hex)
        .map_err(|e| Error::from_reason(format!("Invalid key hex: {}", e)))?;
    let ciphertext = hex::decode(&ciphertext_hex)
        .map_err(|e| Error::from_reason(format!("Invalid ciphertext hex: {}", e)))?;
    let nonce_bytes = hex::decode(&nonce_hex)
        .map_err(|e| Error::from_reason(format!("Invalid nonce hex: {}", e)))?;

    if key_bytes.len() != 32 || nonce_bytes.len() != 12 {
        key_bytes.zeroize();
        return Err(Error::from_reason("Invalid key or nonce length"));
    }

    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let payload = aes_gcm::aead::Payload {
        msg: &ciphertext,
        aad: aad.as_ref().map(|s| s.as_bytes()).unwrap_or(&[]),
    };

    let plaintext = cipher
        .decrypt(nonce, payload)
        .map_err(|_| {
            key_bytes.zeroize();
            Error::from_reason("Decryption failed (integrity check failed)")
        })?;

    key_bytes.zeroize();

    String::from_utf8(plaintext)
        .map_err(|e| Error::from_reason(format!("Invalid UTF-8: {}", e)))
}

// ─── Password Hashing (Argon2id) ──────────────────────────────────────────

#[napi]
pub fn hash_password(password: String) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| Error::from_reason(format!("Hashing failed: {}", e)))?
        .to_string();
    Ok(password_hash)
}

#[napi]
pub fn verify_password(password: String, hash: String) -> bool {
    let parsed_hash = match PasswordHash::new(&hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

// ─── Key Derivation (PBKDF2) ───────────────────────────────────────────────

#[napi]
pub fn derive_key(password: String, salt_hex: String, iterations: u32) -> Result<String> {
    let salt = hex::decode(&salt_hex)
        .map_err(|e| Error::from_reason(format!("Invalid salt hex: {}", e)))?;
    let mut derived_key = [0u8; 32];

    pbkdf2::<Hmac<Sha256>>(
        password.as_bytes(),
        &salt,
        iterations,
        &mut derived_key,
    ).map_err(|e| Error::from_reason(format!("Key derivation failed: {}", e)))?;

    let result = hex::encode(derived_key);
    derived_key.zeroize();
    Ok(result)
}

// ─── Digital Signatures (Ed25519) ──────────────────────────────────────────

#[napi(object)]
pub struct KeyPair {
    pub public_key: String,
    pub private_key: String,
}

#[napi]
pub fn generate_signing_keypair() -> KeyPair {
    let mut secret = [0u8; 32];
    OsRng.fill_bytes(&mut secret);
    let signing_key = SigningKey::from_bytes(&secret);
    let verifying_key = VerifyingKey::from(&signing_key);

    let result = KeyPair {
        public_key: hex::encode(verifying_key.to_bytes()),
        private_key: hex::encode(signing_key.to_bytes()),
    };
    secret.zeroize();
    result
}

#[napi]
pub fn sign_message(message: String, private_key_hex: String) -> Result<String> {
    let mut key_bytes = hex::decode(&private_key_hex)
        .map_err(|e| Error::from_reason(format!("Invalid key hex: {}", e)))?;

    if key_bytes.len() != 32 {
        key_bytes.zeroize();
        return Err(Error::from_reason("Invalid signing key length"));
    }

    let key_arr: &[u8; 32] = key_bytes.as_slice().try_into().map_err(|_| Error::from_reason("Invalid key length"))?;
    let signing_key = SigningKey::from_bytes(key_arr);
    let signature = signing_key.sign(message.as_bytes());

    key_bytes.zeroize();
    Ok(hex::encode(signature.to_bytes()))
}

#[napi]
pub fn verify_signature(message: String, signature_hex: String, public_key_hex: String) -> bool {
    let pub_bytes = match hex::decode(&public_key_hex) {
        Ok(b) if b.len() == 32 => b,
        _ => return false,
    };
    let sig_bytes = match hex::decode(&signature_hex) {
        Ok(b) if b.len() == 64 => b,
        _ => return false,
    };

    let verifying_key = match VerifyingKey::from_bytes(&pub_bytes.try_into().unwrap()) {
        Ok(k) => k,
        Err(_) => return false,
    };
    let signature = Signature::from_bytes(&sig_bytes.try_into().unwrap());

    verifying_key.verify(message.as_bytes(), &signature).is_ok()
}

// ─── Utilities ────────────────────────────────────────────────────────────

#[napi]
pub fn constant_time_equals(a: String, b: String) -> bool {
    a.as_bytes().ct_eq(b.as_bytes()).into()
}

#[derive(Zeroize, ZeroizeOnDrop)]
struct SecretData {
    data: Vec<u8>,
}
