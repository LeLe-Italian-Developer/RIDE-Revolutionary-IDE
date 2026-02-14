/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[napi(object)]
#[derive(Clone, Debug)]
pub struct NativeEnvironmentPaths {
    pub user_data_dir: String,
    pub home_dir: String,
    pub tmp_dir: String,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct NativeParsedArgs {
    pub locale: Option<String>,
    pub user_data_dir: Option<String>,
    pub log_level: Option<String>,
    pub verbose: Option<bool>,
    pub app_name: Option<String>,
    #[napi(ts_type = "Record<string, string>")]
    pub extension_environment: Option<HashMap<String, String>>,
    // Add other args as needed
}

#[napi]
pub struct EnvironmentService {
    args: NativeParsedArgs,
    paths: NativeEnvironmentPaths,
    product_data_folder_name: String,
}

#[napi]
impl EnvironmentService {
    #[napi(constructor)]
    pub fn new(args: NativeParsedArgs, paths: NativeEnvironmentPaths, product_data_folder_name: String) -> Self {
        Self {
            args,
            paths,
            product_data_folder_name,
        }
    }

    #[napi(getter)]
    pub fn app_root(&self) -> String {
        // In a real implementation we might calculate this from the library path or pass it in.
        // For now, returning empty or we need to pass it in constructor.
        // VS Code does: dirname(FileAccess.asFileUri('').fsPath)
        String::new()
    }

    #[napi(getter)]
    pub fn user_home(&self) -> String {
        self.paths.home_dir.clone()
    }

    #[napi(getter)]
    pub fn user_data_path(&self) -> String {
        self.paths.user_data_dir.clone()
    }

    #[napi(getter)]
    pub fn app_settings_home(&self) -> String {
        let mut p = PathBuf::from(&self.paths.user_data_dir);
        p.push("User");
        p.to_string_lossy().to_string()
    }

    #[napi(getter)]
    pub fn tmp_dir(&self) -> String {
        self.paths.tmp_dir.clone()
    }

    #[napi(getter)]
    pub fn cache_home(&self) -> String {
        self.paths.user_data_dir.clone()
    }

    #[napi(getter)]
    pub fn state_resource(&self) -> String {
        let mut p = PathBuf::from(&self.app_settings_home());
        p.push("globalStorage");
        p.push("storage.json");
        p.to_string_lossy().to_string()
    }

    #[napi(getter)]
    pub fn user_roaming_data_home(&self) -> String {
        self.app_settings_home()
    }

    #[napi(getter)]
    pub fn user_data_sync_home(&self) -> String {
        let mut p = PathBuf::from(&self.app_settings_home());
        p.push("sync");
        p.to_string_lossy().to_string()
    }

    #[napi(getter)]
    pub fn logs_home(&self) -> String {
        // If args.logsPath is not set (we don't have it in struct yet), generate one?
        // For simplicity, just return a logs folder in user_data_dir
        let mut p = PathBuf::from(&self.paths.user_data_dir);
        p.push("logs");
        p.to_string_lossy().to_string()
    }

    #[napi(getter)]
    pub fn workspace_storage_home(&self) -> String {
        let mut p = PathBuf::from(&self.app_settings_home());
        p.push("workspaceStorage");
        p.to_string_lossy().to_string()
    }

    #[napi(getter)]
    pub fn local_history_home(&self) -> String {
        let mut p = PathBuf::from(&self.app_settings_home());
        p.push("History");
        p.to_string_lossy().to_string()
    }

    #[napi(getter)]
    pub fn keyboard_layout_resource(&self) -> String {
        let mut p = PathBuf::from(&self.user_roaming_data_home());
        p.push("keyboardLayout.json");
        p.to_string_lossy().to_string()
    }

    #[napi(getter)]
    pub fn argv_resource(&self) -> String {
        // Check VSCODE_PORTABLE env var?
        // Accessing env vars in Rust is easy.
        if let Ok(portable) = std::env::var("VSCODE_PORTABLE") {
            let mut p = PathBuf::from(portable);
            p.push("argv.json");
            return p.to_string_lossy().to_string();
        }
        let mut p = PathBuf::from(&self.paths.home_dir);
        p.push(&self.product_data_folder_name);
        p.push("argv.json");
        p.to_string_lossy().to_string()
    }
}
