use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;

#[napi(object)]
#[derive(Clone)]
pub struct ColorTheme {
    pub id: String,
    pub label: String,
    pub path: String,
    pub is_dark: bool,
}

#[napi]
pub struct ThemeService {
    current_theme_id: String,
    themes: HashMap<String, ColorTheme>,
}

#[napi]
impl ThemeService {
    #[napi(constructor)]
    pub fn new() -> Self {
        let mut themes = HashMap::new();
        // Default themes
        themes.insert("Default Dark Modern".to_string(), ColorTheme {
             id: "Default Dark Modern".to_string(),
             label: "Dark Modern".to_string(),
             path: "".to_string(),
             is_dark: true
        });
        themes.insert("Default Light Modern".to_string(), ColorTheme {
             id: "Default Light Modern".to_string(),
             label: "Light Modern".to_string(),
             path: "".to_string(),
             is_dark: false
        });
        
        Self {
            current_theme_id: "Default Dark Modern".to_string(),
            themes
        }
    }

    #[napi]
    pub fn set_theme(&mut self, theme_id: String) -> Result<()> {
        if self.themes.contains_key(&theme_id) {
            self.current_theme_id = theme_id;
            Ok(())
        } else {
             Err(Error::from_reason(format!("Theme not found: {}", theme_id)))
        }
    }

    #[napi]
    pub fn get_current_theme(&self) -> Option<ColorTheme> {
        self.themes.get(&self.current_theme_id).cloned()
    }
    
    #[napi]
    pub fn register_theme(&mut self, theme: ColorTheme) {
        self.themes.insert(theme.id.clone(), theme);
    }
}
