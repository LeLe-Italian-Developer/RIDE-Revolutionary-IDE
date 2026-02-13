/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Hashing and UUID utilities — Rust port of `src/vs/base/common/hash.ts`
//! and `uuid.ts`.

use napi_derive::napi;
use napi::bindgen_prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Compute a fast non-cryptographic hash of a string (FNV-1a inspired).
#[napi]
pub fn string_hash(value: String) -> f64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    (hasher.finish() & 0x7FFFFFFFFFFFFFFF) as f64
}

/// Compute a hash of multiple values combined.
#[napi]
pub fn combined_hash(values: Vec<String>) -> f64 {
    let mut hasher = DefaultHasher::new();
    for v in &values {
        v.hash(&mut hasher);
    }
    (hasher.finish() & 0x7FFFFFFFFFFFFFFF) as f64
}

/// Generate a UUID v4.
#[napi]
pub fn generate_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Check if a string is a valid UUID.
#[napi]
pub fn is_uuid(value: String) -> bool {
    uuid::Uuid::parse_str(&value).is_ok()
}

/// Compute a 32-bit MurmurHash3 of a string.
#[napi]
pub fn murmur_hash(value: String, seed: Option<u32>) -> u32 {
    let s = seed.unwrap_or(0);
    let bytes = value.as_bytes();
    let len = bytes.len();
    let mut h: u32 = s;
    let nblocks = len / 4;

    // Body
    for i in 0..nblocks {
        let k = u32::from_le_bytes([
            bytes[i * 4],
            bytes[i * 4 + 1],
            bytes[i * 4 + 2],
            bytes[i * 4 + 3],
        ]);
        let k = k.wrapping_mul(0xcc9e2d51);
        let k = k.rotate_left(15);
        let k = k.wrapping_mul(0x1b873593);
        h ^= k;
        h = h.rotate_left(13);
        h = h.wrapping_mul(5).wrapping_add(0xe6546b64);
    }

    // Tail
    let tail = &bytes[nblocks * 4..];
    let mut k1: u32 = 0;
    if tail.len() >= 3 { k1 ^= (tail[2] as u32) << 16; }
    if tail.len() >= 2 { k1 ^= (tail[1] as u32) << 8; }
    if !tail.is_empty() {
        k1 ^= tail[0] as u32;
        k1 = k1.wrapping_mul(0xcc9e2d51);
        k1 = k1.rotate_left(15);
        k1 = k1.wrapping_mul(0x1b873593);
        h ^= k1;
    }

    // Finalization
    h ^= len as u32;
    h ^= h >> 16;
    h = h.wrapping_mul(0x85ebca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2ae35);
    h ^= h >> 16;
    h
}

/// Compute a 64-bit SipHash of a string.
#[napi]
pub fn sip_hash(value: String) -> f64 {
    let mut hasher = siphasher::sip::SipHasher13::new();
    value.as_bytes().hash(&mut hasher);
    (hasher.finish() & 0x7FFFFFFFFFFFFFFF) as f64
}

/// Generate a short unique ID (8 characters, alphanumeric).
#[napi]
pub fn short_id() -> String {
    let uuid = uuid::Uuid::new_v4();
    let hex = uuid.simple().to_string();
    hex[..8].to_string()
}

/// Generate a nanoid-style unique ID of the given length.
#[napi]
pub fn nano_id(length: Option<u32>) -> String {
    let len = length.unwrap_or(21) as usize;
    let alphabet = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_-";
    let bytes: Vec<u8> = (0..len).map(|_| {
        let uuid = uuid::Uuid::new_v4();
        let b = uuid.as_bytes()[0];
        b % alphabet.len() as u8
    }).collect();

    bytes.iter().map(|&i| alphabet.as_bytes()[i as usize] as char).collect()
}

/// Compute a content-based hash suitable for ETags.
#[napi]
pub fn etag_hash(content: String) -> String {
    let hash = string_hash(content) as u64;
    format!("\"{:x}\"", hash)
}

/// Compute a deterministic hash for a file path that can be used for caching.
#[napi]
pub fn path_hash(path: String) -> String {
    let normalized = path.to_lowercase().replace('\\', "/");
    let hash = string_hash(normalized) as u64;
    format!("{:016x}", hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_hash_deterministic() {
        let h1 = string_hash("hello".into());
        let h2 = string_hash("hello".into());
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_string_hash_different() {
        let h1 = string_hash("hello".into());
        let h2 = string_hash("world".into());
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_uuid() {
        let id = generate_uuid();
        assert!(is_uuid(id.clone()));
        assert!(!is_uuid("not-a-uuid".into()));
    }

    #[test]
    fn test_murmur_hash() {
        let h1 = murmur_hash("hello".into(), None);
        let h2 = murmur_hash("hello".into(), None);
        assert_eq!(h1, h2);

        let h3 = murmur_hash("hello".into(), Some(42));
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_short_id() {
        let id = short_id();
        assert_eq!(id.len(), 8);
    }
}
