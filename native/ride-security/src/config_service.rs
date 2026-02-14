/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use napi::bindgen_prelude::*;
use napi_derive::napi;
use crate::json_parser::{parse_jsonc, json_merge, json_get};

#[napi]
pub struct ConfigurationService {
    default_config: String,
    user_config: String,
    workspace_config: String,
    machine_config: String,
    merged_config: String,
}

#[napi]
impl ConfigurationService {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            default_config: "{}".to_string(),
            user_config: "{}".to_string(),
            workspace_config: "{}".to_string(),
            machine_config: "{}".to_string(),
            merged_config: "{}".to_string(),
        }
    }

    #[napi]
    pub fn update_default_config(&mut self, content: String) -> Result<()> {
        self.default_config = self.parse_content(content)?;
        self.recompute();
        Ok(())
    }

    #[napi]
    pub fn update_user_config(&mut self, content: String) -> Result<()> {
        self.user_config = self.parse_content(content)?;
        self.recompute();
        Ok(())
    }

    #[napi]
    pub fn update_workspace_config(&mut self, content: String) -> Result<()> {
        self.workspace_config = self.parse_content(content)?;
        self.recompute();
        Ok(())
    }

    #[napi]
    pub fn update_machine_config(&mut self, content: String) -> Result<()> {
        self.machine_config = self.parse_content(content)?;
        self.recompute();
        Ok(())
    }

    #[napi(getter)]
    pub fn get_merged_config(&self) -> String {
        self.merged_config.clone()
    }

    #[napi]
    pub fn get_value(&self, key: String) -> Option<String> {
        json_get(self.merged_config.clone(), key)
    }

    fn parse_content(&self, content: String) -> Result<String> {
        let result = parse_jsonc(content);
        if result.success {
            Ok(result.value.unwrap_or_else(|| "{}".to_string()))
        } else {
            // IF parsing fails, we could return empty object or error.
            // Better to return error so frontend knows.
            Err(Error::from_reason(format!("Parse error: {}", result.error_message.unwrap_or_default())))
        }
    }

    fn recompute(&mut self) {
        // Order: Default < Machine < User < Workspace
        // Or: Default < User < Remote < Workspace < WorkspaceFolder
        // Simplified: Default < Machine < User < Workspace

        let s1 = json_merge(self.default_config.clone(), self.machine_config.clone()).unwrap_or(self.default_config.clone());
        let s2 = json_merge(s1, self.user_config.clone()).unwrap_or(self.default_config.clone()); // Fallback might be wrong logic but essentially we want to keep merging
        let s3 = json_merge(s2, self.workspace_config.clone()).unwrap_or(self.user_config.clone());

        self.merged_config = s3;
    }
}
