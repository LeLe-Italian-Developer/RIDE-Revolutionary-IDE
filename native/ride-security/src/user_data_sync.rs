use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;

#[napi(object)]
pub struct SyncResource {
    pub name: String,
    pub content: String,
    pub remote_content: Option<String>,
}

#[napi]
pub struct UserDataSyncStore {
    resources: HashMap<String, String>,
}

#[napi]
impl UserDataSyncStore {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    #[napi]
    pub fn detect_conflicts(&self, local: SyncResource) -> bool {
        if let Some(remote) = local.remote_content {
            return local.content != remote;
        }
        false
    }
}
