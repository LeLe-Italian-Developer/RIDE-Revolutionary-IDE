/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Platform detection and environment utilities — Rust port of
//! `src/vs/base/common/platform.ts`, `process.ts`, and `navigator.ts`.

use napi_derive::napi;
use napi::bindgen_prelude::*;

/// Get the current OS type.
#[napi]
pub fn get_os() -> String {
    if cfg!(target_os = "windows") { "windows".into() }
    else if cfg!(target_os = "macos") { "macos".into() }
    else if cfg!(target_os = "linux") { "linux".into() }
    else { "unknown".into() }
}

#[napi]
pub fn is_windows() -> bool { cfg!(target_os = "windows") }
#[napi]
pub fn is_macos() -> bool { cfg!(target_os = "macos") }
#[napi]
pub fn is_linux() -> bool { cfg!(target_os = "linux") }
#[napi]
pub fn get_arch() -> String { std::env::consts::ARCH.to_string() }
#[napi]
pub fn is_arm() -> bool { cfg!(target_arch = "aarch64") || cfg!(target_arch = "arm") }
#[napi]
pub fn get_os_family() -> String { std::env::consts::FAMILY.to_string() }

#[napi]
pub fn get_env(name: String) -> Option<String> { std::env::var(&name).ok() }

#[napi]
pub fn get_home_dir() -> Option<String> {
    dirs::home_dir().map(|p| p.to_string_lossy().to_string())
}
#[napi]
pub fn get_temp_dir() -> String { std::env::temp_dir().to_string_lossy().to_string() }
#[napi]
pub fn get_cwd() -> Result<String> {
    std::env::current_dir().map(|p| p.to_string_lossy().to_string())
        .map_err(|e| Error::from_reason(format!("{}", e)))
}
#[napi]
pub fn get_config_dir() -> Option<String> { dirs::config_dir().map(|p| p.to_string_lossy().to_string()) }
#[napi]
pub fn get_data_dir() -> Option<String> { dirs::data_dir().map(|p| p.to_string_lossy().to_string()) }
#[napi]
pub fn get_cache_dir() -> Option<String> { dirs::cache_dir().map(|p| p.to_string_lossy().to_string()) }

#[napi]
pub fn cpu_count() -> u32 { num_cpus::get() as u32 }
#[napi]
pub fn physical_cpu_count() -> u32 { num_cpus::get_physical() as u32 }

#[napi]
pub fn total_memory() -> f64 {
    let mut sys = sysinfo::System::new();
    sys.refresh_memory();
    sys.total_memory() as f64
}
#[napi]
pub fn available_memory() -> f64 {
    let mut sys = sysinfo::System::new();
    sys.refresh_memory();
    sys.available_memory() as f64
}
#[napi]
pub fn os_version() -> String { sysinfo::System::os_version().unwrap_or_else(|| "unknown".into()) }
#[napi]
pub fn hostname() -> String { sysinfo::System::host_name().unwrap_or_else(|| "unknown".into()) }
#[napi]
pub fn uptime() -> f64 { sysinfo::System::uptime() as f64 }
#[napi]
pub fn get_pid() -> u32 { std::process::id() }

#[napi]
pub fn native_path_sep() -> String { std::path::MAIN_SEPARATOR.to_string() }
#[napi]
pub fn path_list_sep() -> String { if cfg!(target_os = "windows") { ";".into() } else { ":".into() } }
#[napi]
pub fn native_eol() -> String { if cfg!(target_os = "windows") { "\r\n".into() } else { "\n".into() } }

#[napi(object)]
pub struct PlatformCapabilities {
    pub os: String,
    pub arch: String,
    pub is_64bit: bool,
    pub cpu_count: u32,
    pub total_memory_mb: u32,
    pub native_path_sep: String,
}

#[napi]
pub fn get_capabilities() -> PlatformCapabilities {
    let mut sys = sysinfo::System::new();
    sys.refresh_memory();
    PlatformCapabilities {
        os: get_os(), arch: get_arch(),
        is_64bit: cfg!(target_pointer_width = "64"),
        cpu_count: cpu_count(),
        total_memory_mb: (sys.total_memory() / (1024 * 1024)) as u32,
        native_path_sep: native_path_sep(),
    }
}

#[napi]
pub fn default_shell() -> String {
    if cfg!(target_os = "windows") { std::env::var("COMSPEC").unwrap_or("cmd.exe".into()) }
    else { std::env::var("SHELL").unwrap_or("/bin/sh".into()) }
}
#[napi]
pub fn get_username() -> String {
    std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or("unknown".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os() { assert!(!get_os().is_empty()); }
    #[test]
    fn test_cpu() { assert!(cpu_count() > 0); }
    #[test]
    fn test_mem() { assert!(total_memory() > 0.0); }
    #[test]
    fn test_home() { assert!(get_home_dir().is_some()); }
}
