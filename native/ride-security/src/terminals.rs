/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Terminal and shell detection — Rust port of `src/vs/base/node/shell.ts`,
//! `terminals.ts`, and `powershell.ts`.
//! Detects system default shell, available terminals, and terminal encoding.

use napi_derive::napi;
use napi::bindgen_prelude::*;
use std::path::Path;

#[napi(object)]
pub struct ShellInfo {
    pub path: String,
    pub name: String,
    pub is_default: bool,
}

/// Get the system's default shell.
#[napi]
pub fn get_system_shell() -> String {
    #[cfg(target_os = "windows")]
    {
        std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("SHELL").unwrap_or_else(|_| {
            // Try to read from /etc/passwd via libc
            #[cfg(unix)]
            {
                unsafe {
                    let uid = libc::getuid();
                    let pw = libc::getpwuid(uid);
                    if !pw.is_null() {
                        let shell = std::ffi::CStr::from_ptr((*pw).pw_shell);
                        if let Ok(s) = shell.to_str() {
                            if !s.is_empty() && s != "/bin/false" {
                                return s.to_string();
                            }
                        }
                    }
                }
            }
            "/bin/bash".to_string()
        })
    }
}

/// Get the Windows shell (cmd.exe or comspec).
#[napi]
pub fn get_windows_shell() -> String {
    std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
}

/// Detect all available shells on the system.
#[napi]
pub fn detect_available_shells() -> Vec<ShellInfo> {
    let mut shells = Vec::new();
    let default_shell = get_system_shell();

    #[cfg(not(target_os = "windows"))]
    {
        let known_shells = [
            ("/bin/bash", "Bash"),
            ("/bin/zsh", "Zsh"),
            ("/bin/sh", "Sh"),
            ("/bin/fish", "Fish"),
            ("/usr/bin/fish", "Fish"),
            ("/bin/csh", "C Shell"),
            ("/bin/tcsh", "TC Shell"),
            ("/bin/ksh", "Korn Shell"),
            ("/usr/local/bin/bash", "Bash (Homebrew)"),
            ("/usr/local/bin/zsh", "Zsh (Homebrew)"),
            ("/usr/local/bin/fish", "Fish (Homebrew)"),
            ("/opt/homebrew/bin/bash", "Bash (Homebrew ARM)"),
            ("/opt/homebrew/bin/zsh", "Zsh (Homebrew ARM)"),
            ("/opt/homebrew/bin/fish", "Fish (Homebrew ARM)"),
        ];

        for &(path, name) in &known_shells {
            if Path::new(path).exists() {
                shells.push(ShellInfo {
                    path: path.to_string(),
                    name: name.to_string(),
                    is_default: path == default_shell,
                });
            }
        }

        // Also check /etc/shells
        if let Ok(content) = std::fs::read_to_string("/etc/shells") {
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with('/') && Path::new(line).exists() {
                    if !shells.iter().any(|s| s.path == line) {
                        let name = Path::new(line).file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        shells.push(ShellInfo {
                            path: line.to_string(),
                            name,
                            is_default: line == default_shell,
                        });
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".into());
        let known_shells = [
            (format!("{}\\System32\\cmd.exe", windir), "Command Prompt"),
            (format!("{}\\System32\\WindowsPowerShell\\v1.0\\powershell.exe", windir), "Windows PowerShell"),
        ];

        for (path, name) in &known_shells {
            if Path::new(path).exists() {
                shells.push(ShellInfo {
                    path: path.clone(),
                    name: name.to_string(),
                    is_default: *path == default_shell,
                });
            }
        }

        // Check for PowerShell 7+
        let pwsh_paths = [
            "C:\\Program Files\\PowerShell\\7\\pwsh.exe",
            "C:\\Program Files (x86)\\PowerShell\\7\\pwsh.exe",
        ];
        for path in &pwsh_paths {
            if Path::new(path).exists() {
                shells.push(ShellInfo {
                    path: path.to_string(),
                    name: "PowerShell 7".to_string(),
                    is_default: false,
                });
            }
        }

        // Check for Git Bash
        let git_bash_paths = [
            "C:\\Program Files\\Git\\bin\\bash.exe",
            "C:\\Program Files (x86)\\Git\\bin\\bash.exe",
        ];
        for path in &git_bash_paths {
            if Path::new(path).exists() {
                shells.push(ShellInfo {
                    path: path.to_string(),
                    name: "Git Bash".to_string(),
                    is_default: false,
                });
            }
        }
    }

    shells
}

/// Detect terminal encoding.
#[napi]
pub fn detect_terminal_encoding() -> String {
    // Check LANG environment variable
    if let Ok(lang) = std::env::var("LANG") {
        if lang.contains("UTF-8") || lang.contains("utf-8") || lang.contains("utf8") {
            return "utf-8".to_string();
        }
    }

    // Check LC_ALL or LC_CTYPE
    for var in &["LC_ALL", "LC_CTYPE"] {
        if let Ok(val) = std::env::var(var) {
            if val.contains("UTF-8") || val.contains("utf-8") {
                return "utf-8".to_string();
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Default to code page
        return "cp1252".to_string();
    }

    #[cfg(not(target_os = "windows"))]
    {
        "utf-8".to_string()
    }
}

/// Check if a shell path exists and is executable.
#[napi]
pub fn is_shell_valid(shell_path: String) -> bool {
    let p = Path::new(&shell_path);
    if !p.exists() { return false; }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(p) {
            return meta.permissions().mode() & 0o111 != 0;
        }
    }

    #[cfg(not(unix))]
    {
        return p.is_file();
    }

    false
}

/// Get the shell name from a path.
#[napi]
pub fn shell_name_from_path(shell_path: String) -> String {
    Path::new(&shell_path).file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Build shell args for a command execution.
#[napi]
pub fn get_shell_exec_args(shell_path: String) -> Vec<String> {
    let name = shell_name_from_path(shell_path.clone()).to_lowercase();
    match name.as_str() {
        "cmd" => vec!["/C".to_string()],
        "powershell" | "pwsh" => vec!["-NoProfile".to_string(), "-Command".to_string()],
        "bash" | "zsh" | "sh" | "fish" | "ksh" | "csh" | "tcsh" => vec!["-c".to_string()],
        _ => vec!["-c".to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_system_shell() {
        let shell = get_system_shell();
        assert!(!shell.is_empty());
    }

    #[test]
    fn test_detect_shells() {
        let shells = detect_available_shells();
        // Should find at least one shell on any system
        assert!(!shells.is_empty());
    }

    #[test]
    fn test_shell_name() {
        assert_eq!(shell_name_from_path("/bin/bash".into()), "bash");
        assert_eq!(shell_name_from_path("/bin/zsh".into()), "zsh");
    }

    #[test]
    fn test_shell_exec_args() {
        let args = get_shell_exec_args("/bin/bash".into());
        assert_eq!(args, vec!["-c"]);
    }

    #[test]
    fn test_encoding() {
        let enc = detect_terminal_encoding();
        assert!(!enc.is_empty());
    }
}
