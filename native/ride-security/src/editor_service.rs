use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;
use std::sync::Mutex;

#[napi(object)]
#[derive(Clone, Debug)]
pub struct EditorInput {
    pub resource: String,
    pub type_id: Option<String>,
    pub label: Option<String>,
}

#[napi]
pub struct EditorService {
    // group_id -> list of editors
    groups: Mutex<HashMap<u32, Vec<EditorInput>>>,
    active_group: Mutex<u32>,
}

#[napi]
impl EditorService {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            groups: Mutex::new(HashMap::new()),
            active_group: Mutex::new(0),
        }
    }

    #[napi]
    pub fn open_editor(&self, group_id: u32, editor: EditorInput) {
        let mut groups = self.groups.lock().unwrap();
        let group = groups.entry(group_id).or_default();
        
        // If already open, move to front/replace?
        // VS Code logic: if exists in group, make it active.
        if !group.iter().any(|e| e.resource == editor.resource) {
            group.push(editor);
        }
        
        let mut active = self.active_group.lock().unwrap();
        *active = group_id;
    }

    #[napi]
    pub fn close_editor(&self, group_id: u32, resource: String) -> bool {
        let mut groups = self.groups.lock().unwrap();
        if let Some(group) = groups.get_mut(&group_id) {
            let len_before = group.len();
            group.retain(|e| e.resource != resource);
            return group.len() < len_before;
        }
        false
    }

    #[napi]
    pub fn get_opened_editors(&self, group_id: u32) -> Vec<EditorInput> {
        let groups = self.groups.lock().unwrap();
        groups.get(&group_id).cloned().unwrap_or_default()
    }

    #[napi]
    pub fn get_active_group(&self) -> u32 {
        *self.active_group.lock().unwrap()
    }

    #[napi]
    pub fn set_active_group(&self, group_id: u32) {
        let mut active = self.active_group.lock().unwrap();
        *active = group_id;
    }
    
    #[napi]
    pub fn get_all_editors(&self) -> Vec<EditorInput> {
        let groups = self.groups.lock().unwrap();
        let mut all = Vec::new();
        for g in groups.values() {
            all.extend(g.clone());
        }
        all
    }
}
