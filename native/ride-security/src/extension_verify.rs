/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Extension package verification and permission auditing.

use napi::bindgen_prelude::*;
use napi_derive::napi;

use sha2::{Digest, Sha256};

use std::fs;
use std::io::Read;
use std::path::Path;

/// Extension manifest information.
#[napi(object)]
#[derive(Clone)]
pub struct ExtensionManifest {
    pub name: String,
    pub publisher: String,
    pub version: String,
    pub display_name: String,
    pub description: String,
    pub engine_version: String,
    pub categories: Vec<String>,
    pub activation_events: Vec<String>,
}

/// Permission audit result.
#[napi(object)]
#[derive(Clone)]
pub struct PermissionAudit {
    /// Does the extension access the filesystem?
    pub uses_filesystem: bool,
    /// Does the extension make network requests?
    pub uses_network: bool,
    /// Does the extension spawn processes?
    pub uses_process: bool,
    /// Does the extension use the terminal?
    pub uses_terminal: bool,
    /// Does the extension access debug APIs?
    pub uses_debug: bool,
    /// Does the extension access workspace trust?
    pub uses_workspace_trust: bool,
    /// List of VS Code API namespaces used
    pub api_namespaces: Vec<String>,
    /// Risk level: "low", "medium", "high"
    pub risk_level: String,
    /// Detailed findings
    pub findings: Vec<String>,
}

/// Extension verification result.
#[napi(object)]
pub struct VerificationResult {
    pub is_valid: bool,
    pub manifest: Option<ExtensionManifest>,
    pub audit: Option<PermissionAudit>,
    pub file_count: u32,
    pub total_size: f64,
    pub hash: String,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Verify an extension package (VSIX file or unpacked directory).
#[napi]
pub fn verify_extension(extension_path: String) -> Result<VerificationResult> {
    let path = Path::new(&extension_path);
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if !path.exists() {
        return Err(Error::from_reason(format!("Extension not found: {}", extension_path)));
    }

    // Handle directory-based extensions
    if path.is_dir() {
        return verify_unpacked_extension(path, &mut errors, &mut warnings);
    }

    // Handle VSIX (ZIP) files
    if extension_path.ends_with(".vsix") {
        return verify_vsix_extension(path, &mut errors, &mut warnings);
    }

    Err(Error::from_reason("Extension must be a directory or .vsix file"))
}

fn verify_unpacked_extension(dir: &Path, errors: &mut Vec<String>, warnings: &mut Vec<String>) -> Result<VerificationResult> {
    let manifest_path = dir.join("package.json");
    if !manifest_path.exists() {
        errors.push("Missing package.json manifest".to_string());
        return Ok(VerificationResult {
            is_valid: false, manifest: None, audit: None, file_count: 0,
            total_size: 0.0, hash: String::new(), errors: errors.clone(), warnings: warnings.clone(),
        });
    }

    let manifest_content = fs::read_to_string(&manifest_path)
        .map_err(|e| Error::from_reason(format!("Failed to read manifest: {}", e)))?;

    let manifest = parse_manifest(&manifest_content, errors, warnings);
    let audit = audit_extension_dir(dir, &manifest_content);

    // Count files and compute hash
    let mut file_count = 0u32;
    let mut total_size = 0f64;
    let mut hasher = Sha256::new();

    for entry in walkdir(dir) {
        if entry.is_file() {
            file_count += 1;
            total_size += entry.metadata().map(|m| m.len() as f64).unwrap_or(0.0);
            if let Ok(content) = fs::read(&entry) {
                hasher.update(&content);
            }
        }
    }

    let hash = hex::encode(hasher.finalize());

    // Validation checks
    if file_count == 0 { errors.push("Extension contains no files".to_string()); }
    if total_size > 100_000_000.0 { warnings.push("Extension is larger than 100MB".to_string()); }

    Ok(VerificationResult {
        is_valid: errors.is_empty(),
        manifest: Some(manifest),
        audit: Some(audit),
        file_count,
        total_size,
        hash,
        errors: errors.clone(),
        warnings: warnings.clone(),
    })
}

fn verify_vsix_extension(vsix_path: &Path, errors: &mut Vec<String>, warnings: &mut Vec<String>) -> Result<VerificationResult> {
    let file = fs::File::open(vsix_path)
        .map_err(|e| Error::from_reason(format!("Failed to open VSIX: {}", e)))?;

    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| Error::from_reason(format!("Invalid VSIX archive: {}", e)))?;

    let mut manifest_content = String::new();
    let mut found_manifest = false;
    let mut file_count = 0u32;
    let mut total_size = 0f64;
    let mut hasher = Sha256::new();

    for i in 0..archive.len() {
        if let Ok(mut entry) = archive.by_index(i) {
            file_count += 1;
            total_size += entry.size() as f64;

            let name = entry.name().to_string();
            if name.ends_with("package.json") && (name.contains("extension/") || name == "package.json") {
                let mut content = String::new();
                let _ = entry.read_to_string(&mut content);
                manifest_content = content;
                found_manifest = true;
            }

            let mut buf = Vec::new();
            let _ = entry.read_to_end(&mut buf);
            hasher.update(&buf);
        }
    }

    if !found_manifest {
        errors.push("VSIX missing package.json manifest".to_string());
    }

    let manifest = parse_manifest(&manifest_content, errors, warnings);
    let audit = audit_source_code(&manifest_content);

    let hash = hex::encode(hasher.finalize());

    Ok(VerificationResult {
        is_valid: errors.is_empty(),
        manifest: Some(manifest),
        audit: Some(audit),
        file_count,
        total_size,
        hash,
        errors: errors.clone(),
        warnings: warnings.clone(),
    })
}

fn walkdir(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(walkdir(&path));
            } else {
                files.push(path);
            }
        }
    }
    files
}

fn parse_manifest(content: &str, errors: &mut Vec<String>, warnings: &mut Vec<String>) -> ExtensionManifest {
    let json: serde_json::Value = serde_json::from_str(content).unwrap_or_default();

    let name = json.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let publisher = json.get("publisher").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let version = json.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string();

    if name.is_empty() { errors.push("Missing 'name' in manifest".to_string()); }
    if publisher.is_empty() { warnings.push("Missing 'publisher' in manifest".to_string()); }
    if version.is_empty() { errors.push("Missing 'version' in manifest".to_string()); }

    let engine = json.get("engines").and_then(|e| e.get("vscode")).and_then(|v| v.as_str()).unwrap_or("*").to_string();
    let categories = json.get("categories").and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let activation = json.get("activationEvents").and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    ExtensionManifest {
        name, publisher, version,
        display_name: json.get("displayName").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        description: json.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        engine_version: engine,
        categories, activation_events: activation,
    }
}

fn audit_extension_dir(dir: &Path, manifest: &str) -> PermissionAudit {
    let mut all_source = manifest.to_string();
    for file in walkdir(dir) {
        if let Some(ext) = file.extension() {
            let ext_str = ext.to_string_lossy();
            if matches!(ext_str.as_ref(), "js" | "ts" | "mjs" | "cjs") {
                if let Ok(content) = fs::read_to_string(&file) {
                    all_source.push_str(&content);
                }
            }
        }
    }
    audit_source_code(&all_source)
}

fn audit_source_code(source: &str) -> PermissionAudit {
    let uses_fs = source.contains("vscode.workspace.fs") || source.contains("fs.readFile") || source.contains("fs.writeFile") || source.contains("require('fs')");
    let uses_net = source.contains("http.request") || source.contains("https.request") || source.contains("fetch(") || source.contains("XMLHttpRequest") || source.contains("require('http')");
    let uses_proc = source.contains("child_process") || source.contains("spawn(") || source.contains("exec(") || source.contains("execFile(");
    let uses_term = source.contains("vscode.window.createTerminal") || source.contains("Terminal");
    let uses_debug = source.contains("vscode.debug");
    let uses_trust = source.contains("workspaceTrust") || source.contains("isTrusted");

    let mut namespaces = Vec::new();
    for ns in &["workspace", "window", "commands", "debug", "extensions", "env", "languages", "tasks", "scm", "notebooks", "tests", "chat", "lm"] {
        if source.contains(&format!("vscode.{}", ns)) { namespaces.push(ns.to_string()); }
    }

    let mut findings = Vec::new();
    if uses_fs { findings.push("Accesses the filesystem".to_string()); }
    if uses_net { findings.push("Makes network requests".to_string()); }
    if uses_proc { findings.push("Spawns child processes".to_string()); }
    if uses_term { findings.push("Creates terminal instances".to_string()); }
    if uses_debug { findings.push("Uses debug APIs".to_string()); }

    let risk_score = uses_fs as u32 + uses_net as u32 * 2 + uses_proc as u32 * 3 + uses_term as u32 + uses_debug as u32;
    let risk_level = match risk_score { 0..=1 => "low", 2..=3 => "medium", _ => "high" };

    PermissionAudit {
        uses_filesystem: uses_fs, uses_network: uses_net, uses_process: uses_proc,
        uses_terminal: uses_term, uses_debug: uses_debug, uses_workspace_trust: uses_trust,
        api_namespaces: namespaces, risk_level: risk_level.to_string(), findings,
    }
}

/// Audit permissions of an installed extension.
#[napi]
pub fn audit_permissions(extension_path: String) -> Result<PermissionAudit> {
    let path = Path::new(&extension_path);
    if !path.exists() { return Err(Error::from_reason("Extension not found")); }
    let manifest = fs::read_to_string(path.join("package.json")).unwrap_or_default();
    Ok(audit_extension_dir(path, &manifest))
}

/// Check integrity of an installed extension against a known hash.
#[napi]
pub fn check_extension_integrity(extension_path: String, expected_hash: String) -> Result<bool> {
    let path = Path::new(&extension_path);
    if !path.exists() { return Err(Error::from_reason("Extension not found")); }

    let mut hasher = Sha256::new();
    for file in walkdir(path) {
        if file.is_file() {
            if let Ok(content) = fs::read(&file) { hasher.update(&content); }
        }
    }
    Ok(hex::encode(hasher.finalize()) == expected_hash.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_test_extension() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("ride_test_ext");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let manifest = r#"{"name":"test-ext","publisher":"tester","version":"1.0.0","displayName":"Test","engines":{"vscode":"^1.80.0"},"activationEvents":["onCommand:test"],"categories":["Other"]}"#;
        fs::File::create(dir.join("package.json")).unwrap().write_all(manifest.as_bytes()).unwrap();
        fs::File::create(dir.join("extension.js")).unwrap().write_all(b"const vscode = require('vscode');\nvscode.window.showInformationMessage('Hello');").unwrap();
        dir
    }

    #[test]
    fn test_verify_extension() {
        let dir = create_test_extension();
        let result = verify_extension(dir.to_str().unwrap().to_string()).unwrap();
        assert!(result.is_valid);
        assert!(result.manifest.is_some());
        assert!(result.file_count >= 2);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_audit_permissions() {
        let dir = create_test_extension();
        let audit = audit_permissions(dir.to_str().unwrap().to_string()).unwrap();
        assert!(audit.api_namespaces.contains(&"window".to_string()));
        assert_eq!(audit.risk_level, "low");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_integrity_check() {
        let dir = create_test_extension();
        let result = verify_extension(dir.to_str().unwrap().to_string()).unwrap();
        let valid = check_extension_integrity(dir.to_str().unwrap().to_string(), result.hash.clone()).unwrap();
        assert!(valid);
        let invalid = check_extension_integrity(dir.to_str().unwrap().to_string(), "badhash".to_string()).unwrap();
        assert!(!invalid);
        fs::remove_dir_all(&dir).ok();
    }
}
