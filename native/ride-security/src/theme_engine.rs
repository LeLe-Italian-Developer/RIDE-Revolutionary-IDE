use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;

#[napi]
pub struct ThemeEngine {
    token_colors: HashMap<String, String>,
}

#[napi]
impl ThemeEngine {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            token_colors: HashMap::new(),
        }
    }

    #[napi]
    pub fn set_color(&mut self, token: String, color: String) {
        self.token_colors.insert(token, color);
    }

    #[napi]
    pub fn get_color(&self, token: String) -> Option<String> {
        self.token_colors.get(&token).cloned()
    }
}
