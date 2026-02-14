use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;

#[napi(object)]
#[derive(Clone)]
pub struct ExtensionManifest {
    pub name: String,
    pub publisher: String,
    pub version: String,
    pub engines: HashMap<String, String>,
}

#[napi]
pub struct CoreExtensionManagementService {
    installed_extensions: Vec<ExtensionManifest>,
}

#[napi]
impl CoreExtensionManagementService {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            installed_extensions: Vec::new(),
        }
    }

    #[napi]
    pub fn register_extension(&mut self, manifest: ExtensionManifest) {
        self.installed_extensions.push(manifest);
    }

    #[napi]
    pub fn get_installed(&self) -> Vec<ExtensionManifest> {
        self.installed_extensions.clone()
    }
}
