use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;
use crate::pfs;

#[napi(object)]
pub struct WorkbenchFileStat {
    pub is_file: bool,
    pub is_directory: bool,
    pub mtime: f64,
    pub size: f64,
    pub name: String,
}

#[napi]
pub struct FileService {
    capabilities: HashMap<String, bool>,
}

#[napi]
impl FileService {
    #[napi(constructor)]
    pub fn new() -> Self {
        let mut capabilities = HashMap::new();
        capabilities.insert("read".to_string(), true);
        capabilities.insert("write".to_string(), true);
        capabilities.insert("watch".to_string(), true);
        Self { capabilities }
    }

    #[napi]
    pub async fn resolve_stat(&self, path: String) -> Result<WorkbenchFileStat> {
        let metadata = std::fs::metadata(&path).map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(WorkbenchFileStat {
            is_file: metadata.is_file(),
            is_directory: metadata.is_dir(),
            mtime: metadata.modified().unwrap().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as f64,
            size: metadata.len() as f64,
            name: std::path::Path::new(&path).file_name().unwrap_or_default().to_string_lossy().into_owned(),
        })
    }

    #[napi]
    pub fn can_handle(&self, scheme: String) -> bool {
        scheme == "file" || scheme == "vscode-remote"
    }
}
