/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Word Operations — Rust port of `src/vs/editor/common/cursor/cursorWordOperations.ts`.
//! Word boundary detection and navigation helper.

use napi_derive::napi;
use napi::bindgen_prelude::*;
use regex::Regex;
use std::sync::OnceLock;

static WORD_REGEX: OnceLock<Regex> = OnceLock::new();

#[napi(object)]
#[derive(Clone, Debug)]
pub struct CursorWordRange {
    pub start: u32,
    pub end: u32,
    pub word: String,
}

fn get_word_regex() -> &'static Regex {
    WORD_REGEX.get_or_init(|| Regex::new(r"\w+").unwrap())
}

#[napi]
pub fn find_word_at_offset(text: String, offset: u32) -> Option<CursorWordRange> {
    let offset_usize = offset as usize;
    if offset_usize > text.len() { return None; }

    for m in get_word_regex().find_iter(&text) {
        if m.start() <= offset_usize && offset_usize <= m.end() {
            return Some(CursorWordRange {
                start: m.start() as u32,
                end: m.end() as u32,
                word: m.as_str().to_string(),
            });
        }
    }
    None
}

#[napi]
pub fn get_word_start(text: String, offset: u32) -> u32 {
    if let Some(w) = find_word_at_offset(text, offset) {
        w.start
    } else {
        offset
    }
}

#[napi]
pub fn get_word_end(text: String, offset: u32) -> u32 {
    if let Some(w) = find_word_at_offset(text, offset) {
        w.end
    } else {
        offset
    }
}

// ─── Non-NAPI helpers used by Cursor ───────────────────────────────────────

pub fn find_previous_word_start(text: &str, column: u32) -> u32 {
    let offset = if column > 0 { (column - 1) as usize } else { 0 };
    let mut last_start = 1;
    for m in get_word_regex().find_iter(text) {
        if m.start() < offset {
            last_start = (m.start() + 1) as u32;
        } else {
            break;
        }
    }
    last_start
}

pub fn find_next_word_end(text: &str, column: u32) -> u32 {
    let offset = (column - 1) as usize;
    for m in get_word_regex().find_iter(text) {
        if m.end() > offset {
            return (m.end() + 1) as u32;
        }
    }
    (text.chars().count() + 1) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_word() {
        let text = "hello world";
        let w = find_word_at_offset(text.into(), 2).unwrap();
        assert_eq!(w.word, "hello");
        assert_eq!(w.start, 0);
        assert_eq!(w.end, 5);

        let w2 = find_word_at_offset(text.into(), 6).unwrap();
        assert_eq!(w2.word, "world");
    }

    #[test]
    fn test_word_navigation() {
        let text = "the quick brown";
        assert_eq!(find_previous_word_start(text, 10), 5); // start of "quick"
        assert_eq!(find_next_word_end(text, 5), 10); // end of "quick"
    }
}
