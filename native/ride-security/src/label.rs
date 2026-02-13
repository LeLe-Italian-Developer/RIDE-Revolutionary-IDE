/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Label and display utilities — Rust port of `src/vs/base/common/labels.ts`.
//! Path label formatting, file size formatting, and display helpers.

use napi_derive::napi;

#[napi]
pub fn format_file_size(bytes: f64) -> String {
    if bytes < 1024.0 { return format!("{} B", bytes as u64); }
    if bytes < 1_048_576.0 { return format!("{:.1} KB", bytes / 1024.0); }
    if bytes < 1_073_741_824.0 { return format!("{:.1} MB", bytes / 1_048_576.0); }
    if bytes < 1_099_511_627_776.0 { return format!("{:.1} GB", bytes / 1_073_741_824.0); }
    format!("{:.1} TB", bytes / 1_099_511_627_776.0)
}

#[napi]
pub fn format_count(n: f64) -> String {
    if n < 1000.0 { return format!("{}", n as u64); }
    if n < 1_000_000.0 { return format!("{:.1}K", n / 1000.0); }
    if n < 1_000_000_000.0 { return format!("{:.1}M", n / 1_000_000.0); }
    format!("{:.1}B", n / 1_000_000_000.0)
}

/// Shorten a file path for display by collapsing middle segments.
#[napi]
pub fn shorten_path(path: String, max_length: u32) -> String {
    let max = max_length as usize;
    if path.len() <= max { return path; }
    let sep = if path.contains('\\') { '\\' } else { '/' };
    let parts: Vec<&str> = path.split(sep).collect();
    if parts.len() <= 2 { return path; }
    let first = parts[0];
    let last = parts[parts.len() - 1];
    let ellipsis = format!("{}…{}{}", first, sep, last);
    if ellipsis.len() >= max { return format!("…{}{}", sep, last); }
    ellipsis
}

/// Tildify a home directory path (replace home dir with ~).
#[napi]
pub fn tildify(path: String, home: String) -> String {
    let normalized_home = if home.ends_with('/') || home.ends_with('\\') { home.clone() } else { format!("{}/", home) };
    let normalized_path = path.replace('\\', "/");
    let norm_home = normalized_home.replace('\\', "/");
    if normalized_path.starts_with(&norm_home) {
        format!("~/{}", &normalized_path[norm_home.len()..])
    } else if normalized_path == home.replace('\\', "/") {
        "~".to_string()
    } else {
        path
    }
}

/// Untildify replaces ~ with actual home path.
#[napi]
pub fn untildify(path: String, home: String) -> String {
    if path.starts_with("~/") { format!("{}/{}", home.trim_end_matches('/'), &path[2..]) }
    else if path == "~" { home }
    else { path }
}

/// Create a path label with optional workspace root removal.
#[napi]
pub fn path_label(path: String, root: Option<String>) -> String {
    match root {
        Some(r) => {
            let normalized = path.replace('\\', "/");
            let norm_root = if r.ends_with('/') { r.replace('\\', "/") } else { format!("{}/", r.replace('\\', "/")) };
            if normalized.starts_with(&norm_root) { normalized[norm_root.len()..].to_string() }
            else { path }
        }
        None => path,
    }
}

/// Format a percentage.
#[napi]
pub fn format_percentage(value: f64, decimals: Option<u32>) -> String {
    let d = decimals.unwrap_or(1);
    format!("{:.prec$}%", value * 100.0, prec = d as usize)
}

/// Pluralize a word based on count.
#[napi]
pub fn pluralize(count: u32, singular: String, plural: Option<String>) -> String {
    let p = plural.unwrap_or_else(|| format!("{}s", singular));
    if count == 1 { format!("{} {}", count, singular) } else { format!("{} {}", count, p) }
}

/// Truncate to fit into max width, adding ellipsis.
#[napi]
pub fn ellipsis_middle(text: String, max_length: u32) -> String {
    let max = max_length as usize;
    if text.len() <= max { return text; }
    let half = (max - 1) / 2;
    let start: String = text.chars().take(half).collect();
    let end: String = text.chars().rev().take(half).collect::<Vec<_>>().into_iter().rev().collect();
    format!("{}…{}", start, end)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_file_size() {
        assert_eq!(format_file_size(500.0), "500 B");
        assert_eq!(format_file_size(1536.0), "1.5 KB");
        assert_eq!(format_file_size(1_500_000.0), "1.4 MB");
    }
    #[test]
    fn test_tildify() {
        assert_eq!(tildify("/home/user/docs".into(), "/home/user".into()), "~/docs");
    }
    #[test]
    fn test_pluralize() {
        assert_eq!(pluralize(1, "file".into(), None), "1 file");
        assert_eq!(pluralize(5, "file".into(), None), "5 files");
    }
}
