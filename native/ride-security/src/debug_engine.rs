use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use crate::range::Range;

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Breakpoint {
    pub id: String,
    pub uri: String,
    pub range: Range,
    pub enabled: bool,
    pub condition: Option<String>,
}

#[napi]
pub struct DebugEngine {
    breakpoints: Mutex<HashMap<String, Vec<Breakpoint>>>, // URI -> Breakpoints
}

#[napi]
impl DebugEngine {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            breakpoints: Mutex::new(HashMap::new()),
        }
    }

    #[napi]
    pub fn add_breakpoint(&self, bp: Breakpoint) {
        let mut bps = self.breakpoints.lock().unwrap();
        bps.entry(bp.uri.clone()).or_insert_with(Vec::new).push(bp);
    }

    #[napi]
    pub fn remove_breakpoint(&self, uri: String, id: String) -> bool {
        let mut bps = self.breakpoints.lock().unwrap();
        if let Some(list) = bps.get_mut(&uri) {
            let len_before = list.len();
            list.retain(|bp| bp.id != id);
            return list.len() < len_before;
        }
        false
    }

    #[napi]
    pub fn get_breakpoints(&self, uri: String) -> Vec<Breakpoint> {
        let bps = self.breakpoints.lock().unwrap();
        bps.get(&uri).cloned().unwrap_or_default()
    }
}
