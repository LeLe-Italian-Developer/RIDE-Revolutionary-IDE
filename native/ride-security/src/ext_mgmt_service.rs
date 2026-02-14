use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Mutex;

#[napi(object)]
#[derive(Clone)]
pub struct LocalExtension {
    pub id: String,
    pub version: String,
    pub location: String, // Path on disk
    pub publisher: String,
    pub name: String,
    pub description: Option<String>,
}

#[napi]
pub struct WorkbenchExtensionManagementService {
    installed: Mutex<Vec<LocalExtension>>,
}

#[napi]
impl WorkbenchExtensionManagementService {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            installed: Mutex::new(Vec::new())
        }
    }

    #[napi]
    pub fn get_installed(&self) -> Vec<LocalExtension> {
        self.installed.lock().unwrap().clone()
    }

    #[napi]
    pub fn install(&self, vsix_path: String) -> Result<LocalExtension> {
        // Placeholder implementation
        // Real impl would unzip VSIX, read manifest, move to extensions dir
        let file_name = std::path::Path::new(&vsix_path)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("unknown.vsix");

        let name_part = file_name.replace(".vsix", "");

        let new_ext = LocalExtension {
            id: format!("local.{}", name_part),
            version: "1.0.0".to_string(),
            location: vsix_path.clone(),
            publisher: "local".to_string(),
            name: name_part,
            description: Some("Installed via Rust ExtensionManagementService".to_string()),
        };

        self.installed.lock().unwrap().push(new_ext.clone());
        Ok(new_ext)
    }

    #[napi]
    pub fn uninstall(&self, id: String) -> Result<bool> {
        let mut installed = self.installed.lock().unwrap();
        if let Some(pos) = installed.iter().position(|e| e.id == id) {
            installed.remove(pos);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
