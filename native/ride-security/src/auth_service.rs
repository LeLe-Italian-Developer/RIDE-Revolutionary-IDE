use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;
use std::sync::Mutex;

#[napi(object)]
#[derive(Clone, Debug)]
pub struct AuthenticationSession {
    pub id: String,
    pub access_token: String,
    pub account_label: String,
    pub scopes: Vec<String>,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct AuthProviderInfo {
    pub id: String,
    pub label: String,
}

#[napi]
pub struct AuthenticationService {
    providers: Mutex<HashMap<String, AuthProviderInfo>>,
    sessions: Mutex<HashMap<String, Vec<AuthenticationSession>>>, // provider_id -> sessions
}

#[napi]
impl AuthenticationService {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            providers: Mutex::new(HashMap::new()),
            sessions: Mutex::new(HashMap::new()),
        }
    }

    #[napi]
    pub fn register_provider(&self, provider: AuthProviderInfo) {
        let mut providers = self.providers.lock().unwrap();
        providers.insert(provider.id.clone(), provider);
    }

    #[napi]
    pub fn unregister_provider(&self, id: String) {
        let mut providers = self.providers.lock().unwrap();
        providers.remove(&id);
        
        let mut sessions = self.sessions.lock().unwrap();
        sessions.remove(&id);
    }

    #[napi]
    pub fn get_sessions(&self, provider_id: String) -> Vec<AuthenticationSession> {
        let sessions = self.sessions.lock().unwrap();
        sessions.get(&provider_id).cloned().unwrap_or_default()
    }

    #[napi]
    pub fn add_session(&self, provider_id: String, session: AuthenticationSession) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.entry(provider_id).or_default().push(session);
    }

    #[napi]
    pub fn remove_session(&self, provider_id: String, session_id: String) -> bool {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(s) = sessions.get_mut(&provider_id) {
            let len_before = s.len();
            s.retain(|sess| sess.id != session_id);
            return s.len() < len_before;
        }
        false
    }
    
    #[napi]
    pub fn get_provider_ids(&self) -> Vec<String> {
        let providers = self.providers.lock().unwrap();
        providers.keys().cloned().collect()
    }
}
