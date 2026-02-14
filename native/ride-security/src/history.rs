use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Mutex;

#[napi]
pub struct HistoryService {
    recent_files: Mutex<Vec<String>>,
}

#[napi]
impl HistoryService {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            recent_files: Mutex::new(Vec::new()),
        }
    }

    #[napi]
    pub fn add_recently_used(&self, resource: String) {
        let mut recent = self.recent_files.lock().unwrap();
        recent.retain(|r| r != &resource);
        recent.insert(0, resource);
        if recent.len() > 100 {
            recent.pop();
        }
    }

    #[napi]
    pub fn get_recently_used(&self) -> Vec<String> {
        self.recent_files.lock().unwrap().clone()
    }

    #[napi]
    pub fn clear(&self) {
        self.recent_files.lock().unwrap().clear();
    }
}
