use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Mutex;

#[napi(object)]
#[derive(Clone)]
pub struct UserProfile {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

#[napi]
pub struct UserProfileService {
    current_profile_id: Mutex<String>,
    profiles: Mutex<Vec<UserProfile>>,
}

#[napi]
impl UserProfileService {
    #[napi(constructor)]
    pub fn new() -> Self {
        let default_profile = UserProfile {
            id: "default".to_string(),
            name: "Default".to_string(),
            is_default: true,
        };
        Self {
            current_profile_id: Mutex::new("default".to_string()),
            profiles: Mutex::new(vec![default_profile]),
        }
    }

    #[napi]
    pub fn get_current_profile(&self) -> Option<UserProfile> {
        let id = self.current_profile_id.lock().unwrap();
        let profiles = self.profiles.lock().unwrap();
        profiles.iter().find(|p| p.id == *id).cloned()
    }

    #[napi]
    pub fn set_current_profile(&self, id: String) -> bool {
        let profiles = self.profiles.lock().unwrap();
        if profiles.iter().any(|p| p.id == id) {
            let mut curr = self.current_profile_id.lock().unwrap();
            *curr = id;
            return true;
        }
        false
    }

    #[napi]
    pub fn get_all_profiles(&self) -> Vec<UserProfile> {
        self.profiles.lock().unwrap().clone()
    }
}
