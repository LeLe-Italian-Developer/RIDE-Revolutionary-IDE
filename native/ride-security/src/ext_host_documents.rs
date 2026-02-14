/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;
use std::sync::Mutex;
use crate::range::Range;

#[napi(object)]
#[derive(Clone)]
pub struct ExtHostTextDocumentData {
    pub uri: String,
    pub version: u32,
    pub lines: Vec<String>,
    pub language_id: String,
    pub is_dirty: bool,
}

#[napi]
pub struct ExtHostDocuments {
    documents: Mutex<HashMap<String, ExtHostTextDocumentData>>,
}

#[napi]
impl ExtHostDocuments {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            documents: Mutex::new(HashMap::new()),
        }
    }

    #[napi]
    pub fn set_document(&self, data: ExtHostTextDocumentData) {
        let mut docs = self.documents.lock().unwrap();
        docs.insert(data.uri.clone(), data);
    }

    #[napi]
    pub fn get_document(&self, uri: String) -> Option<ExtHostTextDocumentData> {
        let docs = self.documents.lock().unwrap();
        docs.get(&uri).cloned()
    }

    #[napi]
    pub fn apply_edits(&self, uri: String, version: u32, _edits_json: String) -> bool {
        let mut docs = self.documents.lock().unwrap();
        if let Some(doc) = docs.get_mut(&uri) {
            // Apply line-by-line edits (Simplified)
            // In reality, this would use a PieceTree or perform string manipulations
            doc.version = version;
            return true;
        }
        false
    }
}
