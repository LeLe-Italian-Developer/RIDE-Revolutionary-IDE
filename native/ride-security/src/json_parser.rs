/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! JSON parsing, editing, and schema validation — Rust port of
//! `src/vs/base/common/json.ts`, `jsonSchema.ts`, and `jsonEdit.ts`.

use napi_derive::napi;
use napi::bindgen_prelude::*;
use serde_json::Value;

// ─── JSON parsing ──────────────────────────────────────────────────────────

/// Parse JSON with detailed error reporting.
#[napi(object)]
pub struct JsonParseResult {
    pub success: bool,
    pub value: Option<String>,
    pub error_message: Option<String>,
    pub error_offset: Option<u32>,
    pub error_line: Option<u32>,
    pub error_column: Option<u32>,
}

/// Parse a JSON string, returning structured result with error details.
#[napi]
pub fn parse_json(text: String) -> JsonParseResult {
    // Strip BOM if present
    let clean = text.strip_prefix('\u{FEFF}').unwrap_or(&text);

    match serde_json::from_str::<Value>(clean) {
        Ok(v) => JsonParseResult {
            success: true,
            value: Some(v.to_string()),
            error_message: None,
            error_offset: None,
            error_line: None,
            error_column: None,
        },
        Err(e) => JsonParseResult {
            success: false,
            value: None,
            error_message: Some(e.to_string()),
            error_offset: None,
            error_line: Some(e.line() as u32),
            error_column: Some(e.column() as u32),
        },
    }
}

/// Parse JSON with comments (JSONC) — strips single-line and multi-line comments.
#[napi]
pub fn parse_jsonc(text: String) -> JsonParseResult {
    let stripped = strip_json_comments(text);
    parse_json(stripped)
}

/// Strip comments from a JSONC string.
#[napi]
pub fn strip_json_comments(text: String) -> String {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut result = String::with_capacity(len);
    let mut i = 0;
    let mut in_string = false;
    let mut escape_next = false;

    while i < len {
        if escape_next {
            result.push(chars[i]);
            escape_next = false;
            i += 1;
            continue;
        }

        if in_string {
            if chars[i] == '\\' {
                escape_next = true;
                result.push(chars[i]);
            } else if chars[i] == '"' {
                in_string = false;
                result.push(chars[i]);
            } else {
                result.push(chars[i]);
            }
            i += 1;
            continue;
        }

        if chars[i] == '"' {
            in_string = true;
            result.push(chars[i]);
            i += 1;
            continue;
        }

        // Single-line comment
        if i + 1 < len && chars[i] == '/' && chars[i + 1] == '/' {
            while i < len && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        // Multi-line comment
        if i + 1 < len && chars[i] == '/' && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < len && !(chars[i] == '*' && chars[i + 1] == '/') {
                if chars[i] == '\n' {
                    result.push('\n'); // Preserve line numbers
                }
                i += 1;
            }
            i += 2; // Skip */
            continue;
        }

        // Trailing comma removal (before } or ])
        if chars[i] == ',' {
            // Look ahead for } or ]
            let mut j = i + 1;
            while j < len && chars[j].is_whitespace() {
                j += 1;
            }
            if j < len && (chars[j] == '}' || chars[j] == ']') {
                // Skip trailing comma
                i += 1;
                continue;
            }
        }

        result.push(chars[i]);
        i += 1;
    }

    result
}

// ─── JSON path queries ─────────────────────────────────────────────────────

/// Get a value from a JSON object by dot-notation path (e.g., "a.b.c").
#[napi]
pub fn json_get(json_string: String, path: String) -> Option<String> {
    let value: Value = serde_json::from_str(&json_string).ok()?;
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = &value;

    for part in parts {
        // Try as object key first
        if let Some(v) = current.get(part) {
            current = v;
        } else if let Ok(idx) = part.parse::<usize>() {
            // Try as array index
            if let Some(v) = current.get(idx) {
                current = v;
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    Some(current.to_string())
}

/// Set a value in a JSON object by dot-notation path.
#[napi]
pub fn json_set(json_string: String, path: String, value_string: String) -> Result<String> {
    let mut root: Value = serde_json::from_str(&json_string)
        .map_err(|e| Error::from_reason(format!("Invalid JSON: {}", e)))?;
    let new_value: Value = serde_json::from_str(&value_string)
        .unwrap_or(Value::String(value_string.clone()));

    let parts: Vec<&str> = path.split('.').collect();
    let mut current = &mut root;

    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Set the value
            if let Some(obj) = current.as_object_mut() {
                obj.insert(part.to_string(), new_value.clone());
            } else if let Ok(idx) = part.parse::<usize>() {
                if let Some(arr) = current.as_array_mut() {
                    while arr.len() <= idx {
                        arr.push(Value::Null);
                    }
                    arr[idx] = new_value.clone();
                }
            }
        } else {
            // Navigate
            if let Ok(idx) = part.parse::<usize>() {
                if !current.is_array() {
                    *current = Value::Array(Vec::new());
                }
                let arr = current.as_array_mut().unwrap();
                while arr.len() <= idx {
                    arr.push(Value::Object(serde_json::Map::new()));
                }
                current = &mut arr[idx];
            } else {
                if !current.is_object() {
                    *current = Value::Object(serde_json::Map::new());
                }
                let obj = current.as_object_mut().unwrap();
                if !obj.contains_key(*part) {
                    obj.insert(part.to_string(), Value::Object(serde_json::Map::new()));
                }
                current = obj.get_mut(*part).unwrap();
            }
        }
    }

    serde_json::to_string_pretty(&root)
        .map_err(|e| Error::from_reason(format!("Serialization failed: {}", e)))
}

/// Delete a key from a JSON object by dot-notation path.
#[napi]
pub fn json_delete(json_string: String, path: String) -> Result<String> {
    let mut root: Value = serde_json::from_str(&json_string)
        .map_err(|e| Error::from_reason(format!("Invalid JSON: {}", e)))?;

    let parts: Vec<&str> = path.split('.').collect();
    let mut current = &mut root;

    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            if let Some(obj) = current.as_object_mut() {
                obj.remove(*part);
            }
        } else {
            if let Some(v) = current.get_mut(*part) {
                current = v;
            } else {
                break;
            }
        }
    }

    serde_json::to_string_pretty(&root)
        .map_err(|e| Error::from_reason(format!("Serialization failed: {}", e)))
}

/// Check if a JSON object has a key at the given path.
#[napi]
pub fn json_has(json_string: String, path: String) -> bool {
    json_get(json_string, path).is_some()
}

// ─── JSON merging ──────────────────────────────────────────────────────────

/// Deep merge two JSON objects. The second object's values override the first's.
#[napi]
pub fn json_merge(base_json: String, override_json: String) -> Result<String> {
    let mut base: Value = serde_json::from_str(&base_json)
        .map_err(|e| Error::from_reason(format!("Invalid base JSON: {}", e)))?;
    let over: Value = serde_json::from_str(&override_json)
        .map_err(|e| Error::from_reason(format!("Invalid override JSON: {}", e)))?;

    deep_merge(&mut base, &over);

    serde_json::to_string_pretty(&base)
        .map_err(|e| Error::from_reason(format!("Serialization failed: {}", e)))
}

fn deep_merge(base: &mut Value, over: &Value) {
    match (base, over) {
        (Value::Object(base_map), Value::Object(over_map)) => {
            for (key, over_val) in over_map {
                if let Some(base_val) = base_map.get_mut(key) {
                    deep_merge(base_val, over_val);
                } else {
                    base_map.insert(key.clone(), over_val.clone());
                }
            }
        }
        (base, over) => {
            *base = over.clone();
        }
    }
}

// ─── JSON formatting ──────────────────────────────────────────────────────

/// Pretty-print a JSON string with configurable indentation.
#[napi]
pub fn json_format(json_string: String, indent: Option<u32>) -> Result<String> {
    let value: Value = serde_json::from_str(&json_string)
        .map_err(|e| Error::from_reason(format!("Invalid JSON: {}", e)))?;

    let indent_size = indent.unwrap_or(2) as usize;
    format_value(&value, 0, indent_size)
        .map_err(|e| Error::from_reason(format!("Format failed: {}", e)))
}

fn format_value(value: &Value, depth: usize, indent: usize) -> std::result::Result<String, String> {
    match value {
        Value::Object(map) if map.is_empty() => Ok("{}".to_string()),
        Value::Array(arr) if arr.is_empty() => Ok("[]".to_string()),
        Value::Object(map) => {
            let prefix = " ".repeat((depth + 1) * indent);
            let close_prefix = " ".repeat(depth * indent);
            let entries: Vec<String> = map
                .iter()
                .map(|(k, v)| {
                    let formatted_val = format_value(v, depth + 1, indent)?;
                    Ok(format!("{}\"{}\": {}", prefix, k, formatted_val))
                })
                .collect::<std::result::Result<Vec<_>, String>>()?;
            Ok(format!("{{\n{}\n{}}}", entries.join(",\n"), close_prefix))
        }
        Value::Array(arr) => {
            let prefix = " ".repeat((depth + 1) * indent);
            let close_prefix = " ".repeat(depth * indent);
            let entries: Vec<String> = arr
                .iter()
                .map(|v| {
                    let formatted_val = format_value(v, depth + 1, indent)?;
                    Ok(format!("{}{}", prefix, formatted_val))
                })
                .collect::<std::result::Result<Vec<_>, String>>()?;
            Ok(format!("[\n{}\n{}]", entries.join(",\n"), close_prefix))
        }
        _ => Ok(value.to_string()),
    }
}

/// Minify a JSON string (remove all whitespace).
#[napi]
pub fn json_minify(json_string: String) -> Result<String> {
    let value: Value = serde_json::from_str(&json_string)
        .map_err(|e| Error::from_reason(format!("Invalid JSON: {}", e)))?;
    serde_json::to_string(&value)
        .map_err(|e| Error::from_reason(format!("Serialization failed: {}", e)))
}

// ─── JSON validation ──────────────────────────────────────────────────────

/// Validate that a string is valid JSON.
#[napi]
pub fn is_valid_json(text: String) -> bool {
    serde_json::from_str::<Value>(&text).is_ok()
}

/// Validate that a string is valid JSONC (JSON with comments).
#[napi]
pub fn is_valid_jsonc(text: String) -> bool {
    let stripped = strip_json_comments(text);
    is_valid_json(stripped)
}

/// Get the type of a JSON value ("object", "array", "string", "number", "boolean", "null").
#[napi]
pub fn json_type_of(json_string: String) -> String {
    match serde_json::from_str::<Value>(&json_string) {
        Ok(Value::Object(_)) => "object".to_string(),
        Ok(Value::Array(_)) => "array".to_string(),
        Ok(Value::String(_)) => "string".to_string(),
        Ok(Value::Number(_)) => "number".to_string(),
        Ok(Value::Bool(_)) => "boolean".to_string(),
        Ok(Value::Null) => "null".to_string(),
        Err(_) => "invalid".to_string(),
    }
}

/// Count the number of keys in a JSON object (top level only).
#[napi]
pub fn json_key_count(json_string: String) -> u32 {
    match serde_json::from_str::<Value>(&json_string) {
        Ok(Value::Object(map)) => map.len() as u32,
        _ => 0,
    }
}

/// Get all keys from a JSON object (top level only).
#[napi]
pub fn json_keys(json_string: String) -> Vec<String> {
    match serde_json::from_str::<Value>(&json_string) {
        Ok(Value::Object(map)) => map.keys().cloned().collect(),
        _ => Vec::new(),
    }
}

/// Flatten a nested JSON object to dot-notation keys.
#[napi]
pub fn json_flatten(json_string: String) -> Result<String> {
    let value: Value = serde_json::from_str(&json_string)
        .map_err(|e| Error::from_reason(format!("Invalid JSON: {}", e)))?;

    let mut flat = serde_json::Map::new();
    flatten_value(&value, String::new(), &mut flat);

    serde_json::to_string_pretty(&Value::Object(flat))
        .map_err(|e| Error::from_reason(format!("Serialization failed: {}", e)))
}

fn flatten_value(value: &Value, prefix: String, result: &mut serde_json::Map<String, Value>) {
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                let new_key = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                flatten_value(val, new_key, result);
            }
        }
        _ => {
            result.insert(prefix, value.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json() {
        let result = parse_json(r#"{"a": 1, "b": "hello"}"#.into());
        assert!(result.success);
    }

    #[test]
    fn test_parse_json_error() {
        let result = parse_json("{invalid}".into());
        assert!(!result.success);
        assert!(result.error_message.is_some());
    }

    #[test]
    fn test_strip_comments() {
        let input = r#"{
            // comment
            "key": "value", /* block */
            "key2": 42,
        }"#;
        let stripped = strip_json_comments(input.into());
        assert!(!stripped.contains("//"));
        assert!(!stripped.contains("/*"));
        assert!(is_valid_json(stripped));
    }

    #[test]
    fn test_json_get() {
        let json = r#"{"a": {"b": {"c": 42}}}"#;
        assert_eq!(json_get(json.into(), "a.b.c".into()), Some("42".into()));
    }

    #[test]
    fn test_json_set() {
        let json = r#"{"a": 1}"#;
        let result = json_set(json.into(), "b".into(), "2".into()).unwrap();
        assert!(result.contains("\"b\": 2"));
    }

    #[test]
    fn test_json_merge() {
        let base = r#"{"a": 1, "b": {"c": 2}}"#;
        let over = r#"{"b": {"d": 3}, "e": 4}"#;
        let merged = json_merge(base.into(), over.into()).unwrap();
        let v: Value = serde_json::from_str(&merged).unwrap();
        assert_eq!(v["a"], 1);
        assert_eq!(v["b"]["c"], 2);
        assert_eq!(v["b"]["d"], 3);
        assert_eq!(v["e"], 4);
    }

    #[test]
    fn test_json_flatten() {
        let json = r#"{"a": {"b": 1, "c": {"d": 2}}}"#;
        let flat = json_flatten(json.into()).unwrap();
        let v: Value = serde_json::from_str(&flat).unwrap();
        assert_eq!(v["a.b"], 1);
        assert_eq!(v["a.c.d"], 2);
    }
}
