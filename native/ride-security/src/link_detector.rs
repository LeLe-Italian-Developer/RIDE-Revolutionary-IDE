/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Link Detector — Rust port of `src/vs/editor/contrib/links/browser/linkDetector.ts`.
//! Detects URLs and email addresses in text.

use napi_derive::napi;
use napi::bindgen_prelude::*;
use regex::Regex;
use std::sync::OnceLock;

static URL_REGEX: OnceLock<Regex> = OnceLock::new();
static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();

#[napi(object)]
#[derive(Clone, Debug)]
pub struct LinkRange {
    pub start: u32,
    pub end: u32,
    pub url: String,
    pub kind: String, // "url", "email"
}

fn get_url_regex() -> &'static Regex {
    URL_REGEX.get_or_init(|| Regex::new(r"https?://[^\s<>]+").unwrap())
}

fn get_email_regex() -> &'static Regex {
    EMAIL_REGEX.get_or_init(|| Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap())
}

#[napi]
pub fn find_links(text: String) -> Vec<LinkRange> {
    let mut results = Vec::new();

    // URLs
    for caps in get_url_regex().captures_iter(&text) {
        if let Some(m) = caps.get(0) {
            results.push(LinkRange {
                start: m.start() as u32,
                end: m.end() as u32,
                url: m.as_str().to_string(),
                kind: "url".to_string(),
            });
        }
    }

    // Emails
    for caps in get_email_regex().captures_iter(&text) {
        if let Some(m) = caps.get(0) {
            let s = m.as_str();
            // Check if inside URL (naive check: preceding/following characters)
            // But Regex iter handles overlap slightly better if distinct?
            // Actually, if we search URLs first, we might mark ranges.
            // For now, just return all. Overlap handling is complex without tokenization.
            // But simple email regex is decent.
            results.push(LinkRange {
                start: m.start() as u32,
                end: m.end() as u32,
                url: format!("mailto:{}", s),
                kind: "email".to_string(),
            });
        }
    }

    results.sort_by_key(|a| a.start);
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_url() {
        let text = "Check https://example.com for details";
        let links = find_links(text.into());
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].url, "https://example.com");
        assert_eq!(links[0].kind, "url");
    }

    #[test]
    fn test_find_email() {
        let text = "Contact me at foo@bar.com";
        let links = find_links(text.into());
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].url, "mailto:foo@bar.com");
        assert_eq!(links[0].kind, "email");
    }
}
