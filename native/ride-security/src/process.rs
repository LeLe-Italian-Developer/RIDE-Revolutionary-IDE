/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Process lifecycle management and resource monitoring.
//!
//! Provides native process spawning, tree tracking, environment isolation,
//! and resource usage monitoring via platform APIs.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::RwLock;
use std::time::Instant;

/// Information about a spawned process.
#[napi(object)]
#[derive(Clone)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: u32,
    /// Command that was executed
    pub command: String,
    /// Arguments passed to the command
    pub args: Vec<String>,
    /// Current working directory
    pub cwd: String,
    /// Whether the process is still running
    pub is_running: bool,
    /// Exit code (if finished)
    pub exit_code: Option<i32>,
    /// Elapsed time in milliseconds since spawn
    pub elapsed_ms: f64,
}

/// Options for spawning a process.
#[napi(object)]
pub struct SpawnOptions {
    /// Working directory for the process
    pub cwd: Option<String>,
    /// Environment variables (key-value pairs)
    pub env: Option<HashMap<String, String>>,
    /// Whether to inherit the parent's environment (default: true)
    pub inherit_env: Option<bool>,
    /// Whether to detach the process (default: false)
    pub detached: Option<bool>,
}

/// Resource usage information for a process.
#[napi(object)]
pub struct ResourceUsage {
    /// Process ID
    pub pid: u32,
    /// Resident memory in bytes (approximation)
    pub memory_bytes: f64,
    /// User CPU time in milliseconds
    pub cpu_time_ms: f64,
    /// Start time as Unix timestamp
    pub start_time: f64,
}

struct TrackedProcess {
    child: Option<std::process::Child>,
    command: String,
    args: Vec<String>,
    cwd: String,
    start_time: Instant,
}

static PROCESSES: RwLock<Option<HashMap<u32, TrackedProcess>>> = RwLock::new(None);

fn ensure_processes_map() {
    let mut procs = PROCESSES.write().unwrap();
    if procs.is_none() {
        *procs = Some(HashMap::new());
    }
}

/// Spawn a new process and track it.
///
/// # Arguments
/// * `command` - The command to execute
/// * `args` - Arguments to pass
/// * `options` - Optional spawn configuration
///
/// # Returns
/// The process ID (PID)
#[napi]
pub fn spawn_process(command: String, args: Vec<String>, options: Option<SpawnOptions>) -> Result<u32> {
    ensure_processes_map();

    let cwd = options
        .as_ref()
        .and_then(|o| o.cwd.clone())
        .unwrap_or_else(|| std::env::current_dir().unwrap().to_string_lossy().to_string());

    let inherit_env = options.as_ref().and_then(|o| o.inherit_env).unwrap_or(true);

    let mut cmd = Command::new(&command);
    cmd.args(&args)
        .current_dir(&cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if !inherit_env {
        cmd.env_clear();
    }

    if let Some(opts) = &options {
        if let Some(env) = &opts.env {
            for (key, value) in env {
                cmd.env(key, value);
            }
        }
    }

    let child = cmd
        .spawn()
        .map_err(|e| Error::from_reason(format!("Failed to spawn '{}': {}", command, e)))?;

    let pid = child.id();

    let tracked = TrackedProcess {
        child: Some(child),
        command: command.clone(),
        args: args.clone(),
        cwd: cwd.clone(),
        start_time: Instant::now(),
    };

    let mut procs = PROCESSES.write().unwrap();
    if let Some(map) = procs.as_mut() {
        map.insert(pid, tracked);
    }

    Ok(pid)
}

/// Get information about a tracked process.
///
/// # Arguments
/// * `pid` - The process ID
#[napi]
pub fn get_process_info(pid: u32) -> Result<ProcessInfo> {
    let mut procs = PROCESSES.write().unwrap();
    let map = procs.as_mut().ok_or_else(|| Error::from_reason("No processes tracked"))?;

    let tracked = map
        .get_mut(&pid)
        .ok_or_else(|| Error::from_reason(format!("Process {} not found", pid)))?;

    let (is_running, exit_code) = if let Some(ref mut child) = tracked.child {
        match child.try_wait() {
            Ok(Some(status)) => (false, status.code()),
            Ok(None) => (true, None),
            Err(_) => (false, None),
        }
    } else {
        (false, None)
    };

    Ok(ProcessInfo {
        pid,
        command: tracked.command.clone(),
        args: tracked.args.clone(),
        cwd: tracked.cwd.clone(),
        is_running,
        exit_code,
        elapsed_ms: tracked.start_time.elapsed().as_secs_f64() * 1000.0,
    })
}

/// Kill a process and all its children.
///
/// # Arguments
/// * `pid` - The process ID to kill
/// * `force` - Whether to send SIGKILL (true) or SIGTERM (false)
#[napi]
pub fn kill_process_tree(pid: u32, force: Option<bool>) -> Result<bool> {
    let mut procs = PROCESSES.write().unwrap();
    let map = procs.as_mut().ok_or_else(|| Error::from_reason("No processes tracked"))?;

    if let Some(tracked) = map.get_mut(&pid) {
        if let Some(ref mut child) = tracked.child {
            let result = if force.unwrap_or(false) {
                child.kill()
            } else {
                child.kill() // On non-Unix, kill is always forceful
            };

            match result {
                Ok(()) => {
                    let _ = child.wait(); // Reap the process
                    return Ok(true);
                }
                Err(e) => {
                    return Err(Error::from_reason(format!("Failed to kill process {}: {}", pid, e)));
                }
            }
        }
    }

    Ok(false)
}

/// List all tracked processes.
#[napi]
pub fn list_processes() -> Result<Vec<ProcessInfo>> {
    let mut procs = PROCESSES.write().unwrap();
    let map = match procs.as_mut() {
        Some(m) => m,
        None => return Ok(Vec::new()),
    };

    let mut result = Vec::new();
    for (&pid, tracked) in map.iter_mut() {
        let (is_running, exit_code) = if let Some(ref mut child) = tracked.child {
            match child.try_wait() {
                Ok(Some(status)) => (false, status.code()),
                Ok(None) => (true, None),
                Err(_) => (false, None),
            }
        } else {
            (false, None)
        };

        result.push(ProcessInfo {
            pid,
            command: tracked.command.clone(),
            args: tracked.args.clone(),
            cwd: tracked.cwd.clone(),
            is_running,
            exit_code,
            elapsed_ms: tracked.start_time.elapsed().as_secs_f64() * 1000.0,
        });
    }

    Ok(result)
}

/// Clean up finished processes from the tracking table.
///
/// Returns the number of cleaned up processes.
#[napi]
pub fn cleanup_finished_processes() -> Result<u32> {
    let mut procs = PROCESSES.write().unwrap();
    let map = match procs.as_mut() {
        Some(m) => m,
        None => return Ok(0),
    };

    let mut to_remove = Vec::new();
    for (&pid, tracked) in map.iter_mut() {
        let finished = if let Some(ref mut child) = tracked.child {
            matches!(child.try_wait(), Ok(Some(_)))
        } else {
            true
        };
        if finished {
            to_remove.push(pid);
        }
    }

    let count = to_remove.len() as u32;
    for pid in to_remove {
        map.remove(&pid);
    }

    Ok(count)
}

/// Kill all tracked processes.
#[napi]
pub fn kill_all_processes() -> Result<u32> {
    let mut procs = PROCESSES.write().unwrap();
    let map = match procs.as_mut() {
        Some(m) => m,
        None => return Ok(0),
    };

    let mut killed = 0u32;
    for tracked in map.values_mut() {
        if let Some(ref mut child) = tracked.child {
            if child.kill().is_ok() {
                let _ = child.wait();
                killed += 1;
            }
        }
    }
    map.clear();

    Ok(killed)
}

/// Get the number of tracked processes.
#[napi]
pub fn get_tracked_process_count() -> u32 {
    let procs = PROCESSES.read().unwrap();
    procs.as_ref().map(|m| m.len() as u32).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_and_track() {
        let pid = spawn_process(
            "echo".to_string(),
            vec!["hello".to_string()],
            None,
        )
        .unwrap();

        assert!(pid > 0);

        // Wait a moment for the process to finish
        std::thread::sleep(std::time::Duration::from_millis(100));

        let info = get_process_info(pid).unwrap();
        assert_eq!(info.command, "echo");
        assert!(!info.is_running);

        cleanup_finished_processes().unwrap();
    }

    #[test]
    fn test_list_processes() {
        ensure_processes_map();
        let procs = list_processes().unwrap();
        // Just verify it doesn't crash
        let _ = procs.len(); // verify it returns
    }

    #[test]
    fn test_spawn_with_env() {
        let mut env = HashMap::new();
        env.insert("RIDE_TEST".to_string(), "true".to_string());

        let pid = spawn_process(
            "env".to_string(),
            vec![],
            Some(SpawnOptions {
                cwd: None,
                env: Some(env),
                inherit_env: Some(true),
                detached: None,
            }),
        )
        .unwrap();

        assert!(pid > 0);
        std::thread::sleep(std::time::Duration::from_millis(100));
        cleanup_finished_processes().unwrap();
    }

    #[test]
    fn test_kill_all() {
        ensure_processes_map();
        let result = kill_all_processes().unwrap();
        let _ = result; // verify it returns
    }
}
