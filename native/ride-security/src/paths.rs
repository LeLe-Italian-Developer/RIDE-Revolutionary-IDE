/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Path and URI manipulation utilities — Rust port of `src/vs/base/common/path.ts`,
//! `uri.ts`, `extpath.ts`, and `network.ts`.
//!
//! Provides cross-platform path handling, URI parsing/building, path normalization,
//! relative path computation, and resource URI operations.

use napi_derive::napi;
use napi::bindgen_prelude::*;

// ─── Path constants ────────────────────────────────────────────────────────

const SLASH: char = '/';
const BACKSLASH: char = '\\';

/// Path separator for the current platform.
#[napi]
pub fn path_separator() -> String {
    if cfg!(windows) { "\\".to_string() } else { "/".to_string() }
}

// ─── Path normalization ────────────────────────────────────────────────────

/// Normalize a path — resolve `.`, `..`, and duplicate separators.
#[napi]
pub fn normalize_path(path: String) -> String {
    if path.is_empty() {
        return ".".to_string();
    }

    let is_absolute = path.starts_with('/') || path.starts_with('\\');
    let has_trailing_sep = path.ends_with('/') || path.ends_with('\\');

    // Normalize separators
    let normalized = path.replace('\\', "/");
    let parts: Vec<&str> = normalized.split('/').collect();
    let mut stack: Vec<&str> = Vec::new();

    for part in &parts {
        if *part == "." || part.is_empty() {
            continue;
        }
        if *part == ".." {
            if !stack.is_empty() && *stack.last().unwrap() != ".." {
                stack.pop();
            } else if !is_absolute {
                stack.push("..");
            }
        } else {
            stack.push(part);
        }
    }

    let mut result = stack.join("/");
    if is_absolute {
        result = format!("/{}", result);
    }
    if has_trailing_sep && !result.ends_with('/') {
        result.push('/');
    }
    if result.is_empty() {
        return ".".to_string();
    }
    result
}

/// Join multiple path segments.
#[napi]
pub fn join_paths(segments: Vec<String>) -> String {
    let joined = segments.join("/");
    normalize_path(joined)
}

/// Get the directory name (parent) of a path.
#[napi]
pub fn dirname(path: String) -> String {
    if path.is_empty() {
        return ".".to_string();
    }
    let sep_pos = path.rfind('/').or_else(|| path.rfind('\\'));
    match sep_pos {
        Some(0) => "/".to_string(),
        Some(pos) => path[..pos].to_string(),
        None => ".".to_string(),
    }
}

/// Get the file name (last segment) of a path.
#[napi]
pub fn basename(path: String, ext: Option<String>) -> String {
    let name_start = path
        .rfind('/')
        .or_else(|| path.rfind('\\'))
        .map(|p| p + 1)
        .unwrap_or(0);
    let name = &path[name_start..];
    match ext {
        Some(extension) if name.ends_with(&extension) => {
            name[..name.len() - extension.len()].to_string()
        }
        _ => name.to_string(),
    }
}

/// Get the file extension of a path (including the dot).
#[napi]
pub fn extname(path: String) -> String {
    let name = basename(path, None);
    match name.rfind('.') {
        Some(0) | None => String::new(),
        Some(pos) => name[pos..].to_string(),
    }
}

/// Check if a path is absolute.
#[napi]
pub fn is_absolute(path: String) -> bool {
    if path.is_empty() {
        return false;
    }
    let first = path.chars().next().unwrap();
    if first == SLASH || first == BACKSLASH {
        return true;
    }
    // Windows drive letter: C:\
    if path.len() >= 3 {
        let chars: Vec<char> = path.chars().take(3).collect();
        if chars[0].is_ascii_alphabetic() && chars[1] == ':' && (chars[2] == SLASH || chars[2] == BACKSLASH) {
            return true;
        }
    }
    false
}

/// Compute the relative path from `from` to `to`.
#[napi]
pub fn relative_path(from: String, to: String) -> String {
    let from_norm = normalize_path(from).replace('\\', "/");
    let to_norm = normalize_path(to).replace('\\', "/");

    if from_norm == to_norm {
        return String::new();
    }

    let from_parts: Vec<&str> = from_norm.split('/').filter(|s| !s.is_empty()).collect();
    let to_parts: Vec<&str> = to_norm.split('/').filter(|s| !s.is_empty()).collect();

    let common_len = from_parts
        .iter()
        .zip(to_parts.iter())
        .take_while(|(a, b)| a == b)
        .count();

    let ups = from_parts.len() - common_len;
    let mut parts: Vec<&str> = vec![".."; ups];
    parts.extend(&to_parts[common_len..]);

    parts.join("/")
}

/// Resolve paths — like Node.js path.resolve().
#[napi]
pub fn resolve_path(segments: Vec<String>) -> String {
    let mut resolved = String::new();
    for seg in segments.iter().rev() {
        resolved = if resolved.is_empty() {
            seg.clone()
        } else {
            format!("{}/{}", seg, resolved)
        };
        if is_absolute(resolved.clone()) {
            break;
        }
    }
    normalize_path(resolved)
}

// ─── URI utilities ─────────────────────────────────────────────────────────

/// Result of parsing a URI.
#[napi(object)]
pub struct UriComponents {
    pub scheme: String,
    pub authority: String,
    pub path: String,
    pub query: String,
    pub fragment: String,
}

/// Parse a URI string into its components.
#[napi]
pub fn parse_uri(uri_string: String) -> UriComponents {
    let url = url::Url::parse(&uri_string);
    match url {
        Ok(u) => UriComponents {
            scheme: u.scheme().to_string(),
            authority: u.host_str().map(|h| {
                if let Some(port) = u.port() {
                    format!("{}:{}", h, port)
                } else {
                    h.to_string()
                }
            }).unwrap_or_default(),
            path: u.path().to_string(),
            query: u.query().unwrap_or("").to_string(),
            fragment: u.fragment().unwrap_or("").to_string(),
        },
        Err(_) => {
            // Try to parse as a file path
            UriComponents {
                scheme: "file".to_string(),
                authority: String::new(),
                path: uri_string,
                query: String::new(),
                fragment: String::new(),
            }
        }
    }
}

/// Build a URI string from components.
#[napi]
pub fn build_uri(scheme: String, authority: String, path: String, query: Option<String>, fragment: Option<String>) -> String {
    let mut result = format!("{}://", scheme);
    if !authority.is_empty() {
        result.push_str(&authority);
    }
    result.push_str(&path);
    if let Some(q) = query {
        if !q.is_empty() {
            result.push('?');
            result.push_str(&q);
        }
    }
    if let Some(f) = fragment {
        if !f.is_empty() {
            result.push('#');
            result.push_str(&f);
        }
    }
    result
}

/// Convert a file path to a file:// URI.
#[napi]
pub fn file_uri(path: String) -> String {
    let normalized = path.replace('\\', "/");
    if normalized.starts_with('/') {
        format!("file://{}", normalized)
    } else {
        format!("file:///{}", normalized)
    }
}

/// Convert a file:// URI back to a file path.
#[napi]
pub fn uri_to_path(uri: String) -> String {
    let path = uri
        .strip_prefix("file:///")
        .or_else(|| uri.strip_prefix("file://"))
        .unwrap_or(&uri);

    // Decode percent-encoded characters
    percent_decode(path.to_string())
}

/// Percent-encode a string for use in URIs.
#[napi]
pub fn percent_encode(s: String) -> String {
    let mut result = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

/// Percent-decode a URI string.
#[napi]
pub fn percent_decode(s: String) -> String {
    let mut result = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(val) = u8::from_str_radix(
                std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""),
                16,
            ) {
                result.push(val);
                i += 3;
                continue;
            }
        }
        result.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(result).unwrap_or_else(|_| s)
}

// ─── Path matching utilities ───────────────────────────────────────────────

/// Check if a path has a specific extension (case-insensitive).
#[napi]
pub fn has_extension(path: String, ext: String) -> bool {
    let p = path.to_lowercase();
    let e = if ext.starts_with('.') { ext.to_lowercase() } else { format!(".{}", ext.to_lowercase()) };
    p.ends_with(&e)
}

/// Check if a child path is under a parent path.
#[napi]
pub fn is_under(parent: String, child: String) -> bool {
    let p = normalize_path(parent).replace('\\', "/");
    let c = normalize_path(child).replace('\\', "/");
    if p == c {
        return true;
    }
    let prefix = if p.ends_with('/') { p } else { format!("{}/", p) };
    c.starts_with(&prefix)
}

/// Get the depth of a path (number of segments).
#[napi]
pub fn path_depth(path: String) -> u32 {
    let normalized = path.replace('\\', "/");
    normalized
        .split('/')
        .filter(|s| !s.is_empty() && *s != ".")
        .count() as u32
}

/// Check if two paths are equal (platform-aware case sensitivity).
#[napi]
pub fn paths_equal(a: String, b: String) -> bool {
    let na = normalize_path(a).replace('\\', "/");
    let nb = normalize_path(b).replace('\\', "/");
    if cfg!(windows) || cfg!(target_os = "macos") {
        na.to_lowercase() == nb.to_lowercase()
    } else {
        na == nb
    }
}

/// Remove trailing path separator.
#[napi]
pub fn remove_trailing_separator(path: String) -> String {
    let mut p = path;
    while p.len() > 1 && (p.ends_with('/') || p.ends_with('\\')) {
        p.pop();
    }
    p
}

/// Ensure path has trailing separator.
#[napi]
pub fn ensure_trailing_separator(path: String) -> String {
    if path.ends_with('/') || path.ends_with('\\') {
        path
    } else {
        format!("{}/", path)
    }
}

// ─── Windows-specific path utilities ───────────────────────────────────────

/// Check if a path is a Windows UNC path (\\server\share).
#[napi]
pub fn is_unc_path(path: String) -> bool {
    path.starts_with("\\\\") || path.starts_with("//")
}

/// Get the drive letter from a Windows path (e.g., "C" from "C:\foo").
#[napi]
pub fn get_drive_letter(path: String) -> Option<String> {
    let chars: Vec<char> = path.chars().take(3).collect();
    if chars.len() >= 2 && chars[0].is_ascii_alphabetic() && chars[1] == ':' {
        Some(chars[0].to_uppercase().to_string())
    } else {
        None
    }
}

/// Convert a Windows path to POSIX (forward slashes).
#[napi]
pub fn to_posix_path(path: String) -> String {
    path.replace('\\', "/")
}

/// Convert a POSIX path to Windows (backslashes).
#[napi]
pub fn to_windows_path(path: String) -> String {
    path.replace('/', "\\")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        assert_eq!(normalize_path("/a/b/../c".into()), "/a/c");
        assert_eq!(normalize_path("/a/./b/./c".into()), "/a/b/c");
        assert_eq!(normalize_path("a//b".into()), "a/b");
    }

    #[test]
    fn test_join() {
        assert_eq!(join_paths(vec!["a".into(), "b".into(), "c".into()]), "a/b/c");
        assert_eq!(join_paths(vec!["/a".into(), "b".into()]), "/a/b");
    }

    #[test]
    fn test_dirname_basename_extname() {
        assert_eq!(dirname("/a/b/c.txt".into()), "/a/b");
        assert_eq!(basename("/a/b/c.txt".into(), None), "c.txt");
        assert_eq!(extname("/a/b/c.txt".into()), ".txt");
        assert_eq!(basename("/a/b/c.txt".into(), Some(".txt".into())), "c");
    }

    #[test]
    fn test_is_absolute() {
        assert!(is_absolute("/foo".into()));
        assert!(!is_absolute("foo".into()));
        assert!(!is_absolute("".into()));
    }

    #[test]
    fn test_relative() {
        assert_eq!(relative_path("/a/b".into(), "/a/c".into()), "../c");
        assert_eq!(relative_path("/a/b/c".into(), "/a/d".into()), "../../d");
    }

    #[test]
    fn test_file_uri() {
        assert_eq!(file_uri("/home/user/file.txt".into()), "file:///home/user/file.txt");
    }

    #[test]
    fn test_is_under() {
        assert!(is_under("/a/b".into(), "/a/b/c".into()));
        assert!(is_under("/a/b".into(), "/a/b".into()));
        assert!(!is_under("/a/b".into(), "/a/c".into()));
    }

    #[test]
    fn test_has_extension() {
        assert!(has_extension("file.RS".into(), ".rs".into()));
        assert!(has_extension("file.ts".into(), "ts".into()));
    }

    #[test]
    fn test_percent_encode_decode() {
        let encoded = percent_encode("hello world".into());
        assert!(encoded.contains("%20"));
        assert_eq!(percent_decode(encoded), "hello world");
    }
}
