/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Marshalling & serialization utilities — Rust port of `src/vs/base/common/marshalling.ts`.
//! Handles serialization, deserialization, and data conversion between formats.

use napi_derive::napi;
use napi::bindgen_prelude::*;
use serde_json::Value;

/// Serialize a JSON value to MessagePack format.
#[napi]
pub fn json_to_msgpack(json: String) -> Result<Buffer> {
    let v: Value = serde_json::from_str(&json).map_err(|e| Error::from_reason(e.to_string()))?;
    let bytes = rmp_serde::to_vec(&v).map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(Buffer::from(bytes))
}

/// Deserialize MessagePack bytes to JSON string.
#[napi]
pub fn msgpack_to_json(data: Buffer) -> Result<String> {
    let v: Value = rmp_serde::from_slice(data.as_ref()).map_err(|e| Error::from_reason(e.to_string()))?;
    serde_json::to_string(&v).map_err(|e| Error::from_reason(e.to_string()))
}

/// Convert JSON to TOML format.
#[napi]
pub fn json_to_toml(json: String) -> Result<String> {
    let v: toml::Value = serde_json::from_str::<Value>(&json)
        .map_err(|e| Error::from_reason(e.to_string()))
        .and_then(|jv| json_value_to_toml(jv))?;
    toml::to_string_pretty(&v).map_err(|e| Error::from_reason(e.to_string()))
}

/// Convert TOML to JSON format.
#[napi]
pub fn toml_to_json(toml_str: String) -> Result<String> {
    let v: toml::Value = toml::from_str(&toml_str).map_err(|e| Error::from_reason(e.to_string()))?;
    let jv = toml_value_to_json(v);
    serde_json::to_string_pretty(&jv).map_err(|e| Error::from_reason(e.to_string()))
}

fn json_value_to_toml(v: Value) -> Result<toml::Value> {
    match v {
        Value::Null => Ok(toml::Value::String("null".into())),
        Value::Bool(b) => Ok(toml::Value::Boolean(b)),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() { Ok(toml::Value::Integer(i)) }
            else if let Some(f) = n.as_f64() { Ok(toml::Value::Float(f)) }
            else { Ok(toml::Value::String(n.to_string())) }
        }
        Value::String(s) => Ok(toml::Value::String(s)),
        Value::Array(arr) => {
            let items: Result<Vec<toml::Value>> = arr.into_iter().map(json_value_to_toml).collect();
            Ok(toml::Value::Array(items?))
        }
        Value::Object(map) => {
            let mut table = toml::map::Map::new();
            for (k, v) in map { table.insert(k, json_value_to_toml(v)?); }
            Ok(toml::Value::Table(table))
        }
    }
}

fn toml_value_to_json(v: toml::Value) -> Value {
    match v {
        toml::Value::Boolean(b) => Value::Bool(b),
        toml::Value::Integer(i) => Value::Number(serde_json::Number::from(i)),
        toml::Value::Float(f) => serde_json::Number::from_f64(f).map(Value::Number).unwrap_or(Value::Null),
        toml::Value::String(s) => Value::String(s),
        toml::Value::Array(arr) => Value::Array(arr.into_iter().map(toml_value_to_json).collect()),
        toml::Value::Table(map) => {
            let obj: serde_json::Map<String, Value> = map.into_iter().map(|(k, v)| (k, toml_value_to_json(v))).collect();
            Value::Object(obj)
        }
        toml::Value::Datetime(dt) => Value::String(dt.to_string()),
    }
}

/// Compute a checksum (CRC32) of a buffer.
#[napi]
pub fn crc32(data: Buffer) -> u32 {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(data.as_ref());
    hasher.finalize()
}

/// Compute CRC32 of a string.
#[napi]
pub fn crc32_string(data: String) -> u32 {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(data.as_bytes());
    hasher.finalize()
}

/// Variable-length quantity (VLQ) encode a number.
#[napi]
pub fn vlq_encode(mut value: i32) -> Vec<u32> {
    let mut result = Vec::new();
    let negative = value < 0;
    value = value.unsigned_abs() as i32;
    let mut first = true;
    loop {
        let mut digit = (value & 0x1F) as u32;
        value >>= 5;
        if first { digit = (digit << 1) | if negative { 1 } else { 0 }; first = false; }
        if value > 0 { digit |= 0x20; }
        result.push(digit);
        if value == 0 { break; }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_msgpack_roundtrip() {
        let json = r#"{"a": 1, "b": "hello"}"#.to_string();
        let msgpack = json_to_msgpack(json.clone()).unwrap();
        let back = msgpack_to_json(msgpack).unwrap();
        let v1: Value = serde_json::from_str(&json).unwrap();
        let v2: Value = serde_json::from_str(&back).unwrap();
        assert_eq!(v1, v2);
    }
    #[test]
    fn test_crc32() {
        let c = crc32_string("hello".into());
        assert!(c > 0);
        assert_eq!(c, crc32_string("hello".into()));
    }
    #[test]
    fn test_toml_roundtrip() {
        let json = r#"{"name": "test", "version": 1}"#.to_string();
        let t = json_to_toml(json.clone()).unwrap();
        let back = toml_to_json(t).unwrap();
        let v1: Value = serde_json::from_str(&json).unwrap();
        let v2: Value = serde_json::from_str(&back).unwrap();
        assert_eq!(v1, v2);
    }
}
