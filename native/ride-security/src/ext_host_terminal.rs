use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Mutex;

#[napi(object)]
#[derive(Clone)]
pub struct ExtHostTerminalData {
    pub id: String,
    pub name: String,
    pub pid: Option<u32>,
}

#[napi]
pub struct ExtHostTerminal {
    terminals: Mutex<Vec<ExtHostTerminalData>>,
}

#[napi]
impl ExtHostTerminal {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            terminals: Mutex::new(Vec::new()),
        }
    }

    #[napi]
    pub fn add_terminal(&self, terminal: ExtHostTerminalData) {
        let mut t = self.terminals.lock().unwrap();
        t.push(terminal);
    }

    #[napi]
    pub fn remove_terminal(&self, id: String) -> bool {
        let mut t = self.terminals.lock().unwrap();
        let len_before = t.len();
        t.retain(|term| term.id != id);
        t.len() < len_before
    }
}
