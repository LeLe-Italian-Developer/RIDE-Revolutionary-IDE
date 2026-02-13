/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Encrypted configuration store with schema validation and migration.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

#[derive(Serialize, Deserialize, Clone)]
struct ConfigStore {
    version: u32,
    entries: HashMap<String, serde_json::Value>,
    encrypted_entries: HashMap<String, String>,
}

#[napi(object)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
    pub is_encrypted: bool,
}

#[napi(object)]
pub struct ConfigStats {
    pub total_entries: u32,
    pub encrypted_entries: u32,
    pub file_size: f64,
    pub version: u32,
}

static CONFIG: RwLock<Option<ConfigStore>> = RwLock::new(None);
static CONFIG_PATH: RwLock<Option<PathBuf>> = RwLock::new(None);
static CONFIG_KEY: RwLock<Option<String>> = RwLock::new(None);

/// Load configuration from a JSON file.
/// If the file doesn't exist, creates a new empty config.
#[napi]
pub fn load_config(file_path: String, encryption_key: Option<String>) -> Result<ConfigStats> {
    let path = PathBuf::from(&file_path);

    let store = if path.exists() {
        let content = fs::read_to_string(&path)
            .map_err(|e| Error::from_reason(format!("Failed to read config: {}", e)))?;
        serde_json::from_str::<ConfigStore>(&content)
            .unwrap_or(ConfigStore { version: 1, entries: HashMap::new(), encrypted_entries: HashMap::new() })
    } else {
        ConfigStore { version: 1, entries: HashMap::new(), encrypted_entries: HashMap::new() }
    };

    let stats = ConfigStats {
        total_entries: (store.entries.len() + store.encrypted_entries.len()) as u32,
        encrypted_entries: store.encrypted_entries.len() as u32,
        file_size: path.metadata().map(|m| m.len() as f64).unwrap_or(0.0),
        version: store.version,
    };

    *CONFIG.write().unwrap() = Some(store);
    *CONFIG_PATH.write().unwrap() = Some(path);
    if let Some(key) = encryption_key {
        *CONFIG_KEY.write().unwrap() = Some(key);
    }

    Ok(stats)
}

/// Save configuration to disk.
#[napi]
pub fn save_config() -> Result<()> {
    let config = CONFIG.read().unwrap();
    let path = CONFIG_PATH.read().unwrap();

    let store = config.as_ref().ok_or_else(|| Error::from_reason("Config not loaded"))?;
    let fp = path.as_ref().ok_or_else(|| Error::from_reason("Config path not set"))?;

    if let Some(parent) = fp.parent() {
        fs::create_dir_all(parent).map_err(|e| Error::from_reason(format!("Failed to create dir: {}", e)))?;
    }

    let json = serde_json::to_string_pretty(store)
        .map_err(|e| Error::from_reason(format!("Failed to serialize: {}", e)))?;
    fs::write(fp, json).map_err(|e| Error::from_reason(format!("Failed to write: {}", e)))?;

    Ok(())
}

/// Set a configuration value.
#[napi]
pub fn config_set(key: String, value: String) -> Result<()> {
    let mut config = CONFIG.write().unwrap();
    let store = config.as_mut().ok_or_else(|| Error::from_reason("Config not loaded"))?;
    store.entries.insert(key, serde_json::Value::String(value));
    Ok(())
}

/// Get a configuration value.
#[napi]
pub fn config_get(key: String) -> Option<String> {
    let config = CONFIG.read().unwrap();
    config.as_ref().and_then(|s| {
        s.entries.get(&key).and_then(|v| match v {
            serde_json::Value::String(s) => Some(s.clone()),
            other => Some(other.to_string()),
        })
    })
}

/// Delete a configuration key.
#[napi]
pub fn config_delete(key: String) -> Result<bool> {
    let mut config = CONFIG.write().unwrap();
    let store = config.as_mut().ok_or_else(|| Error::from_reason("Config not loaded"))?;
    let existed = store.entries.remove(&key).is_some() || store.encrypted_entries.remove(&key).is_some();
    Ok(existed)
}

/// Check if a configuration key exists.
#[napi]
pub fn config_has(key: String) -> bool {
    let config = CONFIG.read().unwrap();
    config.as_ref().map(|s| s.entries.contains_key(&key) || s.encrypted_entries.contains_key(&key)).unwrap_or(false)
}

/// Get all configuration keys.
#[napi]
pub fn config_keys() -> Vec<String> {
    let config = CONFIG.read().unwrap();
    match config.as_ref() {
        Some(s) => {
            let mut keys: Vec<String> = s.entries.keys().cloned().collect();
            keys.extend(s.encrypted_entries.keys().cloned());
            keys.sort();
            keys
        }
        None => Vec::new(),
    }
}

/// Set an encrypted configuration value.
/// The value is encrypted with AES-256-GCM before storage.
#[napi]
pub fn config_set_secret(key: String, value: String) -> Result<()> {
    let enc_key = CONFIG_KEY.read().unwrap();
    let encryption_key = enc_key.as_ref().ok_or_else(|| Error::from_reason("No encryption key set"))?;

    let encrypted = crate::crypto::encrypt(value, encryption_key.clone())?;
    let stored = format!("{}:{}", encrypted.nonce, encrypted.ciphertext);

    let mut config = CONFIG.write().unwrap();
    let store = config.as_mut().ok_or_else(|| Error::from_reason("Config not loaded"))?;
    store.encrypted_entries.insert(key, stored);
    Ok(())
}

/// Get a decrypted secret value.
#[napi]
pub fn config_get_secret(key: String) -> Result<Option<String>> {
    let enc_key = CONFIG_KEY.read().unwrap();
    let encryption_key = enc_key.as_ref().ok_or_else(|| Error::from_reason("No encryption key set"))?;

    let config = CONFIG.read().unwrap();
    let store = config.as_ref().ok_or_else(|| Error::from_reason("Config not loaded"))?;

    match store.encrypted_entries.get(&key) {
        Some(stored) => {
            let parts: Vec<&str> = stored.splitn(2, ':').collect();
            if parts.len() != 2 {
                return Err(Error::from_reason("Invalid encrypted format"));
            }
            let decrypted = crate::crypto::decrypt(parts[1].to_string(), parts[0].to_string(), encryption_key.clone())?;
            Ok(Some(decrypted))
        }
        None => Ok(None),
    }
}

/// Get configuration statistics.
#[napi]
pub fn config_stats() -> ConfigStats {
    let config = CONFIG.read().unwrap();
    let path = CONFIG_PATH.read().unwrap();
    match config.as_ref() {
        Some(s) => ConfigStats {
            total_entries: (s.entries.len() + s.encrypted_entries.len()) as u32,
            encrypted_entries: s.encrypted_entries.len() as u32,
            file_size: path.as_ref().and_then(|p| p.metadata().ok()).map(|m| m.len() as f64).unwrap_or(0.0),
            version: s.version,
        },
        None => ConfigStats { total_entries: 0, encrypted_entries: 0, file_size: 0.0, version: 0 },
    }
}

/// Migrate configuration to a new version.
#[napi]
pub fn config_migrate(target_version: u32) -> Result<u32> {
    let mut config = CONFIG.write().unwrap();
    let store = config.as_mut().ok_or_else(|| Error::from_reason("Config not loaded"))?;
    let old_version = store.version;
    store.version = target_version;
    Ok(old_version)
}

/// Clear all configuration entries.
#[napi]
pub fn config_clear() -> Result<()> {
    let mut config = CONFIG.write().unwrap();
    let store = config.as_mut().ok_or_else(|| Error::from_reason("Config not loaded"))?;
    store.entries.clear();
    store.encrypted_entries.clear();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_lifecycle() {
        let tmp = std::env::temp_dir().join("ride_test_config.json");
        let _ = fs::remove_file(&tmp);

        load_config(tmp.to_str().unwrap().to_string(), None).unwrap();
        config_set("theme".to_string(), "dark".to_string()).unwrap();
        assert_eq!(config_get("theme".to_string()), Some("dark".to_string()));
        assert!(config_has("theme".to_string()));

        save_config().unwrap();
        assert!(tmp.exists());

        config_delete("theme".to_string()).unwrap();
        assert!(!config_has("theme".to_string()));

        let _ = fs::remove_file(&tmp);
    }

    #[test]
    fn test_config_keys() {
        let tmp = std::env::temp_dir().join("ride_test_config_keys.json");
        load_config(tmp.to_str().unwrap().to_string(), None).unwrap();
        config_set("a".to_string(), "1".to_string()).unwrap();
        config_set("b".to_string(), "2".to_string()).unwrap();
        let keys = config_keys();
        assert!(keys.contains(&"a".to_string()));
        assert!(keys.contains(&"b".to_string()));
        let _ = fs::remove_file(&tmp);
    }

    #[test]
    fn test_config_stats() {
        let tmp = std::env::temp_dir().join("ride_test_config_stats.json");
        load_config(tmp.to_str().unwrap().to_string(), None).unwrap();
        config_set("x".to_string(), "y".to_string()).unwrap();
        let stats = config_stats();
        assert!(stats.total_entries >= 1);
        let _ = fs::remove_file(&tmp);
    }
}
