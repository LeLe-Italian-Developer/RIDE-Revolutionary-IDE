/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Cross-platform file system watcher with debouncing and gitignore-aware filtering.
//!
//! Provides reliable recursive directory watching using the `notify` crate,
//! which uses platform-native APIs (FSEvents on macOS, inotify on Linux,
//! ReadDirectoryChangesW on Windows).

use napi::bindgen_prelude::*;
use napi_derive::napi;
use notify::{
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

/// Represents a file system change event.
#[napi(object)]
#[derive(Clone)]
pub struct FsEvent {
    /// The type of change: "create", "modify", "remove", "rename"
    pub event_type: String,
    /// The absolute path of the affected file or directory
    pub path: String,
    /// Whether the path is a directory
    pub is_directory: bool,
    /// Timestamp in milliseconds since watcher started
    pub timestamp_ms: f64,
}

/// Configuration for the file system watcher.
#[napi(object)]
pub struct WatcherConfig {
    /// Debounce duration in milliseconds (default: 100)
    pub debounce_ms: Option<u32>,
    /// Glob patterns to ignore (e.g., ["node_modules/**", ".git/**"])
    pub ignore_patterns: Option<Vec<String>>,
    /// Whether to watch recursively (default: true)
    pub recursive: Option<bool>,
    /// Maximum number of events to buffer (default: 10000)
    pub max_buffer_size: Option<u32>,
}

/// Internal state for a watched directory.
struct WatchHandle {
    _watcher: RecommendedWatcher,
    receiver: Receiver<notify::Result<Event>>,
    start_time: Instant,
    ignore_patterns: Vec<glob::Pattern>,
    event_buffer: Vec<FsEvent>,
    max_buffer_size: usize,
    last_event_times: HashMap<String, Instant>,
    debounce_duration: Duration,
}

static WATCHERS: RwLock<Option<HashMap<String, Arc<Mutex<WatchHandle>>>>> = RwLock::new(None);

fn ensure_watchers_map() {
    let mut w = WATCHERS.write().unwrap();
    if w.is_none() {
        *w = Some(HashMap::new());
    }
}

fn event_kind_to_string(kind: &EventKind) -> &'static str {
    match kind {
        EventKind::Create(_) => "create",
        EventKind::Modify(_) => "modify",
        EventKind::Remove(_) => "remove",
        EventKind::Any => "modify",
        EventKind::Access(_) => "access",
        EventKind::Other => "other",
    }
}

fn should_ignore(path: &Path, patterns: &[glob::Pattern]) -> bool {
    let path_str = path.to_string_lossy();
    for pattern in patterns {
        if pattern.matches(&path_str) {
            return true;
        }
        // Also check just the filename
        if let Some(name) = path.file_name() {
            if pattern.matches(&name.to_string_lossy()) {
                return true;
            }
        }
    }
    false
}

/// Start watching a directory for file system changes.
///
/// # Arguments
/// * `watch_id` - Unique identifier for this watch (used to retrieve events later)
/// * `directory` - Absolute path to the directory to watch
/// * `config` - Optional configuration for debouncing, ignoring, etc.
#[napi]
pub fn watch_directory(watch_id: String, directory: String, config: Option<WatcherConfig>) -> Result<()> {
    ensure_watchers_map();

    let dir_path = PathBuf::from(&directory);
    if !dir_path.exists() {
        return Err(Error::from_reason(format!("Directory not found: {}", directory)));
    }
    if !dir_path.is_dir() {
        return Err(Error::from_reason(format!("Not a directory: {}", directory)));
    }

    let debounce_ms = config.as_ref().and_then(|c| c.debounce_ms).unwrap_or(100);
    let recursive = config.as_ref().and_then(|c| c.recursive).unwrap_or(true);
    let max_buffer = config.as_ref().and_then(|c| c.max_buffer_size).unwrap_or(10000) as usize;

    let ignore_patterns: Vec<glob::Pattern> = config
        .as_ref()
        .and_then(|c| c.ignore_patterns.as_ref())
        .map(|patterns| {
            patterns
                .iter()
                .filter_map(|p| glob::Pattern::new(p).ok())
                .collect()
        })
        .unwrap_or_default();

    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(
        move |res: notify::Result<Event>| {
            let _ = tx.send(res);
        },
        Config::default().with_poll_interval(Duration::from_millis(debounce_ms as u64)),
    )
    .map_err(|e| Error::from_reason(format!("Failed to create watcher: {}", e)))?;

    let mode = if recursive {
        RecursiveMode::Recursive
    } else {
        RecursiveMode::NonRecursive
    };

    watcher
        .watch(&dir_path, mode)
        .map_err(|e| Error::from_reason(format!("Failed to watch {}: {}", directory, e)))?;

    let handle = WatchHandle {
        _watcher: watcher,
        receiver: rx,
        start_time: Instant::now(),
        ignore_patterns,
        event_buffer: Vec::with_capacity(256),
        max_buffer_size: max_buffer,
        last_event_times: HashMap::new(),
        debounce_duration: Duration::from_millis(debounce_ms as u64),
    };

    let mut watchers = WATCHERS.write().unwrap();
    if let Some(map) = watchers.as_mut() {
        map.insert(watch_id, Arc::new(Mutex::new(handle)));
    }

    Ok(())
}

/// Stop watching a directory.
///
/// # Arguments
/// * `watch_id` - The ID used when calling `watchDirectory`
#[napi]
pub fn unwatch_directory(watch_id: String) -> Result<()> {
    let mut watchers = WATCHERS.write().unwrap();
    if let Some(map) = watchers.as_mut() {
        if map.remove(&watch_id).is_none() {
            return Err(Error::from_reason(format!("No watcher found with ID: {}", watch_id)));
        }
    }
    Ok(())
}

/// Get pending file system events for a watcher.
///
/// Returns all events that have occurred since the last call to `getWatchEvents`.
/// Events are debounced — rapid changes to the same file produce a single event.
///
/// # Arguments
/// * `watch_id` - The ID used when calling `watchDirectory`
///
/// # Returns
/// Array of `FsEvent` objects
#[napi]
pub fn get_watch_events(watch_id: String) -> Result<Vec<FsEvent>> {
    let watchers = WATCHERS.read().unwrap();
    let map = watchers.as_ref().ok_or_else(|| Error::from_reason("No watchers initialized"))?;
    let handle_arc = map
        .get(&watch_id)
        .ok_or_else(|| Error::from_reason(format!("No watcher found with ID: {}", watch_id)))?
        .clone();
    drop(watchers); // Release read lock

    let mut handle = handle_arc.lock().unwrap();

    // Drain the channel
    loop {
        match handle.receiver.try_recv() {
            Ok(Ok(event)) => {
                let event_type = event_kind_to_string(&event.kind);
                if event_type == "access" || event_type == "other" {
                    continue;
                }

                for path in &event.paths {
                    if should_ignore(path, &handle.ignore_patterns) {
                        continue;
                    }

                    let path_str = path.to_string_lossy().to_string();
                    let now = Instant::now();

                    // Debounce: skip if we saw the same path very recently
                    if let Some(last_time) = handle.last_event_times.get(&path_str) {
                        if now.duration_since(*last_time) < handle.debounce_duration {
                            continue;
                        }
                    }
                    handle.last_event_times.insert(path_str.clone(), now);

                    if handle.event_buffer.len() < handle.max_buffer_size {
                        let ts = handle.start_time.elapsed().as_millis() as f64;
                        handle.event_buffer.push(FsEvent {
                            event_type: event_type.to_string(),
                            path: path_str,
                            is_directory: path.is_dir(),
                            timestamp_ms: ts,
                        });
                    }
                }
            }
            Ok(Err(_)) => continue,
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => break,
        }
    }

    let events = std::mem::take(&mut handle.event_buffer);
    handle.last_event_times.clear();
    Ok(events)
}

/// Get the number of active watchers.
#[napi]
pub fn get_watcher_count() -> u32 {
    let watchers = WATCHERS.read().unwrap();
    watchers.as_ref().map(|m| m.len() as u32).unwrap_or(0)
}

/// Stop all active watchers.
#[napi]
pub fn unwatch_all() -> Result<()> {
    let mut watchers = WATCHERS.write().unwrap();
    if let Some(map) = watchers.as_mut() {
        map.clear();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_watch_and_unwatch() {
        let dir = std::env::temp_dir().join("ride_test_watcher");
        fs::create_dir_all(&dir).unwrap();

        watch_directory(
            "test1".to_string(),
            dir.to_str().unwrap().to_string(),
            None,
        )
        .unwrap();

        assert_eq!(get_watcher_count(), 1);

        unwatch_directory("test1".to_string()).unwrap();
        assert_eq!(get_watcher_count(), 0);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_ignore_patterns() {
        let patterns = vec![
            glob::Pattern::new("*.tmp").unwrap(),
            glob::Pattern::new("node_modules").unwrap(),
        ];

        assert!(should_ignore(Path::new("/foo/bar.tmp"), &patterns));
        assert!(should_ignore(Path::new("/project/node_modules"), &patterns));
        assert!(!should_ignore(Path::new("/foo/bar.rs"), &patterns));
    }

    #[test]
    fn test_event_kind_mapping() {
        assert_eq!(event_kind_to_string(&EventKind::Create(notify::event::CreateKind::Any)), "create");
        assert_eq!(event_kind_to_string(&EventKind::Remove(notify::event::RemoveKind::Any)), "remove");
    }

    #[test]
    fn test_watch_nonexistent_dir() {
        let result = watch_directory(
            "bad".to_string(),
            "/nonexistent/path/xyz".to_string(),
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_unwatch_all() {
        let dir = std::env::temp_dir().join("ride_test_watcher_all");
        fs::create_dir_all(&dir).unwrap();

        watch_directory("a".to_string(), dir.to_str().unwrap().to_string(), None).unwrap();
        watch_directory("b".to_string(), dir.to_str().unwrap().to_string(), None).unwrap();

        assert!(get_watcher_count() >= 2);
        unwatch_all().unwrap();
        assert_eq!(get_watcher_count(), 0);

        fs::remove_dir_all(&dir).ok();
    }
}
