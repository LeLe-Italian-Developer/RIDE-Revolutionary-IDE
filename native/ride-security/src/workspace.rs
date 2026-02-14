use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Mutex;

#[napi(object)]
#[derive(Clone)]
pub struct WorkspaceFolder {
    pub uri: String,
    pub name: String,
    pub index: u32,
}

#[napi]
pub struct WorkspaceService {
    folders: Mutex<Vec<WorkspaceFolder>>,
}

#[napi]
impl WorkspaceService {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            folders: Mutex::new(Vec::new()),
        }
    }

    #[napi]
    pub fn get_folders(&self) -> Vec<WorkspaceFolder> {
        self.folders.lock().unwrap().clone()
    }

    #[napi]
    pub fn add_folder(&self, folder: WorkspaceFolder) {
        let mut folders = self.folders.lock().unwrap();
        if !folders.iter().any(|f| f.uri == folder.uri) {
            folders.push(folder);
        }
    }

    #[napi]
    pub fn remove_folder(&self, uri: String) -> bool {
        let mut folders = self.folders.lock().unwrap();
        let len_before = folders.len();
        folders.retain(|f| f.uri != uri);
        folders.len() < len_before
    }
}
