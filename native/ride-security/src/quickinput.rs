use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi(object)]
pub struct QuickPickItem {
    pub label: String,
    pub description: Option<String>,
    pub detail: Option<String>,
}

#[napi]
pub struct QuickInputService {}

#[napi]
impl QuickInputService {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {}
    }

    #[napi]
    pub fn filter_items(&self, items: Vec<QuickPickItem>, query: String) -> Vec<QuickPickItem> {
        let q = query.to_lowercase();
        items.into_iter()
            .filter(|item| {
                item.label.to_lowercase().contains(&q) || 
                item.description.as_ref().map(|d| d.to_lowercase().contains(&q)).unwrap_or(false)
            })
            .collect()
    }
}
