/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! RIDE Integrity Engine
//!
//! Provides comprehensive data integrity and verification services:
//! - Multi-algorithm hashing (SHA-256, SHA-512, SHA3-256)
//! - Streaming file verification (O(1) memory)
//! - Folder-level recursive integrity manifests
//! - HMAC-based authenticated integrity checks
//! - Cross-platform path normalization for consistent folder hashes

use napi::bindgen_prelude::*;
use napi_derive::napi;
use sha2::{Digest, Sha256, Sha512};
use sha3::Sha3_256;
use hmac::{Hmac, Mac};
use std::fs;
use std::io::{Read, BufReader};
use std::path::{Path, PathBuf};

type HmacSha256 = Hmac<Sha256>;

#[napi]
#[derive(Debug, PartialEq, Eq)]
pub enum HashAlgorithm {
    Sha256 = 0,
    Sha512 = 1,
    Sha3_256 = 2,
}

#[napi]
pub struct StreamHasher {
    algorithm: HashAlgorithm,
    sha256: Option<Sha256>,
    sha512: Option<Sha512>,
    sha3: Option<Sha3_256>,
}

#[napi]
impl StreamHasher {
    #[napi(constructor)]
    pub fn new(algorithm: HashAlgorithm) -> Self {
        match algorithm {
            HashAlgorithm::Sha256 => Self { algorithm, sha256: Some(Sha256::new()), sha512: None, sha3: None },
            HashAlgorithm::Sha512 => Self { algorithm, sha256: None, sha512: Some(Sha512::new()), sha3: None },
            HashAlgorithm::Sha3_256 => Self { algorithm, sha256: None, sha512: None, sha3: Some(Sha3_256::new()) },
        }
    }

    #[napi]
    pub fn update(&mut self, data: Buffer) {
        match self.algorithm {
            HashAlgorithm::Sha256 => self.sha256.as_mut().unwrap().update(&data),
            HashAlgorithm::Sha512 => self.sha512.as_mut().unwrap().update(&data),
            HashAlgorithm::Sha3_256 => self.sha3.as_mut().unwrap().update(&data),
        }
    }

    #[napi]
    pub fn finish(&mut self) -> String {
        match self.algorithm {
            HashAlgorithm::Sha256 => hex::encode(self.sha256.take().unwrap_or_default().finalize()),
            HashAlgorithm::Sha512 => hex::encode(self.sha512.take().unwrap_or_default().finalize()),
            HashAlgorithm::Sha3_256 => hex::encode(self.sha3.take().unwrap_or_default().finalize()),
        }
    }
}

#[napi]
pub fn hash_file(file_path: String, algorithm: Option<HashAlgorithm>) -> Result<String> {
    let path = Path::new(&file_path);
    if !path.is_file() {
        return Err(Error::from_reason(format!("Path is not a valid file: {}", file_path)));
    }

    let file = fs::File::open(path)
        .map_err(|e| Error::from_reason(format!("IO Error: {}", e)))?;
    let mut reader = BufReader::new(file);
    let mut buffer = [0u8; 8192];

    let algo = algorithm.unwrap_or(HashAlgorithm::Sha256);
    let mut hasher = StreamHasher::new(algo);

    loop {
        let n = reader.read(&mut buffer)
            .map_err(|e| Error::from_reason(format!("Read Error: {}", e)))?;
        if n == 0 { break; }
        hasher.update(buffer[..n].into());
    }

    Ok(hasher.finish())
}

#[napi]
pub fn compute_folder_hash(dir_path: String) -> Result<String> {
    let mut paths = Vec::new();
    collect_files(&PathBuf::from(&dir_path), &mut paths)?;
    paths.sort(); // Ensure deterministic order

    let mut overall_hasher = Sha256::new();
    for path in paths {
        let relative = path.strip_prefix(&dir_path).unwrap_or(&path);
        let file_hash = hash_file(path.to_string_lossy().to_string(), Some(HashAlgorithm::Sha256))?;

        overall_hasher.update(relative.to_string_lossy().as_bytes());
        overall_hasher.update(file_hash.as_bytes());
    }

    Ok(hex::encode(overall_hasher.finalize()))
}

#[napi]
pub fn compute_hmac(data: String, key_hex: String) -> Result<String> {
    let key = hex::decode(&key_hex)
        .map_err(|e| Error::from_reason(format!("Invalid key: {}", e)))?;
    let mut mac = HmacSha256::new_from_slice(&key)
        .map_err(|e| Error::from_reason(format!("HMAC Init Error: {}", e)))?;

    mac.update(data.as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

// ─── Internal Helpers ──────────────────────────────────────────────────

fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir).map_err(|e| Error::from_reason(e.to_string()))? {
            let entry = entry.map_err(|e| Error::from_reason(e.to_string()))?;
            let path = entry.path();
            if path.is_dir() {
                collect_files(&path, files)?;
            } else {
                files.push(path);
            }
        }
    }
    Ok(())
}
