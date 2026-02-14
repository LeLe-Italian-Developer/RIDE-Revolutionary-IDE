use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Mutex;

#[napi(object)]
#[derive(Clone)]
pub struct ExtHostDebugSession {
    pub id: String,
    pub kind: String,
    pub name: String,
}

#[napi]
pub struct ExtHostDebug {
    sessions: Mutex<Vec<ExtHostDebugSession>>,
}

#[napi]
impl ExtHostDebug {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(Vec::new()),
        }
    }

    #[napi]
    pub fn add_session(&self, session: ExtHostDebugSession) {
        let mut s = self.sessions.lock().unwrap();
        s.push(session);
    }

    #[napi]
    pub fn remove_session(&self, id: String) -> bool {
        let mut s = self.sessions.lock().unwrap();
        let len_before = s.len();
        s.retain(|sess| sess.id != id);
        s.len() < len_before
    }
}
