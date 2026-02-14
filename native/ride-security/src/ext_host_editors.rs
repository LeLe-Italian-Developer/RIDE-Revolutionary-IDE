use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Mutex;
use crate::ext_host_documents::ExtHostTextDocumentData;

#[napi(object)]
#[derive(Clone)]
pub struct ExtHostTextEditorData {
    pub id: String,
    pub document: ExtHostTextDocumentData,
    pub selection: Option<String>, // Serialized selection
}

#[napi]
pub struct ExtHostEditors {
    active_editor_id: Mutex<Option<String>>,
}

#[napi]
impl ExtHostEditors {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            active_editor_id: Mutex::new(None),
        }
    }

    #[napi]
    pub fn get_active_editor_id(&self) -> Option<String> {
        self.active_editor_id.lock().unwrap().clone()
    }

    #[napi]
    pub fn set_active_editor_id(&self, id: Option<String>) {
        let mut active = self.active_editor_id.lock().unwrap();
        *active = id;
    }
}
