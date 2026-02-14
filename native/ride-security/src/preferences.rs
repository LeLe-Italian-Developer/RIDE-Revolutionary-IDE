use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Mutex;

#[napi]
pub struct PreferencesService {
    user_settings_path: Mutex<String>,
}

#[napi]
impl PreferencesService {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            user_settings_path: Mutex::new(String::new()),
        }
    }

    #[napi]
    pub fn get_user_settings_path(&self) -> String {
        self.user_settings_path.lock().unwrap().clone()
    }

    #[napi]
    pub fn set_user_settings_path(&self, path: String) {
        let mut p = self.user_settings_path.lock().unwrap();
        *p = path;
    }
}
