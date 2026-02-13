/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Syntax analysis utilities for the editor.
//! Bracket matching, indentation detection, line ending normalization,
//! word boundary detection, and Unicode-aware text processing.


use napi_derive::napi;

/// Result of bracket matching analysis.
#[napi(object)]
#[derive(Clone)]
pub struct BracketMatch {
    pub open_offset: u32,
    pub close_offset: u32,
    pub bracket_type: String,
    pub depth: u32,
}

/// Indentation detection result.
#[napi(object)]
pub struct IndentationInfo {
    pub use_tabs: bool,
    pub tab_size: u32,
    pub confidence: f64,
    pub lines_with_tabs: u32,
    pub lines_with_spaces: u32,
}

/// Line ending information.
#[napi(object)]
pub struct LineEndingInfo {
    pub dominant: String,
    pub crlf_count: u32,
    pub lf_count: u32,
    pub cr_count: u32,
    pub mixed: bool,
}

/// Word boundary result.
#[napi(object)]
#[derive(Clone)]
pub struct WordRange {
    pub start: u32,
    pub end: u32,
    pub word: String,
}

/// Text statistics.
#[napi(object)]
pub struct TextStats {
    pub char_count: u32,
    pub line_count: u32,
    pub word_count: u32,
    pub whitespace_count: u32,
    pub max_line_length: u32,
    pub avg_line_length: f64,
    pub has_trailing_newline: bool,
    pub has_bom: bool,
}

const OPEN_BRACKETS: &[char] = &['(', '[', '{', '<'];
const CLOSE_BRACKETS: &[char] = &[')', ']', '}', '>'];

fn bracket_type(c: char) -> &'static str {
    match c {
        '(' | ')' => "paren",
        '[' | ']' => "square",
        '{' | '}' => "curly",
        '<' | '>' => "angle",
        _ => "unknown",
    }
}

/// Find all matching bracket pairs in the text.
#[napi]
pub fn match_brackets(text: String) -> Vec<BracketMatch> {
    let mut matches = Vec::new();
    let mut stack: Vec<(char, u32, u32)> = Vec::new(); // (bracket, offset, depth)
    let mut depth = 0u32;
    let mut in_string = false;
    let mut string_char: char = '"';
    let mut prev_char = '\0';

    for (i, c) in text.char_indices() {
        // Handle string literals
        if (c == '"' || c == '\'' || c == '`') && prev_char != '\\' {
            if in_string && c == string_char {
                in_string = false;
            } else if !in_string {
                in_string = true;
                string_char = c;
            }
        }
        prev_char = c;

        if in_string { continue; }

        if OPEN_BRACKETS.contains(&c) {
            depth += 1;
            stack.push((c, i as u32, depth));
        } else if CLOSE_BRACKETS.contains(&c) {
            let close_idx = CLOSE_BRACKETS.iter().position(|&b| b == c);
            if let Some(idx) = close_idx {
                let expected_open = OPEN_BRACKETS[idx];
                if let Some(pos) = stack.iter().rposition(|&(b, _, _)| b == expected_open) {
                    let (_, open_offset, d) = stack.remove(pos);
                    matches.push(BracketMatch {
                        open_offset,
                        close_offset: i as u32,
                        bracket_type: bracket_type(c).to_string(),
                        depth: d,
                    });
                    depth = depth.saturating_sub(1);
                }
            }
        }
    }
    matches
}

/// Find the matching bracket for a given position.
#[napi]
pub fn find_matching_bracket(text: String, position: u32) -> Option<u32> {
    let all = match_brackets(text);
    for m in &all {
        if m.open_offset == position { return Some(m.close_offset); }
        if m.close_offset == position { return Some(m.open_offset); }
    }
    None
}

/// Detect indentation style and tab size.
#[napi]
pub fn detect_indentation(text: String) -> IndentationInfo {
    let mut tab_lines = 0u32;
    let mut space_lines = 0u32;
    let mut space_widths: Vec<u32> = Vec::new();

    for line in text.lines() {
        if line.is_empty() { continue; }
        if line.starts_with('\t') {
            tab_lines += 1;
        } else if line.starts_with(' ') {
            space_lines += 1;
            let indent_len = line.len() - line.trim_start_matches(' ').len();
            if indent_len > 0 { space_widths.push(indent_len as u32); }
        }
    }

    let total = tab_lines + space_lines;
    let use_tabs = tab_lines > space_lines;
    let confidence = if total == 0 { 0.5 } else { (tab_lines.max(space_lines) as f64) / (total as f64) };

    // Detect tab size from space indentation GCDs
    let tab_size = if space_widths.is_empty() {
        4
    } else {
        let mut counts = [0u32; 9]; // 1-8
        for &w in &space_widths {
            for size in 1..=8u32 {
                if w % size == 0 { counts[size as usize] += 1; }
            }
        }
        // Prefer 2 or 4
        if counts[2] > counts[4] && counts[2] as f64 > space_widths.len() as f64 * 0.7 { 2 }
        else if counts[4] as f64 > space_widths.len() as f64 * 0.5 { 4 }
        else { 4 }
    };

    IndentationInfo { use_tabs, tab_size, confidence, lines_with_tabs: tab_lines, lines_with_spaces: space_lines }
}

/// Detect and report line ending style.
#[napi]
pub fn detect_line_endings(text: String) -> LineEndingInfo {
    let mut crlf = 0u32;
    let mut lf = 0u32;
    let mut cr = 0u32;
    let bytes = text.as_bytes();

    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\r' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                crlf += 1;
                i += 2;
            } else {
                cr += 1;
                i += 1;
            }
        } else if bytes[i] == b'\n' {
            lf += 1;
            i += 1;
        } else {
            i += 1;
        }
    }

    let dominant = if crlf >= lf && crlf >= cr { "crlf" }
        else if lf >= cr { "lf" }
        else { "cr" };
    let mixed = (crlf > 0) as u32 + (lf > 0) as u32 + (cr > 0) as u32 > 1;

    LineEndingInfo { dominant: dominant.to_string(), crlf_count: crlf, lf_count: lf, cr_count: cr, mixed }
}

/// Normalize line endings to the specified style.
#[napi]
pub fn normalize_line_endings(text: String, target: String) -> String {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    match target.as_str() {
        "crlf" => normalized.replace('\n', "\r\n"),
        "cr" => normalized.replace('\n', "\r"),
        _ => normalized,
    }
}

/// Extract words with their positions from text.
#[napi]
pub fn extract_words(text: String) -> Vec<WordRange> {
    let mut words = Vec::new();
    let mut word_start: Option<usize> = None;

    for (i, c) in text.char_indices() {
        let is_word_char = c.is_alphanumeric() || c == '_';
        match (is_word_char, word_start) {
            (true, None) => { word_start = Some(i); }
            (false, Some(start)) => {
                words.push(WordRange {
                    start: start as u32,
                    end: i as u32,
                    word: text[start..i].to_string(),
                });
                word_start = None;
            }
            _ => {}
        }
    }
    if let Some(start) = word_start {
        words.push(WordRange { start: start as u32, end: text.len() as u32, word: text[start..].to_string() });
    }
    words
}

/// Get the word at a specific offset in the text.
#[napi]
pub fn word_at_position(text: String, offset: u32) -> Option<WordRange> {
    let off = offset as usize;
    if off >= text.len() { return None; }

    let bytes = text.as_bytes();
    let is_word = |b: u8| b.is_ascii_alphanumeric() || b == b'_';

    if !is_word(bytes[off]) { return None; }

    let mut start = off;
    while start > 0 && is_word(bytes[start - 1]) { start -= 1; }
    let mut end = off;
    while end < bytes.len() && is_word(bytes[end]) { end += 1; }

    Some(WordRange { start: start as u32, end: end as u32, word: text[start..end].to_string() })
}

/// Compute text statistics.
#[napi]
pub fn text_stats(text: String) -> TextStats {
    let lines: Vec<&str> = text.lines().collect();
    let line_count = lines.len().max(1) as u32;
    let char_count = text.chars().count() as u32;
    let word_count = text.split_whitespace().count() as u32;
    let whitespace_count = text.chars().filter(|c| c.is_whitespace()).count() as u32;
    let max_line_length = lines.iter().map(|l| l.len()).max().unwrap_or(0) as u32;
    let avg_line_length = if lines.is_empty() { 0.0 } else { lines.iter().map(|l| l.len()).sum::<usize>() as f64 / lines.len() as f64 };
    let has_trailing_newline = text.ends_with('\n') || text.ends_with("\r\n");
    let has_bom = text.starts_with('\u{feff}');

    TextStats { char_count, line_count, word_count, whitespace_count, max_line_length, avg_line_length, has_trailing_newline, has_bom }
}

/// Remove BOM (Byte Order Mark) from text if present.
#[napi]
pub fn strip_bom(text: String) -> String {
    text.strip_prefix('\u{feff}').unwrap_or(&text).to_string()
}

/// Count occurrences of a substring.
#[napi]
pub fn count_substring(text: String, substring: String) -> u32 {
    text.matches(&substring).count() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bracket_matching() {
        let matches = match_brackets("fn foo() { bar(x) }".to_string());
        assert!(matches.len() >= 2); // () and {}
    }

    #[test]
    fn test_find_matching() {
        let text = "(hello)".to_string();
        assert_eq!(find_matching_bracket(text.clone(), 0), Some(6));
        assert_eq!(find_matching_bracket(text, 6), Some(0));
    }

    #[test]
    fn test_detect_spaces() {
        let text = "fn main() {\n    let x = 1;\n    let y = 2;\n}\n";
        let info = detect_indentation(text.to_string());
        assert!(!info.use_tabs);
        assert_eq!(info.tab_size, 4);
    }

    #[test]
    fn test_detect_tabs() {
        let text = "fn main() {\n\tlet x = 1;\n\tlet y = 2;\n}\n";
        let info = detect_indentation(text.to_string());
        assert!(info.use_tabs);
    }

    #[test]
    fn test_line_endings_lf() {
        let info = detect_line_endings("a\nb\nc\n".to_string());
        assert_eq!(info.dominant, "lf");
        assert!(!info.mixed);
    }

    #[test]
    fn test_line_endings_crlf() {
        let info = detect_line_endings("a\r\nb\r\nc\r\n".to_string());
        assert_eq!(info.dominant, "crlf");
    }

    #[test]
    fn test_normalize_to_crlf() {
        let result = normalize_line_endings("a\nb\n".to_string(), "crlf".to_string());
        assert_eq!(result, "a\r\nb\r\n");
    }

    #[test]
    fn test_extract_words() {
        let words = extract_words("hello world_123 foo".to_string());
        assert_eq!(words.len(), 3);
        assert_eq!(words[0].word, "hello");
        assert_eq!(words[1].word, "world_123");
    }

    #[test]
    fn test_word_at_position() {
        let result = word_at_position("hello world".to_string(), 7);
        assert!(result.is_some());
        assert_eq!(result.unwrap().word, "world");
    }

    #[test]
    fn test_text_stats() {
        let stats = text_stats("Hello World\nLine 2\n".to_string());
        assert_eq!(stats.line_count, 2);
        assert!(stats.word_count >= 3);
        assert!(stats.has_trailing_newline);
    }

    #[test]
    fn test_strip_bom() {
        assert_eq!(strip_bom("\u{feff}Hello".to_string()), "Hello");
        assert_eq!(strip_bom("Hello".to_string()), "Hello");
    }
}
