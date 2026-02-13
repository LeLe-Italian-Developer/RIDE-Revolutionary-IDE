/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! High-performance text diff engine using the Myers algorithm.
//!
//! Provides line-level and word-level diffing for the editor and SCM views,
//! with unified diff output format support.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use similar::{ChangeTag, TextDiff, Algorithm};

/// Represents a single diff change (add, remove, or equal).
#[napi(object)]
#[derive(Clone)]
pub struct DiffChange {
    /// The type of change: "add", "remove", or "equal"
    pub tag: String,
    /// The text content of this change
    pub content: String,
    /// Line number in the old text (for "remove" and "equal")
    pub old_line: Option<u32>,
    /// Line number in the new text (for "add" and "equal")
    pub new_line: Option<u32>,
}

/// Summary statistics for a diff operation.
#[napi(object)]
pub struct DiffStats {
    /// Number of lines added
    pub additions: u32,
    /// Number of lines removed
    pub deletions: u32,
    /// Number of unchanged lines
    pub unchanged: u32,
    /// Total number of changes (additions + deletions)
    pub total_changes: u32,
}

/// Result of a diff operation.
#[napi(object)]
pub struct DiffResult {
    /// Individual changes
    pub changes: Vec<DiffChange>,
    /// Summary statistics
    pub stats: DiffStats,
    /// Unified diff string output
    pub unified_diff: String,
}

/// Word-level diff change.
#[napi(object)]
#[derive(Clone)]
pub struct WordDiffChange {
    /// The type of change: "add", "remove", or "equal"
    pub tag: String,
    /// The word or text fragment
    pub content: String,
}

fn tag_to_string(tag: ChangeTag) -> String {
    match tag {
        ChangeTag::Insert => "add".to_string(),
        ChangeTag::Delete => "remove".to_string(),
        ChangeTag::Equal => "equal".to_string(),
    }
}

/// Compute a line-level diff between two texts.
///
/// Uses the Myers diff algorithm for optimal results.
///
/// # Arguments
/// * `old_text` - The original text
/// * `new_text` - The modified text
///
/// # Returns
/// A `DiffResult` with individual changes, stats, and unified diff output
#[napi]
pub fn compute_diff(old_text: String, new_text: String) -> DiffResult {
    let diff = TextDiff::configure()
        .algorithm(Algorithm::Myers)
        .diff_lines(&old_text, &new_text);

    let mut changes = Vec::new();
    let mut additions: u32 = 0;
    let mut deletions: u32 = 0;
    let mut unchanged: u32 = 0;

    for change in diff.iter_all_changes() {
        let tag = change.tag();
        match tag {
            ChangeTag::Insert => additions += 1,
            ChangeTag::Delete => deletions += 1,
            ChangeTag::Equal => unchanged += 1,
        }

        changes.push(DiffChange {
            tag: tag_to_string(tag),
            content: change.value().to_string(),
            old_line: change.old_index().map(|i| (i + 1) as u32),
            new_line: change.new_index().map(|i| (i + 1) as u32),
        });
    }

    let unified = diff
        .unified_diff()
        .context_radius(3)
        .header("a", "b")
        .to_string();

    DiffResult {
        changes,
        stats: DiffStats {
            additions,
            deletions,
            unchanged,
            total_changes: additions + deletions,
        },
        unified_diff: unified,
    }
}

/// Compute a word-level diff between two strings (useful for inline diffs).
///
/// # Arguments
/// * `old_text` - The original text
/// * `new_text` - The modified text
///
/// # Returns
/// Array of word-level changes
#[napi]
pub fn compute_word_diff(old_text: String, new_text: String) -> Vec<WordDiffChange> {
    let diff = TextDiff::configure()
        .algorithm(Algorithm::Myers)
        .diff_words(&old_text, &new_text);

    diff.iter_all_changes()
        .map(|change| WordDiffChange {
            tag: tag_to_string(change.tag()),
            content: change.value().to_string(),
        })
        .collect()
}

/// Compute a character-level diff between two strings (finest granularity).
///
/// # Arguments
/// * `old_text` - The original text
/// * `new_text` - The modified text
///
/// # Returns
/// Array of character-level changes
#[napi]
pub fn compute_char_diff(old_text: String, new_text: String) -> Vec<WordDiffChange> {
    let diff = TextDiff::configure()
        .algorithm(Algorithm::Myers)
        .diff_chars(&old_text, &new_text);

    diff.iter_all_changes()
        .map(|change| WordDiffChange {
            tag: tag_to_string(change.tag()),
            content: change.value().to_string(),
        })
        .collect()
}

/// Apply a simple unified patch to text.
///
/// This is a basic patch applicator that handles simple add/remove operations.
///
/// # Arguments
/// * `original` - The original text
/// * `changes` - Array of DiffChange objects to apply
///
/// # Returns
/// The patched text
#[napi]
pub fn apply_patch(original: String, changes: Vec<DiffChange>) -> Result<String> {
    let mut result = Vec::new();
    let original_lines: Vec<&str> = original.lines().collect();
    let mut old_idx = 0;

    for change in &changes {
        match change.tag.as_str() {
            "equal" => {
                if old_idx < original_lines.len() {
                    result.push(original_lines[old_idx].to_string());
                    old_idx += 1;
                }
            }
            "remove" => {
                old_idx += 1; // Skip this line
            }
            "add" => {
                result.push(change.content.trim_end_matches('\n').to_string());
            }
            _ => {}
        }
    }

    // Append any remaining original lines
    while old_idx < original_lines.len() {
        result.push(original_lines[old_idx].to_string());
        old_idx += 1;
    }

    Ok(result.join("\n"))
}

/// Get diff statistics without computing full changes (faster).
///
/// # Arguments
/// * `old_text` - The original text
/// * `new_text` - The modified text
#[napi]
pub fn diff_stats(old_text: String, new_text: String) -> DiffStats {
    let diff = TextDiff::configure()
        .algorithm(Algorithm::Myers)
        .diff_lines(&old_text, &new_text);

    let mut additions: u32 = 0;
    let mut deletions: u32 = 0;
    let mut unchanged: u32 = 0;

    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Insert => additions += 1,
            ChangeTag::Delete => deletions += 1,
            ChangeTag::Equal => unchanged += 1,
        }
    }

    DiffStats {
        additions,
        deletions,
        unchanged,
        total_changes: additions + deletions,
    }
}

/// Compute the similarity ratio between two texts (0.0 to 1.0).
///
/// # Arguments
/// * `text_a` - First text
/// * `text_b` - Second text
///
/// # Returns
/// Similarity ratio where 1.0 means identical and 0.0 means completely different
#[napi]
pub fn similarity_ratio(text_a: String, text_b: String) -> f64 {
    let diff = TextDiff::configure()
        .algorithm(Algorithm::Myers)
        .diff_chars(&text_a, &text_b);
    diff.ratio().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_diff_additions() {
        let old = "line1\nline2\n";
        let new = "line1\nline2\nline3\n";
        let result = compute_diff(old.to_string(), new.to_string());
        assert_eq!(result.stats.additions, 1);
        assert_eq!(result.stats.deletions, 0);
    }

    #[test]
    fn test_compute_diff_deletions() {
        let old = "line1\nline2\nline3\n";
        let new = "line1\nline3\n";
        let result = compute_diff(old.to_string(), new.to_string());
        assert_eq!(result.stats.deletions, 1);
    }

    #[test]
    fn test_word_diff() {
        let old = "The quick brown fox";
        let new = "The slow brown bear";
        let changes = compute_word_diff(old.to_string(), new.to_string());
        let added: Vec<_> = changes.iter().filter(|c| c.tag == "add").collect();
        let removed: Vec<_> = changes.iter().filter(|c| c.tag == "remove").collect();
        assert!(!added.is_empty());
        assert!(!removed.is_empty());
    }

    #[test]
    fn test_similarity_identical() {
        let ratio = similarity_ratio("hello".to_string(), "hello".to_string());
        assert!((ratio - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_similarity_different() {
        let ratio = similarity_ratio("abc".to_string(), "xyz".to_string());
        assert!(ratio < 0.5);
    }

    #[test]
    fn test_unified_diff_output() {
        let old = "line1\nline2\n";
        let new = "line1\nmodified\n";
        let result = compute_diff(old.to_string(), new.to_string());
        assert!(result.unified_diff.contains("---"));
        assert!(result.unified_diff.contains("+++"));
    }

    #[test]
    fn test_diff_stats_fast() {
        let old = "a\nb\nc\n";
        let new = "a\nx\nc\n";
        let stats = diff_stats(old.to_string(), new.to_string());
        assert_eq!(stats.additions, 1);
        assert_eq!(stats.deletions, 1);
        assert_eq!(stats.unchanged, 2);
    }
}
