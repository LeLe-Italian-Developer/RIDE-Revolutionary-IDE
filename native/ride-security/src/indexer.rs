/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Fast workspace file indexer with fuzzy matching.
//!
//! Provides parallel directory traversal and fuzzy file name matching
//! for quick-open (Ctrl+P) functionality, with metadata caching.

use ignore::WalkBuilder;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, RwLock};
use std::time::UNIX_EPOCH;

/// File metadata for indexed files.
#[napi(object)]
#[derive(Clone)]
pub struct FileInfo {
    /// Absolute path to the file
    pub path: String,
    /// File name (basename)
    pub name: String,
    /// File extension (without dot)
    pub extension: String,
    /// File size in bytes
    pub size: f64,
    /// Last modified time as Unix timestamp (seconds)
    pub modified: f64,
    /// Whether it's a directory
    pub is_directory: bool,
    /// Relative path from workspace root
    pub relative_path: String,
    /// Directory depth from root
    pub depth: u32,
}

/// Fuzzy match result with scoring.
#[napi(object)]
#[derive(Clone)]
pub struct FuzzyMatchResult {
    /// The matched file info
    pub file: FileInfo,
    /// Match score (higher = better match, 0-1000)
    pub score: u32,
    /// Indices of matched characters in the filename
    pub matched_indices: Vec<u32>,
}

/// Index statistics.
#[napi(object)]
pub struct IndexStats {
    /// Total number of indexed files
    pub total_files: u32,
    /// Total number of indexed directories
    pub total_directories: u32,
    /// Total size of all indexed files in bytes
    pub total_size: f64,
    /// Time taken to build index in milliseconds
    pub build_time_ms: f64,
    /// Number of file extensions found
    pub unique_extensions: u32,
}

static WORKSPACE_INDEX: RwLock<Option<Vec<FileInfo>>> = RwLock::new(None);
static INDEX_ROOT: RwLock<Option<String>> = RwLock::new(None);

/// Build the workspace file index.
///
/// Scans the directory tree in parallel, respecting .gitignore.
///
/// # Arguments
/// * `root_directory` - Absolute path to the workspace root
///
/// # Returns
/// Index statistics
#[napi]
pub fn index_workspace(root_directory: String) -> Result<IndexStats> {
    let start = std::time::Instant::now();
    let root = Path::new(&root_directory);

    if !root.exists() || !root.is_dir() {
        return Err(Error::from_reason(format!("Invalid directory: {}", root_directory)));
    }

    let entries: Vec<_> = WalkBuilder::new(root)
        .git_ignore(true)
        .hidden(false)
        .build()
        .filter_map(|e| e.ok())
        .collect();

    let root_path = root.to_path_buf();
    let files: Vec<FileInfo> = entries
        .par_iter()
        .filter_map(|entry| {
            let path = entry.path();
            let metadata = path.metadata().ok()?;
            let is_dir = metadata.is_dir();
            let size = if is_dir { 0 } else { metadata.len() };
            let modified = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0);

            let relative = path
                .strip_prefix(&root_path)
                .ok()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            let depth = relative.matches('/').count() as u32
                + relative.matches('\\').count() as u32;

            Some(FileInfo {
                path: path.to_string_lossy().to_string(),
                name: path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
                extension: path
                    .extension()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or_default(),
                size: size as f64,
                modified,
                is_directory: is_dir,
                relative_path: relative,
                depth,
            })
        })
        .collect();

    let total_files = files.iter().filter(|f| !f.is_directory).count() as u32;
    let total_dirs = files.iter().filter(|f| f.is_directory).count() as u32;
    let total_size: f64 = files.iter().map(|f| f.size).sum();
    let extensions: std::collections::HashSet<_> = files
        .iter()
        .filter(|f| !f.extension.is_empty())
        .map(|f| &f.extension)
        .collect();

    let stats = IndexStats {
        total_files,
        total_directories: total_dirs,
        total_size,
        build_time_ms: start.elapsed().as_secs_f64() * 1000.0,
        unique_extensions: extensions.len() as u32,
    };

    // Store the index
    {
        let mut idx = WORKSPACE_INDEX.write().unwrap();
        *idx = Some(files);
    }
    {
        let mut r = INDEX_ROOT.write().unwrap();
        *r = Some(root_directory);
    }

    Ok(stats)
}

/// Fuzzy match a query against the workspace file index.
///
/// Returns files ranked by how well they match the query,
/// similar to VS Code's Ctrl+P behavior.
///
/// # Arguments
/// * `query` - The fuzzy search query (e.g., "mncr" matches "mainController")
/// * `max_results` - Maximum number of results to return (default: 50)
/// * `files_only` - Whether to exclude directories (default: true)
#[napi]
pub fn fuzzy_match(query: String, max_results: Option<u32>, files_only: Option<bool>) -> Vec<FuzzyMatchResult> {
    let max = max_results.unwrap_or(50) as usize;
    let only_files = files_only.unwrap_or(true);

    let index = WORKSPACE_INDEX.read().unwrap();
    let files = match index.as_ref() {
        Some(f) => f,
        None => return Vec::new(),
    };

    let query_lower = query.to_lowercase();
    let query_chars: Vec<char> = query_lower.chars().collect();

    if query_chars.is_empty() {
        // Return most recently modified files
        let mut sorted: Vec<_> = files
            .iter()
            .filter(|f| !only_files || !f.is_directory)
            .cloned()
            .collect();
        sorted.sort_by(|a, b| b.modified.partial_cmp(&a.modified).unwrap_or(std::cmp::Ordering::Equal));
        return sorted
            .into_iter()
            .take(max)
            .map(|file| FuzzyMatchResult {
                file,
                score: 0,
                matched_indices: Vec::new(),
            })
            .collect();
    }

    let results = Mutex::new(Vec::new());

    files.par_iter().for_each(|file| {
        if only_files && file.is_directory {
            return;
        }

        let name_lower = file.name.to_lowercase();
        let path_lower = file.relative_path.to_lowercase();

        // Try matching against filename first (higher score), then full path
        if let Some((score, indices)) = fuzzy_score(&query_chars, &name_lower) {
            let bonus = 200; // Bonus for filename match
            let mut res = results.lock().unwrap();
            res.push(FuzzyMatchResult {
                file: file.clone(),
                score: (score + bonus).min(1000),
                matched_indices: indices,
            });
        } else if let Some((score, indices)) = fuzzy_score(&query_chars, &path_lower) {
            let mut res = results.lock().unwrap();
            res.push(FuzzyMatchResult {
                file: file.clone(),
                score: score.min(1000),
                matched_indices: indices,
            });
        }
    });

    let mut all_results = results.into_inner().unwrap();
    all_results.sort_by(|a, b| b.score.cmp(&a.score));
    all_results.truncate(max);
    all_results
}

/// Internal fuzzy scoring function.
/// Returns (score, matched_indices) or None if no match.
fn fuzzy_score(query: &[char], target: &str) -> Option<(u32, Vec<u32>)> {
    let target_chars: Vec<char> = target.chars().collect();
    let mut score: u32 = 0;
    let mut indices = Vec::with_capacity(query.len());
    let mut target_idx = 0;
    let mut prev_matched = false;
    let mut prev_was_separator = true; // Start of string counts as separator

    for &q_char in query {
        let mut found = false;
        while target_idx < target_chars.len() {
            let t_char = target_chars[target_idx];
            let is_separator = t_char == '/' || t_char == '\\' || t_char == '.' || t_char == '_' || t_char == '-' || t_char == ' ';

            if t_char == q_char {
                indices.push(target_idx as u32);

                // Scoring bonuses
                if prev_was_separator {
                    score += 50; // Word boundary match
                }
                if prev_matched {
                    score += 30; // Consecutive match
                }
                if target_idx == 0 {
                    score += 100; // First character match
                }
                score += 10; // Base match

                prev_matched = true;
                prev_was_separator = is_separator;
                target_idx += 1;
                found = true;
                break;
            }

            prev_matched = false;
            prev_was_separator = is_separator;
            target_idx += 1;
        }

        if !found {
            return None;
        }
    }

    // Bonus for shorter targets (prefer shorter file names)
    let length_penalty = target_chars.len() as u32;
    score = score.saturating_sub(length_penalty / 2);

    // Bonus for closer match ratio
    let ratio = (query.len() as f64) / (target_chars.len() as f64);
    score += (ratio * 100.0) as u32;

    Some((score, indices))
}

/// Get metadata for a specific file from the index.
///
/// # Arguments
/// * `file_path` - Absolute or relative path to look up
#[napi]
pub fn get_file_metadata(file_path: String) -> Option<FileInfo> {
    let index = WORKSPACE_INDEX.read().unwrap();
    index.as_ref().and_then(|files| {
        files.iter().find(|f| f.path == file_path || f.relative_path == file_path).cloned()
    })
}

/// Get files grouped by extension.
///
/// Returns a map of extension -> count.
#[napi]
pub fn get_extension_stats() -> HashMap<String, u32> {
    let index = WORKSPACE_INDEX.read().unwrap();
    let mut stats = HashMap::new();

    if let Some(files) = index.as_ref() {
        for file in files.iter().filter(|f| !f.is_directory && !f.extension.is_empty()) {
            *stats.entry(file.extension.clone()).or_insert(0) += 1;
        }
    }

    stats
}

/// Get the total number of indexed items.
#[napi]
pub fn get_index_size() -> u32 {
    let index = WORKSPACE_INDEX.read().unwrap();
    index.as_ref().map(|f| f.len() as u32).unwrap_or(0)
}

/// Clear the workspace index.
#[napi]
pub fn clear_index() {
    let mut idx = WORKSPACE_INDEX.write().unwrap();
    *idx = None;
    let mut r = INDEX_ROOT.write().unwrap();
    *r = None;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;

    fn create_test_workspace() -> PathBuf {
        let dir = std::env::temp_dir().join("ride_test_indexer");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        fs::File::create(dir.join("main.rs")).unwrap().write_all(b"fn main() {}").unwrap();
        fs::File::create(dir.join("config.json")).unwrap().write_all(b"{}").unwrap();
        fs::File::create(dir.join("README.md")).unwrap().write_all(b"# Test").unwrap();

        let sub = dir.join("src");
        fs::create_dir_all(&sub).unwrap();
        fs::File::create(sub.join("lib.rs")).unwrap().write_all(b"mod test;").unwrap();
        fs::File::create(sub.join("utils.ts")).unwrap().write_all(b"export {}").unwrap();

        dir
    }

    #[test]
    fn test_index_workspace() {
        let dir = create_test_workspace();
        let stats = index_workspace(dir.to_str().unwrap().to_string()).unwrap();
        assert!(stats.total_files >= 5);
        assert!(stats.unique_extensions >= 3);
        assert!(stats.build_time_ms >= 0.0);
        clear_index();
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_fuzzy_matching() {
        let dir = create_test_workspace();
        index_workspace(dir.to_str().unwrap().to_string()).unwrap();

        let results = fuzzy_match("main".to_string(), None, None);
        assert!(!results.is_empty());
        assert!(results[0].file.name.contains("main"));

        clear_index();
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_fuzzy_score_basic() {
        let query: Vec<char> = "mn".chars().collect();
        let result = fuzzy_score(&query, "main");
        assert!(result.is_some());
        let (score, indices) = result.unwrap();
        assert!(score > 0);
        assert_eq!(indices.len(), 2);
    }

    #[test]
    fn test_fuzzy_score_no_match() {
        let query: Vec<char> = "xyz".chars().collect();
        let result = fuzzy_score(&query, "main");
        assert!(result.is_none());
    }

    #[test]
    fn test_extension_stats() {
        let dir = create_test_workspace();
        index_workspace(dir.to_str().unwrap().to_string()).unwrap();
        let stats = get_extension_stats();
        assert!(stats.contains_key("rs"));
        assert!(*stats.get("rs").unwrap() >= 2);
        clear_index();
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_empty_query_returns_recent() {
        let dir = create_test_workspace();
        index_workspace(dir.to_str().unwrap().to_string()).unwrap();
        let results = fuzzy_match("".to_string(), Some(3), None);
        assert!(!results.is_empty());
        assert!(results.len() <= 3);
        clear_index();
        fs::remove_dir_all(&dir).ok();
    }
}
