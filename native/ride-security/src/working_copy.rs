/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkingCopyInfo {
    pub uri: String,
    pub is_dirty: bool,
    pub backup_path: Option<String>,
    pub mtime: f64,
}

#[napi]
pub struct WorkingCopyManager {
    copies: Mutex<HashMap<String, WorkingCopyInfo>>,
}

#[napi]
impl WorkingCopyManager {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            copies: Mutex::new(HashMap::new()),
        }
    }

    #[napi]
    pub fn set_dirty(&self, uri: String, is_dirty: bool, backup_path: Option<String>) {
        let mut copies = self.copies.lock().unwrap();
        let info = copies.entry(uri.clone()).or_insert_with(|| WorkingCopyInfo {
            uri: uri.clone(),
            is_dirty,
            backup_path: backup_path.clone(),
            mtime: chrono::Utc::now().timestamp_millis() as f64,
        });
        info.is_dirty = is_dirty;
        info.backup_path = backup_path;
        info.mtime = chrono::Utc::now().timestamp_millis() as f64;
    }

    #[napi]
    pub fn get_dirty_copies(&self) -> Vec<WorkingCopyInfo> {
        let copies = self.copies.lock().unwrap();
        copies.values()
            .filter(|c| c.is_dirty)
            .cloned()
            .collect()
    }

    #[napi]
    pub fn remove_copy(&self, uri: String) -> bool {
        let mut copies = self.copies.lock().unwrap();
        copies.remove(&uri).is_some()
    }
}
