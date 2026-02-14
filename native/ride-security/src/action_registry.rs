use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;
use std::sync::Mutex;

#[napi(object)]
#[derive(Clone)]
pub struct Action {
    pub id: String,
    pub label: String,
    pub tooltip: Option<String>,
    pub class: Option<String>,
    pub enabled: bool,
    pub checked: Option<bool>,
}

#[napi]
pub struct ActionRegistry {
    actions: Mutex<HashMap<String, Action>>,
}

#[napi]
impl ActionRegistry {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            actions: Mutex::new(HashMap::new()),
        }
    }

    #[napi]
    pub fn register_action(&self, action: Action) -> bool {
        let mut actions = self.actions.lock().unwrap();
        if actions.contains_key(&action.id) {
            return false;
        }
        actions.insert(action.id.clone(), action);
        true
    }

    #[napi]
    pub fn unregister_action(&self, id: String) -> bool {
        let mut actions = self.actions.lock().unwrap();
        actions.remove(&id).is_some()
    }

    #[napi]
    pub fn get_action(&self, id: String) -> Option<Action> {
        let actions = self.actions.lock().unwrap();
        actions.get(&id).cloned()
    }

    #[napi]
    pub fn set_enabled(&self, id: String, enabled: bool) -> bool {
        let mut actions = self.actions.lock().unwrap();
        if let Some(action) = actions.get_mut(&id) {
            action.enabled = enabled;
            return true;
        }
        false
    }
}
