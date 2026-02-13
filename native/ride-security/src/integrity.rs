/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! SHA-256 file integrity verification for extension security.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

/// Compute the SHA-256 hash of a file.
///
/// # Arguments
/// * `file_path` - Absolute path to the file to hash
///
/// # Returns
/// A 64-character lowercase hex string representing the SHA-256 digest
#[napi]
pub fn hash_file(file_path: String) -> Result<String> {
    let path = Path::new(&file_path);

    if !path.exists() {
        return Err(Error::from_reason(format!("File not found: {}", file_path)));
    }

    if !path.is_file() {
        return Err(Error::from_reason(format!("Not a file: {}", file_path)));
    }

    let contents = fs::read(path)
        .map_err(|e| Error::from_reason(format!("Failed to read file {}: {}", file_path, e)))?;

    let mut hasher = Sha256::new();
    hasher.update(&contents);
    let result = hasher.finalize();

    Ok(hex::encode(result))
}

/// Compute the SHA-256 hash of a string/buffer.
///
/// # Arguments
/// * `data` - The data to hash (UTF-8 string)
///
/// # Returns
/// A 64-character lowercase hex string representing the SHA-256 digest
#[napi]
pub fn hash_string(data: String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hex::encode(hasher.finalize())
}

/// Verify a file's integrity against an expected SHA-256 hash.
///
/// # Arguments
/// * `file_path` - Absolute path to the file to verify
/// * `expected_hash` - Expected 64-character lowercase hex SHA-256 hash
///
/// # Returns
/// `true` if the file's hash matches the expected hash, `false` otherwise
#[napi]
pub fn verify_file_integrity(file_path: String, expected_hash: String) -> Result<bool> {
    let actual_hash = hash_file(file_path)?;
    Ok(actual_hash == expected_hash.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_hash_string() {
        // Known SHA-256 of "hello"
        let hash = hash_string("hello".to_string());
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_hash_file_and_verify() {
        let dir = std::env::temp_dir();
        let file_path = dir.join("ride_test_integrity.txt");
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"test content for integrity check").unwrap();
        drop(file);

        let hash = hash_file(file_path.to_str().unwrap().to_string()).unwrap();
        let verified = verify_file_integrity(
            file_path.to_str().unwrap().to_string(),
            hash.clone(),
        )
        .unwrap();

        assert!(verified);

        // Wrong hash should fail
        let wrong = verify_file_integrity(
            file_path.to_str().unwrap().to_string(),
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        )
        .unwrap();

        assert!(!wrong);

        fs::remove_file(file_path).ok();
    }
}
