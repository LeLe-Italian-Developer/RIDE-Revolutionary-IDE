use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;
use std::sync::Mutex;

#[napi]
pub struct ExtHostLanguages {
    diagnostics: Mutex<HashMap<String, Vec<String>>>, // Owner -> Serialized Diagnostics
}

#[napi]
impl ExtHostLanguages {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            diagnostics: Mutex::new(HashMap::new()),
        }
    }

    #[napi]
    pub fn set_diagnostics(&self, owner: String, data: Vec<String>) {
        let mut diag = self.diagnostics.lock().unwrap();
        diag.insert(owner, data);
    }

    #[napi]
    pub fn clear_diagnostics(&self, owner: String) {
        let mut diag = self.diagnostics.lock().unwrap();
        diag.remove(&owner);
    }
}
