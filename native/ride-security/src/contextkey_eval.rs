use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextKeyExpression {
    pub key: String,
    pub value: Option<String>,
    pub operator: String, // "==", "!=", "in", "notIn", "RegexMatches"
}

#[napi]
pub struct ContextKeyService {
    context: HashMap<String, String>,
}

#[napi]
impl ContextKeyService {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            context: HashMap::new(),
        }
    }

    #[napi]
    pub fn set_context(&mut self, key: String, value: String) {
        self.context.insert(key, value);
    }

    #[napi]
    pub fn get_context_value(&self, key: String) -> Option<String> {
        self.context.get(&key).cloned()
    }

    #[napi]
    pub fn evaluate(&self, expression: ContextKeyExpression) -> bool {
        let current_val = self.context.get(&expression.key);
        
        match expression.operator.as_str() {
            "==" => {
                if let Some(target) = expression.value {
                    current_val.map(|v| v == &target).unwrap_or(false)
                } else {
                    current_val.is_none()
                }
            },
            "!=" => {
                if let Some(target) = expression.value {
                    current_val.map(|v| v != &target).unwrap_or(true)
                } else {
                    current_val.is_some()
                }
            },
            "in" => {
                if let (Some(cv), Some(target)) = (current_val, expression.value) {
                    target.contains(cv)
                } else {
                    false
                }
            },
            "RegexMatches" => {
                if let (Some(cv), Some(pattern)) = (current_val, expression.value) {
                    if let Ok(re) = regex::Regex::new(&pattern) {
                        re.is_match(cv)
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            _ => false
        }
    }
}
