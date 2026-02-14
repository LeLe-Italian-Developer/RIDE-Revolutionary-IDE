use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;

#[napi]
pub struct ConfigResolver {}

#[napi]
impl ConfigResolver {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {}
    }

    #[napi]
    pub fn resolve(&self, value: String, vars: HashMap<String, String>) -> String {
        let mut resolved = value;
        for (name, val) in vars {
            let pattern = format!("${{{}}}", name);
            resolved = resolved.replace(&pattern, &val);
        }
        resolved
    }
}
