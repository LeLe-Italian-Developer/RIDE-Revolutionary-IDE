/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! RIDE Polyglot Notebook Engine v2
//!
//! Features:
//! - Hybrid Cell Model (Markup, Code, Interactive)
//! - Complex Output Management (Mime-aware outputs, multiple items)
//! - Cell Execution State Machine with metadata and timing
//! - Incremental Notebook Serialization
//! - Resource reference tracking for notebook assets

use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[napi]
#[derive(Debug, Serialize, Deserialize)]
pub enum NotebookCellKind {
    Markup = 1,
    Code = 2,
    Interactive = 3,
}

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotebookCellOutputItem {
    pub mime: String,
    pub data: Vec<u8>,
}

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotebookCellOutput {
    pub id: String,
    pub items: Vec<NotebookCellOutputItem>,
    pub metadata: Option<HashMap<String, String>>,
}

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotebookCell {
    pub id: String,
    pub kind: i32, // NotebookCellKind
    pub content: String,
    pub language: String,
    pub outputs: Vec<NotebookCellOutput>,
    pub execution_summary: Option<NotebookExecutionSummary>,
    pub is_dirty: bool,
}

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotebookExecutionSummary {
    pub execution_order: u32,
    pub success: bool,
    pub start_time: f64,
    pub duration_ms: f64,
}

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotebookData {
    pub uri: String,
    pub cells: Vec<NotebookCell>,
    pub metadata: HashMap<String, String>,
    pub version: u32,
}

#[napi]
pub struct NotebookEngine {
    notebooks: Mutex<HashMap<String, NotebookData>>,
}

#[napi]
impl NotebookEngine {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            notebooks: Mutex::new(HashMap::new()),
        }
    }

    #[napi]
    pub fn open_notebook(&self, uri: String, data: NotebookData) {
        let mut notebooks = self.notebooks.lock().unwrap();
        notebooks.insert(uri, data);
    }

    #[napi]
    pub fn update_cell_content(&self, uri: String, cell_id: String, content: String) -> bool {
        let mut notebooks = self.notebooks.lock().unwrap();
        if let Some(notebook) = notebooks.get_mut(&uri) {
            notebook.version += 1;
            if let Some(cell) = notebook.cells.iter_mut().find(|c| c.id == cell_id) {
                cell.content = content;
                cell.is_dirty = true;
                return true;
            }
        }
        false
    }

    #[napi]
    pub fn set_cell_outputs(&self, uri: String, cell_id: String, outputs: Vec<NotebookCellOutput>) -> bool {
        let mut notebooks = self.notebooks.lock().unwrap();
        if let Some(notebook) = notebooks.get_mut(&uri) {
            if let Some(cell) = notebook.cells.iter_mut().find(|c| c.id == cell_id) {
                cell.outputs = outputs;
                return true;
            }
        }
        false
    }

    #[napi]
    pub fn update_execution_summary(&self, uri: String, cell_id: String, summary: NotebookExecutionSummary) -> bool {
        let mut notebooks = self.notebooks.lock().unwrap();
        if let Some(notebook) = notebooks.get_mut(&uri) {
            if let Some(cell) = notebook.cells.iter_mut().find(|c| c.id == cell_id) {
                cell.execution_summary = Some(summary);
                return true;
            }
        }
        false
    }

    #[napi]
    pub fn add_cell(&self, uri: String, cell: NotebookCell, index: u32) -> bool {
        let mut notebooks = self.notebooks.lock().unwrap();
        if let Some(notebook) = notebooks.get_mut(&uri) {
            let idx = index as usize;
            if idx <= notebook.cells.len() {
                notebook.cells.insert(idx, cell);
                notebook.version += 1;
                return true;
            }
        }
        false
    }

    #[napi]
    pub fn get_notebook(&self, uri: String) -> Option<NotebookData> {
        self.notebooks.lock().unwrap().get(&uri).cloned()
    }
}
