/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! High-performance parallel text search engine.
//!
//! Provides fast regex and literal text search across workspace files,
//! with gitignore-aware file walking and parallel scanning via `rayon`.

use ignore::WalkBuilder;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use rayon::prelude::*;
use regex::Regex;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

/// A single search match within a file.
#[napi(object)]
#[derive(Clone)]
pub struct SearchMatch {
    /// Absolute path to the file
    pub file_path: String,
    /// 1-based line number of the match
    pub line_number: u32,
    /// 0-based column offset of the match start
    pub column: u32,
    /// The full line content containing the match
    pub line_content: String,
    /// The matched text
    pub match_text: String,
    /// Length of the match in characters
    pub match_length: u32,
}

/// Options for search operations.
#[napi(object)]
pub struct SearchOptions {
    /// Whether to use regex matching (default: false = literal)
    pub is_regex: Option<bool>,
    /// Case-insensitive search (default: false)
    pub case_insensitive: Option<bool>,
    /// Glob patterns to include (e.g., ["*.rs", "*.ts"])
    pub include_globs: Option<Vec<String>>,
    /// Glob patterns to exclude (e.g., ["node_modules/**"])
    pub exclude_globs: Option<Vec<String>>,
    /// Maximum number of results (default: 10000)
    pub max_results: Option<u32>,
    /// Whether to respect .gitignore (default: true)
    pub respect_gitignore: Option<bool>,
    /// Search only in file names, not content
    pub filename_only: Option<bool>,
    /// Maximum file size in bytes to search (default: 10MB)
    pub max_file_size: Option<u32>,
    /// Whether to match whole words only (default: false)
    pub whole_word: Option<bool>,
}

/// Result summary for a search operation.
#[napi(object)]
pub struct SearchResult {
    /// All matches found
    pub matches: Vec<SearchMatch>,
    /// Number of files scanned
    pub files_scanned: u32,
    /// Number of files with matches
    pub files_with_matches: u32,
    /// Total number of matches found
    pub total_matches: u32,
    /// Whether the search was truncated (hit max_results)
    pub truncated: bool,
    /// Duration of the search in milliseconds
    pub duration_ms: f64,
}

/// Search for text across all files in a directory.
///
/// Uses parallel file scanning for maximum performance.
/// Respects .gitignore by default.
///
/// # Arguments
/// * `directory` - Root directory to search in
/// * `query` - Search term or regex pattern
/// * `options` - Optional search configuration
#[napi]
pub fn search_files(directory: String, query: String, options: Option<SearchOptions>) -> Result<SearchResult> {
    let start = std::time::Instant::now();
    let dir_path = Path::new(&directory);

    if !dir_path.exists() || !dir_path.is_dir() {
        return Err(Error::from_reason(format!("Invalid directory: {}", directory)));
    }

    let is_regex = options.as_ref().and_then(|o| o.is_regex).unwrap_or(false);
    let case_insensitive = options.as_ref().and_then(|o| o.case_insensitive).unwrap_or(false);
    let max_results = options.as_ref().and_then(|o| o.max_results).unwrap_or(10000) as usize;
    let respect_gitignore = options.as_ref().and_then(|o| o.respect_gitignore).unwrap_or(true);
    let filename_only = options.as_ref().and_then(|o| o.filename_only).unwrap_or(false);
    let max_file_size = options.as_ref().and_then(|o| o.max_file_size).unwrap_or(10_000_000) as u64;
    let whole_word = options.as_ref().and_then(|o| o.whole_word).unwrap_or(false);

    // Build the regex pattern
    let pattern = if is_regex {
        if case_insensitive {
            format!("(?i){}", query)
        } else {
            query.clone()
        }
    } else {
        let escaped = regex::escape(&query);
        let word_bounded = if whole_word {
            format!(r"\b{}\b", escaped)
        } else {
            escaped
        };
        if case_insensitive {
            format!("(?i){}", word_bounded)
        } else {
            word_bounded
        }
    };

    let re = Regex::new(&pattern)
        .map_err(|e| Error::from_reason(format!("Invalid pattern: {}", e)))?;

    // Build the file walker
    let mut walker = WalkBuilder::new(dir_path);
    walker.git_ignore(respect_gitignore);
    walker.hidden(false);
    walker.max_filesize(Some(max_file_size));

    if let Some(opts) = &options {
        if let Some(excludes) = &opts.exclude_globs {
            let mut override_builder = ignore::overrides::OverrideBuilder::new(dir_path);
            for p in excludes {
                let _ = override_builder.add(&format!("!{}", p));
            }
            if let Ok(ovr) = override_builder.build() {
                walker.overrides(ovr);
            }
        }
    }

    // Collect all file paths first
    let files: Vec<_> = walker
        .build()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
        .map(|entry| entry.into_path())
        .collect();

    let files_scanned = files.len() as u32;

    if filename_only {
        // Search in filenames only
        let mut matches: Vec<SearchMatch> = files
            .par_iter()
            .filter_map(|path| {
                let filename = path.file_name()?.to_string_lossy();
                if let Some(m) = re.find(&filename) {
                    Some(SearchMatch {
                        file_path: path.to_string_lossy().to_string(),
                        line_number: 0,
                        column: m.start() as u32,
                        line_content: filename.to_string(),
                        match_text: m.as_str().to_string(),
                        match_length: m.len() as u32,
                    })
                } else {
                    None
                }
            })
            .collect();
        matches.truncate(max_results);

        let total = matches.len() as u32;
        let files_with = matches.iter().map(|m| &m.file_path).collect::<std::collections::HashSet<_>>().len() as u32;

        return Ok(SearchResult {
            truncated: total as usize >= max_results,
            matches,
            files_scanned,
            files_with_matches: files_with,
            total_matches: total,
            duration_ms: start.elapsed().as_secs_f64() * 1000.0,
        });
    }

    // Search file contents in parallel
    let all_matches = Mutex::new(Vec::with_capacity(256));
    let match_count = std::sync::atomic::AtomicUsize::new(0);

    files.par_iter().for_each(|path| {
        if match_count.load(std::sync::atomic::Ordering::Relaxed) >= max_results {
            return;
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return, // Skip binary/unreadable files
        };

        let mut file_matches = Vec::new();

        for (line_idx, line) in content.lines().enumerate() {
            if match_count.load(std::sync::atomic::Ordering::Relaxed) >= max_results {
                break;
            }

            for mat in re.find_iter(line) {
                file_matches.push(SearchMatch {
                    file_path: path.to_string_lossy().to_string(),
                    line_number: (line_idx + 1) as u32,
                    column: mat.start() as u32,
                    line_content: line.to_string(),
                    match_text: mat.as_str().to_string(),
                    match_length: mat.len() as u32,
                });
                match_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }

        if !file_matches.is_empty() {
            let mut all = all_matches.lock().unwrap();
            all.extend(file_matches);
        }
    });

    let matches = all_matches.into_inner().unwrap();
    let total = matches.len() as u32;
    let files_with = matches.iter().map(|m| &m.file_path).collect::<std::collections::HashSet<_>>().len() as u32;

    Ok(SearchResult {
        truncated: total as usize >= max_results,
        matches,
        files_scanned,
        files_with_matches: files_with,
        total_matches: total,
        duration_ms: start.elapsed().as_secs_f64() * 1000.0,
    })
}

/// Search for text within a single file.
///
/// # Arguments
/// * `file_path` - Absolute path to the file
/// * `query` - Search term or regex pattern
/// * `is_regex` - Whether the query is a regex
/// * `case_insensitive` - Whether to ignore case
#[napi]
pub fn search_in_file(
    file_path: String,
    query: String,
    is_regex: Option<bool>,
    case_insensitive: Option<bool>,
) -> Result<Vec<SearchMatch>> {
    let path = Path::new(&file_path);
    if !path.exists() || !path.is_file() {
        return Err(Error::from_reason(format!("File not found: {}", file_path)));
    }

    let use_regex = is_regex.unwrap_or(false);
    let ignore_case = case_insensitive.unwrap_or(false);

    let pattern = if use_regex {
        if ignore_case { format!("(?i){}", query) } else { query }
    } else {
        let escaped = regex::escape(&query);
        if ignore_case { format!("(?i){}", escaped) } else { escaped }
    };

    let re = Regex::new(&pattern)
        .map_err(|e| Error::from_reason(format!("Invalid pattern: {}", e)))?;

    let content = fs::read_to_string(path)
        .map_err(|e| Error::from_reason(format!("Failed to read {}: {}", file_path, e)))?;

    let mut matches = Vec::new();
    for (line_idx, line) in content.lines().enumerate() {
        for mat in re.find_iter(line) {
            matches.push(SearchMatch {
                file_path: file_path.clone(),
                line_number: (line_idx + 1) as u32,
                column: mat.start() as u32,
                line_content: line.to_string(),
                match_text: mat.as_str().to_string(),
                match_length: mat.len() as u32,
            });
        }
    }

    Ok(matches)
}

/// Count occurrences of a pattern in a directory (fast mode — no line details).
///
/// # Arguments
/// * `directory` - Root directory
/// * `query` - Search pattern
/// * `is_regex` - Whether the query is a regex
#[napi]
pub fn count_matches(directory: String, query: String, is_regex: Option<bool>) -> Result<u32> {
    let dir_path = Path::new(&directory);
    if !dir_path.exists() {
        return Err(Error::from_reason(format!("Directory not found: {}", directory)));
    }

    let pattern = if is_regex.unwrap_or(false) {
        query
    } else {
        regex::escape(&query)
    };

    let re = Regex::new(&pattern)
        .map_err(|e| Error::from_reason(format!("Invalid pattern: {}", e)))?;

    let files: Vec<_> = WalkBuilder::new(dir_path)
        .git_ignore(true)
        .build()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
        .map(|e| e.into_path())
        .collect();

    let count: usize = files
        .par_iter()
        .map(|path| {
            fs::read_to_string(path)
                .map(|content| re.find_iter(&content).count())
                .unwrap_or(0)
        })
        .sum();

    Ok(count as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;

    fn create_test_dir() -> PathBuf {
        let dir = std::env::temp_dir().join("ride_test_search");
        fs::create_dir_all(&dir).unwrap();

        let mut f1 = fs::File::create(dir.join("hello.rs")).unwrap();
        writeln!(f1, "fn main() {{").unwrap();
        writeln!(f1, "    println!(\"Hello, RIDE!\");").unwrap();
        writeln!(f1, "    let x = 42;").unwrap();
        writeln!(f1, "}}").unwrap();

        let mut f2 = fs::File::create(dir.join("world.ts")).unwrap();
        writeln!(f2, "const greeting = \"Hello World\";").unwrap();
        writeln!(f2, "console.log(greeting);").unwrap();
        writeln!(f2, "// Hello again").unwrap();

        let sub = dir.join("subdir");
        fs::create_dir_all(&sub).unwrap();
        let mut f3 = fs::File::create(sub.join("nested.txt")).unwrap();
        writeln!(f3, "Hello from nested file").unwrap();

        dir
    }

    #[test]
    fn test_search_literal() {
        let dir = create_test_dir();
        let result = search_files(dir.to_str().unwrap().to_string(), "Hello".to_string(), None).unwrap();
        assert!(result.total_matches >= 3);
        assert!(result.files_with_matches >= 2);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_search_case_insensitive() {
        let dir = create_test_dir();
        let result = search_files(
            dir.to_str().unwrap().to_string(),
            "hello".to_string(),
            Some(SearchOptions {
                case_insensitive: Some(true),
                is_regex: None,
                include_globs: None,
                exclude_globs: None,
                max_results: None,
                respect_gitignore: None,
                filename_only: None,
                max_file_size: None,
                whole_word: None,
            }),
        )
        .unwrap();
        assert!(result.total_matches >= 3);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_search_regex() {
        let dir = create_test_dir();
        let result = search_files(
            dir.to_str().unwrap().to_string(),
            r"\d+".to_string(),
            Some(SearchOptions {
                is_regex: Some(true),
                case_insensitive: None,
                include_globs: None,
                exclude_globs: None,
                max_results: None,
                respect_gitignore: None,
                filename_only: None,
                max_file_size: None,
                whole_word: None,
            }),
        )
        .unwrap();
        assert!(result.total_matches >= 1); // "42" in hello.rs
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_search_in_single_file() {
        let dir = create_test_dir();
        let matches = search_in_file(
            dir.join("hello.rs").to_str().unwrap().to_string(),
            "Hello".to_string(),
            None,
            None,
        )
        .unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].line_number, 2);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_count_matches() {
        let dir = create_test_dir();
        let count = count_matches(dir.to_str().unwrap().to_string(), "Hello".to_string(), None).unwrap();
        assert!(count >= 3);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_filename_search() {
        let dir = create_test_dir();
        let result = search_files(
            dir.to_str().unwrap().to_string(),
            "hello".to_string(),
            Some(SearchOptions {
                filename_only: Some(true),
                case_insensitive: Some(true),
                is_regex: None,
                include_globs: None,
                exclude_globs: None,
                max_results: None,
                respect_gitignore: None,
                max_file_size: None,
                whole_word: None,
            }),
        )
        .unwrap();
        assert!(result.total_matches >= 1);
        fs::remove_dir_all(&dir).ok();
    }
}
