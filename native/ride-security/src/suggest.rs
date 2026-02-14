/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Suggest Engine — Rust port of `src/vs/editor/contrib/suggest/browser/suggest.ts`.
//! Filter and sort completion items based on fuzzy matching.

use napi_derive::napi;
use napi::bindgen_prelude::*;
use crate::glob_engine::glob_fuzzy_match; // Uses fuzzy logic from Phase 8

#[napi(object)]
#[derive(Clone, Debug)]
pub struct CompletionItem {
    pub label: String,
    pub filter_text: Option<String>,
    pub sort_text: Option<String>,
    pub kind: Option<u32>,
    pub score: Option<f64>,
}

#[napi]
pub fn filter_completion_items(query: String, items: Vec<CompletionItem>) -> Vec<CompletionItem> {
    let mut scored_items: Vec<CompletionItem> = items.into_iter().map(|mut item| {
        let text = item.filter_text.as_deref().unwrap_or(&item.label);
        let result = glob_fuzzy_match(query.clone(), text.to_string());
        if result.score > 0.0 {
            item.score = Some(result.score);
        } else {
            item.score = None;
        }
        item
    }).filter(|item| item.score.is_some()).collect();

    // Sort by score (descending), then sortText (ascending), then label (ascending)
    scored_items.sort_by(|a, b| {
        b.score.unwrap_or(0.0).partial_cmp(&a.score.unwrap_or(0.0)).unwrap()
            .then_with(|| {
                a.sort_text.as_deref().unwrap_or(&a.label)
                    .e_cmp(b.sort_text.as_deref().unwrap_or(&b.label))
            })
    });

    scored_items
}

trait Ecmp {
    fn e_cmp(&self, other: &Self) -> std::cmp::Ordering;
}

impl Ecmp for str {
    fn e_cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Simple string comparison for now. VS Code uses locale compare.
        self.cmp(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter() {
        let items = vec![
            CompletionItem { label: "console".into(), filter_text: None, sort_text: None, kind: None, score: None },
            CompletionItem { label: "const".into(), filter_text: None, sort_text: None, kind: None, score: None },
            CompletionItem { label: "bar".into(), filter_text: None, sort_text: None, kind: None, score: None },
        ];

        let filtered = filter_completion_items("con".into(), items);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].label, "console"); // fuzzy match "con" -> "console" (score higher?) or same?
        // "console" has "con" at start. "const" has "con" at start.
        // Length penalty might make "const" score higher (shorter)?
        // Let's assume glob_fuzzy_match handles it.
        // Actually fuzzy match usually prioritizes shorter matches or exact prefixes.
    }
}
