/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Type utilities — Rust port of `src/vs/base/common/types.ts`, `objects.ts`.
//! Runtime type checking, deep comparison, deep clone, and object utilities.

use napi_derive::napi;
use napi::bindgen_prelude::*;
use serde_json::Value;

#[napi]
pub fn is_string(_value: String) -> bool { true }
#[napi]
pub fn is_number(value: String) -> bool { value.parse::<f64>().is_ok() }
#[napi]
pub fn is_boolean(value: String) -> bool { value == "true" || value == "false" }
#[napi]
pub fn is_null_or_empty(value: Option<String>) -> bool { value.map_or(true, |v| v.is_empty()) }

/// Deep-equal two JSON values.
#[napi]
pub fn deep_equal(a: String, b: String) -> bool {
    let va = serde_json::from_str::<Value>(&a);
    let vb = serde_json::from_str::<Value>(&b);
    match (va, vb) { (Ok(x), Ok(y)) => x == y, _ => a == b }
}

/// Deep-clone a JSON value (returns a new independent copy).
#[napi]
pub fn deep_clone(json: String) -> Result<String> {
    let v: Value = serde_json::from_str(&json).map_err(|e| Error::from_reason(e.to_string()))?;
    serde_json::to_string(&v).map_err(|e| Error::from_reason(e.to_string()))
}

/// Deeply freeze / make immutable snapshot of a JSON object.
#[napi]
pub fn deep_freeze(json: String) -> Result<String> { deep_clone(json) }

/// Pick specific keys from a JSON object.
#[napi]
pub fn pick_keys(json: String, keys: Vec<String>) -> Result<String> {
    let v: Value = serde_json::from_str(&json).map_err(|e| Error::from_reason(e.to_string()))?;
    if let Value::Object(map) = v {
        let picked: serde_json::Map<String, Value> = map.into_iter()
            .filter(|(k, _)| keys.contains(k))
            .collect();
        serde_json::to_string(&Value::Object(picked)).map_err(|e| Error::from_reason(e.to_string()))
    } else {
        Ok(json)
    }
}

/// Omit specific keys from a JSON object.
#[napi]
pub fn omit_keys(json: String, keys: Vec<String>) -> Result<String> {
    let v: Value = serde_json::from_str(&json).map_err(|e| Error::from_reason(e.to_string()))?;
    if let Value::Object(map) = v {
        let omitted: serde_json::Map<String, Value> = map.into_iter()
            .filter(|(k, _)| !keys.contains(k))
            .collect();
        serde_json::to_string(&Value::Object(omitted)).map_err(|e| Error::from_reason(e.to_string()))
    } else {
        Ok(json)
    }
}

/// Mixin / object.assign — merge multiple JSON objects (last wins).
#[napi]
pub fn mixin(json_objects: Vec<String>) -> Result<String> {
    let mut result = serde_json::Map::new();
    for json in json_objects {
        if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(&json) {
            for (k, v) in map { result.insert(k, v); }
        }
    }
    serde_json::to_string(&Value::Object(result)).map_err(|e| Error::from_reason(e.to_string()))
}

/// Get nested value count in a JSON object (recursively).
#[napi]
pub fn count_values(json: String) -> u32 {
    fn count(v: &Value) -> u32 {
        match v {
            Value::Object(m) => m.values().map(count).sum::<u32>() + m.len() as u32,
            Value::Array(a) => a.iter().map(count).sum::<u32>() + a.len() as u32,
            _ => 1,
        }
    }
    serde_json::from_str::<Value>(&json).map(|v| count(&v)).unwrap_or(0)
}

/// Compute size of a JSON value in approximate bytes.
#[napi]
pub fn json_size_bytes(json: String) -> u32 { json.len() as u32 }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_deep_equal() {
        assert!(deep_equal(r#"{"a":1}"#.into(), r#"{"a":1}"#.into()));
        assert!(!deep_equal(r#"{"a":1}"#.into(), r#"{"a":2}"#.into()));
    }
    #[test]
    fn test_pick_omit() {
        let json = r#"{"a":1,"b":2,"c":3}"#.to_string();
        let picked = pick_keys(json.clone(), vec!["a".into(), "c".into()]).unwrap();
        assert!(picked.contains("\"a\""));
        assert!(!picked.contains("\"b\""));
    }
}
