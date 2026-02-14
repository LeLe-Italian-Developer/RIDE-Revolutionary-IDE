use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerminalInstance {
    pub id: String,
    pub name: String,
    pub shell_path: String,
    pub env: HashMap<String, String>,
    pub pid: Option<u32>,
}

#[napi]
pub struct TerminalEngine {
    instances: Mutex<HashMap<String, TerminalInstance>>,
}

#[napi]
impl TerminalEngine {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            instances: Mutex::new(HashMap::new()),
        }
    }

    #[napi]
    pub fn register_instance(&self, instance: TerminalInstance) {
        let mut instances = self.instances.lock().unwrap();
        instances.insert(instance.id.clone(), instance);
    }

    #[napi]
    pub fn unregister_instance(&self, id: String) -> bool {
        let mut instances = self.instances.lock().unwrap();
        instances.remove(&id).is_some()
    }

    #[napi]
    pub fn get_instance(&self, id: String) -> Option<TerminalInstance> {
        let instances = self.instances.lock().unwrap();
        instances.get(&id).cloned()
    }

    #[napi]
    pub fn list_instances(&self) -> Vec<TerminalInstance> {
        let instances = self.instances.lock().unwrap();
        instances.values().cloned().collect()
    }
}
