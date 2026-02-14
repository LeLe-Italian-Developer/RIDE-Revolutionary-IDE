/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! RIDE Extension API Type Definitions
//!
//! Provides Rust counterparts for all VS Code API structures:
//! - Diagnostics & Problems
//! - Hover & Tooltips
//! - Completion Items & Snippets
//! - Selection & Document States
//! - Document Links & Code Actions

use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiTextDocumentData {
    pub uri: String,
    pub language_id: String,
    pub version: i32,
    pub is_dirty: bool,
    pub text: String,
}

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiTextEditorData {
    pub id: String,
    pub document_uri: String,
    pub selections: Vec<SelectionData>,
    pub visible_ranges: Vec<RangeData>,
}

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelectionData {
    pub anchor_line: u32,
    pub anchor_column: u32,
    pub active_line: u32,
    pub active_column: u32,
}

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RangeData {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

#[napi]
#[derive(Debug, Serialize, Deserialize)]
pub enum EndOfLine {
    LF = 1,
    CRLF = 2,
}

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignatureHelp {
    pub signatures: Vec<SignatureInformation>,
    pub active_signature: u32,
    pub active_parameter: u32,
}

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignatureInformation {
    pub label: String,
    pub documentation: Option<String>,
    pub parameters: Vec<ParameterInformation>,
    pub active_parameter: Option<u32>,
}

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParameterInformation {
    pub label: String, // String or [num, num]
    pub documentation: Option<String>,
}

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodeAction {
    pub title: String,
    pub command: Option<Command>,
    pub edit: Option<WorkspaceEdit>,
    pub diagnostics: Option<Vec<DiagnosticData>>,
    pub is_preferred: Option<bool>,
}

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Command {
    pub title: String,
    pub command: String,
    pub arguments: Option<Vec<serde_json::Value>>,
}

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceEdit {
    pub changes: Option<HashMap<String, Vec<TextEditData>>>,
}

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextEditData {
    pub range: RangeData,
    pub new_text: String,
}

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiagnosticData {
    pub range: RangeData,
    pub message: String,
    pub severity: i32,
}

use std::collections::HashMap;
