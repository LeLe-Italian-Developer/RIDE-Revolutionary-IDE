/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Glob pattern matching and fuzzy scoring — Rust port of `src/vs/base/common/glob.ts`,
//! `filters.ts`, and `fuzzyScorer.ts`.
//!
//! Provides glob pattern compilation, multi-pattern matching, fuzzy string
//! matching with scoring for file pickers and command palettes.

use napi_derive::napi;
use napi::bindgen_prelude::*;
use std::collections::HashMap;

// ─── Glob matching ─────────────────────────────────────────────────────────

/// Test if a path matches a single glob pattern.
#[napi]
pub fn matches_glob(path: String, pattern: String) -> bool {
    let glob = glob::Pattern::new(&pattern);
    match glob {
        Ok(g) => g.matches(&path),
        Err(_) => {
            // Fall back to simple wildcard matching
            simple_glob_match(&path, &pattern)
        }
    }
}

/// Test if a path matches any of the given glob patterns.
#[napi]
pub fn matches_any_glob(path: String, patterns: Vec<String>) -> bool {
    patterns.iter().any(|p| matches_glob(path.clone(), p.clone()))
}

/// Test if a path matches none of the given glob patterns.
#[napi]
pub fn matches_no_glob(path: String, patterns: Vec<String>) -> bool {
    !matches_any_glob(path, patterns)
}

/// Filter an array of paths by a glob pattern.
#[napi]
pub fn filter_by_glob(paths: Vec<String>, pattern: String) -> Vec<String> {
    paths
        .into_iter()
        .filter(|p| matches_glob(p.clone(), pattern.clone()))
        .collect()
}

/// Simple glob matching without the glob crate — supports * and ? wildcards.
fn simple_glob_match(text: &str, pattern: &str) -> bool {
    let t: Vec<char> = text.chars().collect();
    let p: Vec<char> = pattern.chars().collect();
    let (tlen, plen) = (t.len(), p.len());
    let mut ti = 0;
    let mut pi = 0;
    let mut star_pi = None;
    let mut star_ti = 0;

    while ti < tlen {
        if pi < plen && (p[pi] == '?' || p[pi] == t[ti]) {
            ti += 1;
            pi += 1;
        } else if pi < plen && p[pi] == '*' {
            star_pi = Some(pi);
            star_ti = ti;
            pi += 1;
        } else if let Some(sp) = star_pi {
            pi = sp + 1;
            star_ti += 1;
            ti = star_ti;
        } else {
            return false;
        }
    }

    while pi < plen && p[pi] == '*' {
        pi += 1;
    }
    pi == plen
}

/// Parse a glob expression with brace expansion, e.g., `*.{ts,js}`.
#[napi]
pub fn expand_braces(pattern: String) -> Vec<String> {
    if let (Some(start), Some(end)) = (pattern.find('{'), pattern.rfind('}')) {
        let prefix = &pattern[..start];
        let suffix = &pattern[end + 1..];
        let alternatives = &pattern[start + 1..end];

        alternatives
            .split(',')
            .map(|alt| format!("{}{}{}", prefix, alt.trim(), suffix))
            .collect()
    } else {
        vec![pattern]
    }
}

/// Create a negated glob pattern (matches paths NOT matching the pattern).
#[napi]
pub fn negate_glob(pattern: String) -> String {
    if pattern.starts_with('!') {
        pattern[1..].to_string()
    } else {
        format!("!{}", pattern)
    }
}

// ─── Fuzzy matching ────────────────────────────────────────────────────────

/// Result of a fuzzy match computation.
#[napi(object)]
pub struct GlobFuzzyResult {
    /// Matching score (0 = no match, higher = better match).
    pub score: f64,
    /// Indices of matched characters in the target string.
    pub matches: Vec<u32>,
}

/// Fuzzy match a query against a target string.
/// Returns a score and the positions of matched characters.
/// Score is 0 if there's no match.
#[napi]
pub fn glob_fuzzy_match(query: String, target: String) -> GlobFuzzyResult {
    let query_lower: Vec<char> = query.to_lowercase().chars().collect();
    let target_lower: Vec<char> = target.to_lowercase().chars().collect();
    let target_chars: Vec<char> = target.chars().collect();

    if query_lower.is_empty() {
        return GlobFuzzyResult { score: 1.0, matches: Vec::new() };
    }

    if query_lower.len() > target_lower.len() {
        return GlobFuzzyResult { score: 0.0, matches: Vec::new() };
    }

    let mut qi = 0;
    let mut matches = Vec::new();
    let mut score: f64 = 0.0;
    let mut prev_match_idx: Option<usize> = None;
    let mut consecutive_bonus: f64 = 0.0;

    for (ti, &tc) in target_lower.iter().enumerate() {
        if qi < query_lower.len() && tc == query_lower[qi] {
            matches.push(ti as u32);

            // Base score for a match
            score += 1.0;

            // Bonus for consecutive matches
            if let Some(prev) = prev_match_idx {
                if ti == prev + 1 {
                    consecutive_bonus += 5.0;
                    score += consecutive_bonus;
                } else {
                    consecutive_bonus = 0.0;
                }
            }

            // Bonus for matching at word boundary (after _, -, space, or camelCase)
            if ti == 0 {
                score += 10.0; // Start of string
            } else {
                let prev_char = target_chars[ti - 1];
                if prev_char == '_' || prev_char == '-' || prev_char == ' ' || prev_char == '/' || prev_char == '\\' {
                    score += 8.0; // Word boundary
                } else if prev_char.is_lowercase() && target_chars[ti].is_uppercase() {
                    score += 7.0; // camelCase boundary
                }
            }

            // Bonus for exact case match
            if target_chars[ti] == query.chars().nth(qi).unwrap_or(' ') {
                score += 1.0;
            }

            prev_match_idx = Some(ti);
            qi += 1;
        }
    }

    if qi < query_lower.len() {
        // Not all query characters matched
        return GlobFuzzyResult { score: 0.0, matches: Vec::new() };
    }

    // Normalize score by target length (shorter targets score higher)
    score /= target_lower.len() as f64;
    score *= 100.0; // Scale to 0-100 range

    GlobFuzzyResult { score, matches }
}

/// Score multiple targets against a query and return sorted results.
#[napi]
pub fn fuzzy_score_sorted(query: String, targets: Vec<String>, max_results: Option<u32>) -> Vec<GlobFuzzyResult> {
    let max = max_results.unwrap_or(50) as usize;

    let mut scored: Vec<(usize, GlobFuzzyResult)> = targets
        .iter()
        .enumerate()
        .map(|(i, t)| (i, glob_fuzzy_match(query.clone(), t.clone())))
        .filter(|(_, r)| r.score > 0.0)
        .collect();

    scored.sort_by(|a, b| b.1.score.partial_cmp(&a.1.score).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(max);

    scored.into_iter().map(|(_, r)| r).collect()
}

/// Quick filter — returns only items that fuzzy match the query.
#[napi]
pub fn fuzzy_filter(query: String, items: Vec<String>) -> Vec<String> {
    items
        .into_iter()
        .filter(|item| glob_fuzzy_match(query.clone(), item.clone()).score > 0.0)
        .collect()
}

// ─── Path-aware matching ───────────────────────────────────────────────────

/// Match a query against the basename of a file path (for file pickers).
#[napi]
pub fn glob_fuzzy_match_path(query: String, file_path: String) -> GlobFuzzyResult {
    // Extract basename
    let basename = file_path
        .rfind('/')
        .or_else(|| file_path.rfind('\\'))
        .map(|pos| &file_path[pos + 1..])
        .unwrap_or(&file_path);

    let mut result = glob_fuzzy_match(query.clone(), basename.to_string());

    // Give a small bonus if the directory path also partially matches
    if result.score > 0.0 {
        let dir = &file_path[..file_path.len() - basename.len()];
        let dir_match = glob_fuzzy_match(query, dir.to_string());
        if dir_match.score > 0.0 {
            result.score += dir_match.score * 0.2; // Small boost for directory match
        }
    }

    result
}

/// Score and sort file paths for a fuzzy file picker.
#[napi]
pub fn fuzzy_pick_files(query: String, paths: Vec<String>, max_results: Option<u32>) -> Vec<String> {
    let max = max_results.unwrap_or(50) as usize;

    let mut scored: Vec<(String, f64)> = paths
        .into_iter()
        .map(|p| {
            let score = glob_fuzzy_match_path(query.clone(), p.clone()).score;
            (p, score)
        })
        .filter(|(_, score)| *score > 0.0)
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(max);

    scored.into_iter().map(|(p, _)| p).collect()
}

// ─── Highlight utilities ───────────────────────────────────────────────────

/// Create highlight ranges from fuzzy match positions.
/// Returns an array of [start, end] pairs for consecutive matched positions.
#[napi]
pub fn create_highlight_ranges(match_positions: Vec<u32>) -> Vec<Vec<u32>> {
    if match_positions.is_empty() {
        return Vec::new();
    }

    let mut ranges = Vec::new();
    let mut range_start = match_positions[0];
    let mut range_end = match_positions[0];

    for &pos in match_positions.iter().skip(1) {
        if pos == range_end + 1 {
            range_end = pos;
        } else {
            ranges.push(vec![range_start, range_end + 1]);
            range_start = pos;
            range_end = pos;
        }
    }
    ranges.push(vec![range_start, range_end + 1]);
    ranges
}

// ─── Word-level matching ────────────────────────────────────────────────────

/// Split a camelCase or snake_case string into words.
#[napi]
pub fn split_identifier_words(identifier: String) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();

    for ch in identifier.chars() {
        if ch == '_' || ch == '-' || ch == ' ' || ch == '.' {
            if !current.is_empty() {
                words.push(current.clone());
                current.clear();
            }
        } else if ch.is_uppercase() && !current.is_empty() && current.chars().last().map_or(false, |c| c.is_lowercase()) {
            words.push(current.clone());
            current.clear();
            current.push(ch);
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

/// Match a query against multiple words of an identifier.
#[napi]
pub fn word_match(query: String, identifier: String) -> bool {
    let words = split_identifier_words(identifier);
    let query_words = split_identifier_words(query);

    let mut wi = 0;
    for qw in &query_words {
        let ql = qw.to_lowercase();
        let mut found = false;
        while wi < words.len() {
            if words[wi].to_lowercase().starts_with(&ql) {
                found = true;
                wi += 1;
                break;
            }
            wi += 1;
        }
        if !found {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_glob() {
        assert!(simple_glob_match("hello.ts", "*.ts"));
        assert!(simple_glob_match("hello.ts", "hello.*"));
        assert!(!simple_glob_match("hello.rs", "*.ts"));
        assert!(simple_glob_match("abc", "a?c"));
    }

    #[test]
    fn test_expand_braces() {
        let expanded = expand_braces("*.{ts,js,rs}".into());
        assert_eq!(expanded, vec!["*.ts", "*.js", "*.rs"]);
    }

    #[test]
    fn test_glob_fuzzy_match() {
        let result = glob_fuzzy_match("abc".into(), "aXbYcZ".into());
        assert!(result.score > 0.0);
        assert_eq!(result.matches, vec![0, 2, 4]);

        let no_match = glob_fuzzy_match("xyz".into(), "abc".into());
        assert_eq!(no_match.score, 0.0);
    }

    #[test]
    fn test_glob_fuzzy_match_ordering() {
        // Exact prefix match should score higher than scattered match
        let exact = glob_fuzzy_match("foo".into(), "fooBar".into());
        let scattered = glob_fuzzy_match("foo".into(), "fXoYoZ".into());
        assert!(exact.score > scattered.score);
    }

    #[test]
    fn test_split_identifier_words() {
        assert_eq!(split_identifier_words("camelCase".into()), vec!["camel", "Case"]);
        assert_eq!(split_identifier_words("snake_case".into()), vec!["snake", "case"]);
        assert_eq!(split_identifier_words("kebab-case".into()), vec!["kebab", "case"]);
    }

    #[test]
    fn test_word_match() {
        assert!(word_match("cm".into(), "camelMatch".into()));
        assert!(word_match("gFI".into(), "getFileInfo".into()));
        assert!(!word_match("xyz".into(), "getFileInfo".into()));
    }

    #[test]
    fn test_create_highlight_ranges() {
        let ranges = create_highlight_ranges(vec![0, 1, 2, 5, 6, 9]);
        assert_eq!(ranges, vec![vec![0, 3], vec![5, 7], vec![9, 10]]);
    }

    #[test]
    fn test_filter_by_glob() {
        let paths = vec!["a.ts".into(), "b.rs".into(), "c.ts".into()];
        let filtered = filter_by_glob(paths, "*.ts".into());
        assert_eq!(filtered, vec!["a.ts", "c.ts"]);
    }
}
