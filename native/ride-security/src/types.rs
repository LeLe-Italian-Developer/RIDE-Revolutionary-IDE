/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a disposable object.
#[napi]
pub struct Disposable {
    id: String,
}

#[napi]
impl Disposable {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
        }
    }

    #[napi]
    pub fn dispose(&self) -> bool {
        // In a real system, we'd trigger a native-to-JS disposal event.
        true
    }
}

/// A collection of disposables.
#[napi]
pub struct WorkbenchDisposableStore {
    disposables: Vec<String>,
}

#[napi]
impl WorkbenchDisposableStore {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self { disposables: Vec::new() }
    }

    #[napi]
    pub fn add(&mut self, d_id: String) {
        self.disposables.push(d_id);
    }

    #[napi]
    pub fn clear(&mut self) -> Vec<String> {
        self.disposables.drain(..).collect()
    }
}

/// Generic JSON value type for NAPI bridge
#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonValue {
    pub value_json: String,
}

/// Event interface for workbench events.
#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventData {
    pub type_name: String,
    pub payload_json: String,
}

/// Type-safe range for line-based buffers.
#[napi(object)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineRange {
    pub start_line_number: u32,
    pub end_line_number: u32,
}

impl LineRange {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start_line_number: start, end_line_number: end }
    }

    pub fn contains(&self, line: u32) -> bool {
        line >= self.start_line_number && line <= self.end_line_number
    }
}

/// Generic marker/problem information.
#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IMarkerData {
    pub code: Option<String>,
    pub severity: i32,
    pub message: String,
    pub source: Option<String>,
    pub start_line_number: u32,
    pub start_column: u32,
    pub end_line_number: u32,
    pub end_column: u32,
}
