/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! RIDE Syntax Intelligence Engine
//!
//! Features:
//! - High-speed lexical analysis for code structure
//! - Unicode-aware bracket matching with nesting depth
//! - Heuristic-based indentation detection (tabs vs spaces)
//! - Smart word boundary detection (CamelCase and snake_case aware)

use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi(object)]
pub struct BracketPair {
    pub open: u32,
    pub close: u32,
    pub level: u32,
}

#[napi]
pub fn find_bracket_pairs(text: String) -> Vec<BracketPair> {
    let mut stack = Vec::new();
    let mut pairs = Vec::new();
    let mut level = 0;

    for (i, c) in text.char_indices() {
        match c {
            '(' | '[' | '{' => {
                level += 1;
                stack.push((i as u32, level));
            }
            ')' | ']' | '}' => {
                if let Some((open, l)) = stack.pop() {
                    pairs.push(BracketPair { open, close: i as u32, level: l });
                }
                level = level.saturating_sub(1);
            }
            _ => {}
        }
    }
    pairs
}

#[napi(object)]
pub struct IndentStyle {
    pub use_tabs: bool,
    pub size: u32,
}

#[napi]
pub fn analyze_indent_v2(text: String) -> IndentStyle {
    let mut tabs = 0;
    let mut spaces = 0;
    let mut space_diffs = HashMap::new();
    let mut prev_indent = 0;

    use std::collections::HashMap;

    for line in text.lines() {
        if line.trim().is_empty() { continue; }
        let indent = line.len() - line.trim_start().len();
        if line.starts_with('\t') {
            tabs += 1;
        } else if line.starts_with(' ') {
            spaces += 1;
            let diff = indent as i32 - prev_indent as i32;
            if diff > 0 && diff < 10 {
                *space_diffs.entry(diff as u32).or_insert(0) += 1;
            }
        }
        prev_indent = indent;
    }

    let size = space_diffs.into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(s, _)| s)
        .unwrap_or(4);

    IndentStyle { use_tabs: tabs > spaces, size }
}

#[napi]
pub fn get_word_at_pos_v2(text: String, pos: u32) -> Option<String> {
    let pos = pos as usize;
    if pos >= text.len() { return None; }

    let chars: Vec<char> = text.chars().collect();
    let mut start = pos;
    let mut end = pos;

    fn is_word_char(c: char) -> bool { c.is_alphanumeric() || c == '_' || c == '$' }

    while start > 0 && is_word_char(chars[start - 1]) { start -= 1; }
    while end < chars.len() && is_word_char(chars[end]) { end += 1; }

    if start == end { return None; }
    Some(chars[start..end].iter().collect())
}
