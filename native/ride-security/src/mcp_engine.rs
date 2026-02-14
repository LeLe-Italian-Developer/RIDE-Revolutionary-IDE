use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MCPServer {
    pub name: String,
    pub version: String,
    pub tools: Vec<String>,
}

#[napi]
pub struct MCPEngine {
    servers: Mutex<HashMap<String, MCPServer>>,
}

#[napi]
impl MCPEngine {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            servers: Mutex::new(HashMap::new()),
        }
    }

    #[napi]
    pub fn register_server(&self, server: MCPServer) {
        let mut servers = self.servers.lock().unwrap();
        servers.insert(server.name.clone(), server);
    }

    #[napi]
    pub fn unregister_server(&self, name: String) -> bool {
        let mut servers = self.servers.lock().unwrap();
        servers.remove(&name).is_some()
    }

    #[napi]
    pub fn get_server(&self, name: String) -> Option<MCPServer> {
        let servers = self.servers.lock().unwrap();
        servers.get(&name).cloned()
    }
}
