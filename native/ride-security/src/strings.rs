/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! String manipulation utilities — Rust port of `src/vs/base/common/strings.ts`.
//!
//! Provides Unicode-aware string operations: comparison, escaping, padding,
//! regex escaping, word boundaries, whitespace normalization, and more.

use napi_derive::napi;
use napi::bindgen_prelude::*;
use std::collections::HashMap;

// ─── Character classification ──────────────────────────────────────────────

/// Check if a character code is a high surrogate (0xD800–0xDBFF).
#[napi]
pub fn is_high_surrogate(char_code: u32) -> bool {
    (0xD800..=0xDBFF).contains(&char_code)
}

/// Check if a character code is a low surrogate (0xDC00–0xDFFF).
#[napi]
pub fn is_low_surrogate(char_code: u32) -> bool {
    (0xDC00..=0xDFFF).contains(&char_code)
}

/// Compute the full Unicode code point from a surrogate pair.
#[napi]
pub fn compute_code_point(high: u32, low: u32) -> u32 {
    ((high - 0xD800) << 10) + (low - 0xDC00) + 0x10000
}

/// Check if a code point is a basic ASCII letter (a-z, A-Z).
#[napi]
pub fn is_ascii_letter(code: u32) -> bool {
    matches!(code, 65..=90 | 97..=122)
}

/// Check if a code point is a digit (0-9).
#[napi]
pub fn is_digit(code: u32) -> bool {
    (48..=57).contains(&code)
}

/// Check if a character is a whitespace character.
#[napi]
pub fn is_whitespace(ch: String) -> bool {
    ch.chars().next().map_or(false, |c| c.is_whitespace())
}

/// Check if a character is an upper-case ASCII letter.
#[napi]
pub fn is_upper_ascii_letter(code: u32) -> bool {
    (65..=90).contains(&code)
}

/// Check if a character is a lower-case ASCII letter.
#[napi]
pub fn is_lower_ascii_letter(code: u32) -> bool {
    (97..=122).contains(&code)
}

// ─── String comparison ─────────────────────────────────────────────────────

/// Case-insensitive string comparison. Returns true if both strings are
/// equal when converted to lowercase.
#[napi]
pub fn equals_ignore_case(a: String, b: String) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

/// Case-insensitive check if `haystack` starts with `needle`.
#[napi]
pub fn starts_with_ignore_case(haystack: String, needle: String) -> bool {
    haystack.to_lowercase().starts_with(&needle.to_lowercase())
}

/// Compare two strings, returning -1, 0, or 1.
#[napi]
pub fn compare(a: String, b: String) -> i32 {
    match a.cmp(&b) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

/// Case-insensitive compare of two strings.
#[napi]
pub fn compare_ignore_case(a: String, b: String) -> i32 {
    let la = a.to_lowercase();
    let lb = b.to_lowercase();
    match la.cmp(&lb) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

/// Compare two strings with numeric awareness (natural sort).
/// e.g., "file2" < "file10"
#[napi]
pub fn compare_natural(a: String, b: String) -> i32 {
    let mut ai = a.chars().peekable();
    let mut bi = b.chars().peekable();

    loop {
        match (ai.peek(), bi.peek()) {
            (None, None) => return 0,
            (None, Some(_)) => return -1,
            (Some(_), None) => return 1,
            (Some(&ac), Some(&bc)) => {
                if ac.is_ascii_digit() && bc.is_ascii_digit() {
                    let mut an = String::new();
                    while let Some(&c) = ai.peek() {
                        if c.is_ascii_digit() {
                            an.push(c);
                            ai.next();
                        } else {
                            break;
                        }
                    }
                    let mut bn = String::new();
                    while let Some(&c) = bi.peek() {
                        if c.is_ascii_digit() {
                            bn.push(c);
                            bi.next();
                        } else {
                            break;
                        }
                    }
                    let na: u64 = an.parse().unwrap_or(0);
                    let nb: u64 = bn.parse().unwrap_or(0);
                    match na.cmp(&nb) {
                        std::cmp::Ordering::Less => return -1,
                        std::cmp::Ordering::Greater => return 1,
                        std::cmp::Ordering::Equal => {}
                    }
                } else {
                    let al = ac.to_lowercase().next().unwrap_or(ac);
                    let bl = bc.to_lowercase().next().unwrap_or(bc);
                    match al.cmp(&bl) {
                        std::cmp::Ordering::Less => return -1,
                        std::cmp::Ordering::Greater => return 1,
                        std::cmp::Ordering::Equal => {
                            ai.next();
                            bi.next();
                        }
                    }
                }
            }
        }
    }
}

// ─── String transformations ────────────────────────────────────────────────

/// Escape special regex characters in a string.
#[napi]
pub fn escape_regex(value: String) -> String {
    regex::escape(&value)
}

/// Escape HTML entities in a string.
#[napi]
pub fn escape_html(text: String) -> String {
    let mut result = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#39;"),
            _ => result.push(ch),
        }
    }
    result
}

/// Convert the first character to uppercase.
#[napi]
pub fn uppercase_first_letter(s: String) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => {
            let upper: String = c.to_uppercase().collect();
            upper + chars.as_str()
        }
    }
}

/// Convert the first character to lowercase.
#[napi]
pub fn lowercase_first_letter(s: String) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => {
            let lower: String = c.to_lowercase().collect();
            lower + chars.as_str()
        }
    }
}

/// Convert a string to camelCase.
#[napi]
pub fn to_camel_case(s: String) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = false;
    let mut first = true;
    for ch in s.chars() {
        if ch == '_' || ch == '-' || ch == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(ch.to_uppercase());
            capitalize_next = false;
            first = false;
        } else if first {
            result.extend(ch.to_lowercase());
            first = false;
        } else {
            result.push(ch);
        }
    }
    result
}

/// Convert a string to snake_case.
#[napi]
pub fn to_snake_case(s: String) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    let mut prev_lower = false;
    for ch in s.chars() {
        if ch.is_uppercase() {
            if prev_lower {
                result.push('_');
            }
            result.extend(ch.to_lowercase());
            prev_lower = false;
        } else if ch == '-' || ch == ' ' {
            result.push('_');
            prev_lower = false;
        } else {
            result.push(ch);
            prev_lower = ch.is_lowercase();
        }
    }
    result
}

/// Convert a string to kebab-case.
#[napi]
pub fn to_kebab_case(s: String) -> String {
    to_snake_case(s).replace('_', "-")
}

// ─── String padding & trimming ─────────────────────────────────────────────

/// Pad a string on the left to the specified length.
#[napi]
pub fn pad_left(s: String, total_width: u32, pad_char: Option<String>) -> String {
    let pad = pad_char.and_then(|p| p.chars().next()).unwrap_or(' ');
    let width = total_width as usize;
    if s.len() >= width {
        return s;
    }
    let padding: String = std::iter::repeat(pad).take(width - s.len()).collect();
    padding + &s
}

/// Pad a string on the right to the specified length.
#[napi]
pub fn pad_right(s: String, total_width: u32, pad_char: Option<String>) -> String {
    let pad = pad_char.and_then(|p| p.chars().next()).unwrap_or(' ');
    let width = total_width as usize;
    if s.len() >= width {
        return s;
    }
    let padding: String = std::iter::repeat(pad).take(width - s.len()).collect();
    s + &padding
}

/// Trim whitespace from both ends (Unicode-aware).
#[napi]
pub fn trim_string(s: String) -> String {
    s.trim().to_string()
}

/// Trim whitespace from the start.
#[napi]
pub fn ltrim(s: String, char_to_trim: Option<String>) -> String {
    match char_to_trim {
        Some(ch) => {
            let c = ch.chars().next().unwrap_or(' ');
            s.trim_start_matches(c).to_string()
        }
        None => s.trim_start().to_string(),
    }
}

/// Trim whitespace from the end.
#[napi]
pub fn rtrim(s: String, char_to_trim: Option<String>) -> String {
    match char_to_trim {
        Some(ch) => {
            let c = ch.chars().next().unwrap_or(' ');
            s.trim_end_matches(c).to_string()
        }
        None => s.trim_end().to_string(),
    }
}

// ─── String analysis ───────────────────────────────────────────────────────

/// Count occurrences of a substring within a string.
#[napi]
pub fn count_occurrences(haystack: String, needle: String) -> u32 {
    if needle.is_empty() {
        return 0;
    }
    haystack.matches(&needle).count() as u32
}

/// Get the common prefix length of two strings.
#[napi]
pub fn common_prefix_length(a: String, b: String) -> u32 {
    a.chars()
        .zip(b.chars())
        .take_while(|(ac, bc)| ac == bc)
        .count() as u32
}

/// Get the common suffix length of two strings.
#[napi]
pub fn common_suffix_length(a: String, b: String) -> u32 {
    a.chars()
        .rev()
        .zip(b.chars().rev())
        .take_while(|(ac, bc)| ac == bc)
        .count() as u32
}

/// Check if a string contains only whitespace.
#[napi]
pub fn is_blank(s: String) -> bool {
    s.trim().is_empty()
}

/// Check if the first character is uppercase.
#[napi]
pub fn is_first_upper(s: String) -> bool {
    s.chars().next().map_or(false, |c| c.is_uppercase())
}

/// Count the number of lines in a string.
#[napi]
pub fn count_lines(s: String) -> u32 {
    if s.is_empty() {
        return 0;
    }
    (s.matches('\n').count() + 1) as u32
}

/// Get the length of a string in UTF-16 code units (matching JavaScript's .length).
#[napi]
pub fn utf16_length(s: String) -> u32 {
    s.encode_utf16().count() as u32
}

// ─── String splitting & joining ────────────────────────────────────────────

/// Split a string into lines, preserving line endings.
#[napi]
pub fn split_lines(text: String) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\r' {
            if i + 1 < chars.len() && chars[i + 1] == '\n' {
                current.push('\r');
                current.push('\n');
                lines.push(current.clone());
                current.clear();
                i += 2;
            } else {
                current.push('\r');
                lines.push(current.clone());
                current.clear();
                i += 1;
            }
        } else if chars[i] == '\n' {
            current.push('\n');
            lines.push(current.clone());
            current.clear();
            i += 1;
        } else {
            current.push(chars[i]);
            i += 1;
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// Create a regex-safe version of a string for use in word boundary matching.
#[napi]
pub fn create_word_regex(word: String) -> String {
    format!(r"\b{}\b", regex::escape(&word))
}

// ─── String encoding ──────────────────────────────────────────────────────

/// Encode a string to Base64.
#[napi]
pub fn encode_base64(data: String) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data.as_bytes())
}

/// Decode a Base64 string.
#[napi]
pub fn decode_base64(encoded: String) -> Result<String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded.as_bytes())
        .map_err(|e| Error::from_reason(format!("Invalid base64: {}", e)))?;
    String::from_utf8(bytes)
        .map_err(|e| Error::from_reason(format!("Invalid UTF-8 in base64: {}", e)))
}

/// Convert a string to a hex-encoded representation.
#[napi]
pub fn to_hex_string(data: String) -> String {
    data.bytes().map(|b| format!("{:02x}", b)).collect()
}

/// Convert hex string back to regular string.
#[napi]
pub fn from_hex_string(hex: String) -> Result<String> {
    let bytes: Result<Vec<u8>> = (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| Error::from_reason(format!("Invalid hex: {}", e)))
        })
        .collect();
    String::from_utf8(bytes?)
        .map_err(|e| Error::from_reason(format!("Invalid UTF-8: {}", e)))
}

// ─── Levenshtein distance ──────────────────────────────────────────────────

/// Compute the Levenshtein edit distance between two strings.
#[napi]
pub fn levenshtein_distance(a: String, b: String) -> u32 {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 { return n as u32; }
    if n == 0 { return m as u32; }

    let mut prev: Vec<u32> = (0..=(n as u32)).collect();
    let mut curr = vec![0u32; n + 1];

    for i in 1..=m {
        curr[0] = i as u32;
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1)
                .min(curr[j - 1] + 1)
                .min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[n]
}

// ─── Template string processing ────────────────────────────────────────────

/// Simple template interpolation: replaces `{key}` with values from a map.
#[napi]
pub fn interpolate(template: String, values: HashMap<String, String>) -> String {
    let mut result = template;
    for (key, value) in &values {
        result = result.replace(&format!("{{{}}}", key), value);
    }
    result
}

/// Repeat a string n times.
#[napi]
pub fn repeat_string(s: String, count: u32) -> String {
    s.repeat(count as usize)
}

/// Truncate a string to the specified length, appending "…" if truncated.
#[napi]
pub fn truncate(s: String, max_length: u32, suffix: Option<String>) -> String {
    let max = max_length as usize;
    if s.len() <= max {
        return s;
    }
    let sfx = suffix.unwrap_or_else(|| "…".to_string());
    let cut = max.saturating_sub(sfx.len());
    let truncated: String = s.chars().take(cut).collect();
    truncated + &sfx
}

/// Remove ANSI escape codes from a string.
#[napi]
pub fn strip_ansi(s: String) -> String {
    let re = regex::Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").unwrap();
    re.replace_all(&s, "").to_string()
}

/// Normalize whitespace — collapse multiple spaces/tabs into single space.
#[napi]
pub fn normalize_whitespace(s: String) -> String {
    let re = regex::Regex::new(r"\s+").unwrap();
    re.replace_all(s.trim(), " ").to_string()
}

/// Word-wrap text to the specified column width.
#[napi]
pub fn word_wrap(text: String, width: u32) -> String {
    let max = width as usize;
    let mut result = String::with_capacity(text.len() + text.len() / max);
    for line in text.lines() {
        let mut col = 0;
        for word in line.split_whitespace() {
            if col > 0 && col + 1 + word.len() > max {
                result.push('\n');
                col = 0;
            }
            if col > 0 {
                result.push(' ');
                col += 1;
            }
            result.push_str(word);
            col += word.len();
        }
        result.push('\n');
    }
    if result.ends_with('\n') && !text.ends_with('\n') {
        result.pop();
    }
    result
}

// ─── Unicode utilities ─────────────────────────────────────────────────────

/// Check if a character is a full-width character (CJK, etc.).
#[napi]
pub fn is_full_width_character(code: u32) -> bool {
    // CJK Unified Ideographs, Hangul, Katakana, etc.
    matches!(code,
        0x1100..=0x115F |   // Hangul Jamo
        0x2E80..=0x303E |   // CJK Radicals, Kangxi, etc.
        0x3040..=0x9FFF |   // Hiragana, Katakana, CJK Unified
        0xAC00..=0xD7A3 |   // Hangul Syllables
        0xF900..=0xFAFF |   // CJK Compatibility Ideographs
        0xFE10..=0xFE1F |   // Vertical Forms
        0xFE30..=0xFE6F |   // CJK Compatibility Forms
        0xFF01..=0xFF60 |   // Fullwidth Forms
        0xFFE0..=0xFFE6 |   // Fullwidth Signs
        0x20000..=0x2FA1F   // CJK Extension B-F
    )
}

/// Get the display width of a string, accounting for full-width characters.
#[napi]
pub fn string_display_width(s: String) -> u32 {
    s.chars()
        .map(|c| if is_full_width_character(c as u32) { 2 } else { 1 })
        .sum::<u32>()
}

/// Check if a character is an emoji.
#[napi]
pub fn is_emoji(code: u32) -> bool {
    matches!(code,
        0x1F600..=0x1F64F |   // Emoticons
        0x1F300..=0x1F5FF |   // Misc Symbols & Pictographs
        0x1F680..=0x1F6FF |   // Transport & Map
        0x1F900..=0x1F9FF |   // Supplemental Symbols
        0x2600..=0x26FF |     // Misc Symbols
        0x2700..=0x27BF |     // Dingbats
        0xFE00..=0xFE0F |     // Variation Selectors
        0x1FA00..=0x1FA6F |   // Chess Symbols
        0x1FA70..=0x1FAFF |   // Symbols Extended-A
        0x200D |              // Zero Width Joiner
        0x231A..=0x231B |     // Watch, Hourglass
        0x23E9..=0x23F3 |     // Media controls
        0x23F8..=0x23FA       // Media controls
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equals_ignore_case() {
        assert!(equals_ignore_case("Hello".into(), "hello".into()));
        assert!(equals_ignore_case("WORLD".into(), "world".into()));
        assert!(!equals_ignore_case("abc".into(), "def".into()));
    }

    #[test]
    fn test_compare_natural() {
        assert_eq!(compare_natural("file2".into(), "file10".into()), -1);
        assert_eq!(compare_natural("file10".into(), "file2".into()), 1);
        assert_eq!(compare_natural("file1".into(), "file1".into()), 0);
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(
            escape_html("<div class=\"test\">&</div>".into()),
            "&lt;div class=&quot;test&quot;&gt;&amp;&lt;/div&gt;"
        );
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("hello_world".into()), "helloWorld");
        assert_eq!(to_camel_case("my-component".into()), "myComponent");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("helloWorld".into()), "hello_world");
        assert_eq!(to_snake_case("MyComponent".into()), "my_component");
    }

    #[test]
    fn test_levenshtein() {
        assert_eq!(levenshtein_distance("kitten".into(), "sitting".into()), 3);
        assert_eq!(levenshtein_distance("".into(), "abc".into()), 3);
        assert_eq!(levenshtein_distance("abc".into(), "abc".into()), 0);
    }

    #[test]
    fn test_common_prefix_length() {
        assert_eq!(common_prefix_length("abcdef".into(), "abcxyz".into()), 3);
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("Hello World".into(), 8, None), "Hello W…");
        assert_eq!(truncate("Hi".into(), 8, None), "Hi");
    }

    #[test]
    fn test_split_lines() {
        let lines = split_lines("a\nb\r\nc\n".into());
        assert_eq!(lines, vec!["a\n", "b\r\n", "c\n"]);
    }

    #[test]
    fn test_utf16_length() {
        assert_eq!(utf16_length("hello".into()), 5);
        assert_eq!(utf16_length("😀".into()), 2); // emoji is 2 UTF-16 units
    }

    #[test]
    fn test_pad_left() {
        assert_eq!(pad_left("42".into(), 5, Some("0".into())), "00042");
    }

    #[test]
    fn test_word_wrap() {
        let wrapped = word_wrap("the quick brown fox jumps".into(), 10);
        assert!(wrapped.contains('\n'));
    }
}
