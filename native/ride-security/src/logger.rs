/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Structured logging with ring buffer, rotation, and filtering.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

#[napi(object)]
#[derive(Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: f64,
    pub level: String,
    pub message: String,
    pub source: String,
    pub data: Option<String>,
}

#[napi(object)]
pub struct LoggerConfig {
    pub max_buffer_size: Option<u32>,
    pub min_level: Option<u32>,
    pub file_path: Option<String>,
    pub max_file_size: Option<u32>,
    pub max_rotated_files: Option<u32>,
}

struct LoggerState {
    buffer: VecDeque<LogEntry>,
    max_buffer_size: usize,
    min_level: u32,
    file_path: Option<PathBuf>,
    max_file_size: u64,
    max_rotated_files: u32,
    total_logged: u64,
}

static LOGGER: RwLock<Option<LoggerState>> = RwLock::new(None);

fn level_str(level: u32) -> &'static str {
    match level { 0 => "TRACE", 1 => "DEBUG", 2 => "INFO", 3 => "WARN", 4 => "ERROR", 5 => "FATAL", _ => "UNKNOWN" }
}

fn now_ms() -> f64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs_f64() * 1000.0
}

fn rotate_log_files(base: &Path, max: u32) {
    let _ = fs::remove_file(format!("{}.{}", base.display(), max));
    for i in (1..max).rev() {
        let _ = fs::rename(format!("{}.{}", base.display(), i), format!("{}.{}", base.display(), i + 1));
    }
    let _ = fs::rename(base, format!("{}.1", base.display()));
}

#[napi]
pub fn init_logger(config: Option<LoggerConfig>) -> Result<()> {
    let max_buf = config.as_ref().and_then(|c| c.max_buffer_size).unwrap_or(5000) as usize;
    let min_lvl = config.as_ref().and_then(|c| c.min_level).unwrap_or(2);
    let max_fs = config.as_ref().and_then(|c| c.max_file_size).unwrap_or(10_000_000) as u64;
    let max_rot = config.as_ref().and_then(|c| c.max_rotated_files).unwrap_or(5);
    let fp = config.as_ref().and_then(|c| c.file_path.as_ref()).map(PathBuf::from);

    if let Some(ref p) = fp {
        if let Some(parent) = p.parent() { let _ = fs::create_dir_all(parent); }
    }

    *LOGGER.write().unwrap() = Some(LoggerState {
        buffer: VecDeque::with_capacity(max_buf), max_buffer_size: max_buf,
        min_level: min_lvl, file_path: fp, max_file_size: max_fs,
        max_rotated_files: max_rot, total_logged: 0,
    });
    Ok(())
}

#[napi]
pub fn log_message(level: u32, message: String, source: String, data: Option<String>) -> Result<()> {
    let mut logger = LOGGER.write().unwrap();
    let state = match logger.as_mut() {
        Some(s) => s,
        None => { drop(logger); init_logger(None)?; return log_message(level, message, source, data); }
    };

    if level < state.min_level { return Ok(()); }

    let entry = LogEntry { timestamp: now_ms(), level: level_str(level).to_string(), message, source, data };

    if state.buffer.len() >= state.max_buffer_size { state.buffer.pop_front(); }
    state.buffer.push_back(entry.clone());
    state.total_logged += 1;

    if let Some(ref fp) = state.file_path {
        if let Ok(meta) = fs::metadata(fp) {
            if meta.len() >= state.max_file_size { rotate_log_files(fp, state.max_rotated_files); }
        }
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(fp) {
            let _ = writeln!(f, "{}", serde_json::to_string(&entry).unwrap_or_default());
        }
    }
    Ok(())
}

#[napi]
pub fn get_recent_logs(count: Option<u32>, min_level: Option<u32>, source_filter: Option<String>) -> Vec<LogEntry> {
    let logger = LOGGER.read().unwrap();
    let state = match logger.as_ref() { Some(s) => s, None => return Vec::new() };
    let max = count.unwrap_or(100) as usize;
    let lvl = min_level.unwrap_or(0);

    state.buffer.iter().rev().filter(|e| {
        let n = match e.level.as_str() { "TRACE"=>0,"DEBUG"=>1,"INFO"=>2,"WARN"=>3,"ERROR"=>4,"FATAL"=>5,_=>0 };
        if n < lvl { return false; }
        if let Some(ref s) = source_filter { if !e.source.contains(s.as_str()) { return false; } }
        true
    }).take(max).cloned().collect()
}

#[napi]
pub fn get_total_logged() -> f64 {
    LOGGER.read().unwrap().as_ref().map(|s| s.total_logged as f64).unwrap_or(0.0)
}

#[napi]
pub fn clear_log_buffer() {
    if let Some(s) = LOGGER.write().unwrap().as_mut() { s.buffer.clear(); }
}

#[napi]
pub fn rotate_logs() -> Result<()> {
    let logger = LOGGER.read().unwrap();
    if let Some(s) = logger.as_ref() {
        if let Some(ref fp) = s.file_path { rotate_log_files(fp, s.max_rotated_files); }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_and_log() {
        init_logger(Some(LoggerConfig { max_buffer_size: Some(100), min_level: Some(0), file_path: None, max_file_size: None, max_rotated_files: None })).unwrap();
        log_message(2, "Test".to_string(), "test".to_string(), None).unwrap();
        log_message(4, "Error".to_string(), "test".to_string(), None).unwrap();
        let logs = get_recent_logs(Some(10), None, None);
        assert!(logs.len() >= 2);
    }

    #[test]
    fn test_level_filtering() {
        init_logger(Some(LoggerConfig { max_buffer_size: Some(100), min_level: Some(3), file_path: None, max_file_size: None, max_rotated_files: None })).unwrap();
        log_message(1, "Debug".to_string(), "test".to_string(), None).unwrap();
        log_message(3, "Warn".to_string(), "test".to_string(), None).unwrap();
        let logs = get_recent_logs(Some(10), None, None);
        assert!(logs.iter().all(|l| l.level != "DEBUG"));
    }

    #[test]
    fn test_ring_buffer_overflow() {
        init_logger(Some(LoggerConfig { max_buffer_size: Some(5), min_level: Some(0), file_path: None, max_file_size: None, max_rotated_files: None })).unwrap();
        for i in 0..10 { log_message(2, format!("Msg {}", i), "test".to_string(), None).unwrap(); }
        let logs = get_recent_logs(Some(100), None, None);
        assert!(logs.len() <= 5);
    }

    #[test]
    fn test_file_logging() {
        let tmp = std::env::temp_dir().join("ride_test_log.jsonl");
        let _ = fs::remove_file(&tmp);
        init_logger(Some(LoggerConfig { max_buffer_size: Some(100), min_level: Some(0), file_path: Some(tmp.to_str().unwrap().to_string()), max_file_size: Some(1_000_000), max_rotated_files: Some(3) })).unwrap();
        log_message(2, "File test".to_string(), "test".to_string(), None).unwrap();
        assert!(tmp.exists());
        let _ = fs::remove_file(&tmp);
    }
}
