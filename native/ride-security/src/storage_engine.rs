use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;
use std::sync::Mutex;
use std::fs;
use std::path::PathBuf;

#[napi]
pub struct StorageEngine {
    db_path: PathBuf,
    cache: Mutex<HashMap<String, String>>,
}

#[napi]
impl StorageEngine {
    #[napi(constructor)]
    pub fn new(path: String) -> Self {
        let db_path = PathBuf::from(path);
        let mut cache = HashMap::new();
        
        if db_path.exists() {
            if let Ok(content) = fs::read_to_string(&db_path) {
                if let Ok(data) = serde_json::from_str::<HashMap<String, String>>(&content) {
                    cache = data;
                }
            }
        }

        Self {
            db_path,
            cache: Mutex::new(cache),
        }
    }

    #[napi]
    pub fn get_value(&self, key: String) -> Option<String> {
        let cache = self.cache.lock().unwrap();
        cache.get(&key).cloned()
    }

    #[napi]
    pub fn set_value(&self, key: String, value: String) {
        let mut cache = self.cache.lock().unwrap();
        cache.insert(key, value);
        self.persist(&cache);
    }

    #[napi]
    pub fn delete_value(&self, key: String) -> bool {
        let mut cache = self.cache.lock().unwrap();
        let removed = cache.remove(&key).is_some();
        if removed {
            self.persist(&cache);
        }
        removed
    }

    fn persist(&self, cache: &HashMap<String, String>) {
        if let Ok(content) = serde_json::to_string_pretty(cache) {
            let _ = fs::write(&self.db_path, content);
        }
    }
}
