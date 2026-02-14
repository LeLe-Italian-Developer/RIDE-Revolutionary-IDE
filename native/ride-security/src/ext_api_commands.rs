/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;
use std::sync::Mutex;
use serde_json::Value;

#[napi(object)]
#[derive(Clone)]
pub struct CommandInfo {
    pub id: String,
    pub title: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
}

#[napi]
pub struct ExtApiCommands {
    commands: Mutex<HashMap<String, CommandInfo>>,
}

#[napi]
impl ExtApiCommands {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            commands: Mutex::new(HashMap::new()),
        }
    }

    #[napi]
    pub fn register_command(&self, info: CommandInfo) -> bool {
        let mut cmds = self.commands.lock().unwrap();
        if cmds.contains_key(&info.id) {
            return false;
        }
        cmds.insert(info.id.clone(), info);
        true
    }

    #[napi]
    pub fn unregister_command(&self, id: String) -> bool {
        let mut cmds = self.commands.lock().unwrap();
        cmds.remove(&id).is_some()
    }

    #[napi]
    pub fn get_commands(&self) -> Vec<CommandInfo> {
        let cmds = self.commands.lock().unwrap();
        cmds.values().cloned().collect()
    }

    #[napi]
    pub fn execute_command(&self, id: String, _args_json: String) -> Result<String> {
        let cmds = self.commands.lock().unwrap();
        if !cmds.contains_key(&id) {
            return Err(napi::Error::from_reason(format!("Command '{}' not found", id)));
        }

        // In reality, this would trigger a callback to the JS Extension Host
        // or execute a built-in native command.
        Ok(format!("Command '{}' executed", id))
    }

    #[napi]
    pub fn has_command(&self, id: String) -> bool {
        let cmds = self.commands.lock().unwrap();
        cmds.contains_key(&id)
    }
}
