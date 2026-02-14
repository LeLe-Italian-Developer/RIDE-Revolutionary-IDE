use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Mutex;

#[napi(object)]
#[derive(Clone)]
pub struct ExtHostWorkspaceFolder {
    pub uri: String,
    pub name: String,
    pub index: u32,
}

#[napi]
pub struct ExtHostWorkspace {
    folders: Mutex<Vec<ExtHostWorkspaceFolder>>,
}

#[napi]
impl ExtHostWorkspace {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            folders: Mutex::new(Vec::new()),
        }
    }

    #[napi]
    pub fn set_folders(&self, folders: Vec<ExtHostWorkspaceFolder>) {
        let mut f = self.folders.lock().unwrap();
        *f = folders;
    }

    #[napi]
    pub fn get_folders(&self) -> Vec<ExtHostWorkspaceFolder> {
        self.folders.lock().unwrap().clone()
    }
}
