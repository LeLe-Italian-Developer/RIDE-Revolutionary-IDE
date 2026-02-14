use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Mutex;

#[napi(object)]
#[derive(Clone)]
pub struct ExtHostSCMProvider {
    pub id: String,
    pub label: String,
    pub root_uri: Option<String>,
}

#[napi]
pub struct ExtHostSCM {
    providers: Mutex<Vec<ExtHostSCMProvider>>,
}

#[napi]
impl ExtHostSCM {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            providers: Mutex::new(Vec::new()),
        }
    }

    #[napi]
    pub fn register_provider(&self, provider: ExtHostSCMProvider) {
        let mut p = self.providers.lock().unwrap();
        p.push(provider);
    }

    #[napi]
    pub fn unregister_provider(&self, id: String) -> bool {
        let mut p = self.providers.lock().unwrap();
        let len_before = p.len();
        p.retain(|prov| prov.id != id);
        p.len() < len_before
    }
}
