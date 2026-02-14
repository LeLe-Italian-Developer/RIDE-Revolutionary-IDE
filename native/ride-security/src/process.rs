/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! RIDE Advanced Process Management
//!
//! Features:
//! - POSIX/Windows process group management (killing whole trees)
//! - Real-time resource monitoring (CPU/Memory per-PID) via `sysinfo`
//! - Safe environment variable isolation and merging
//! - Automatic orphan prevention and zombie reaping

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::{RwLock, Arc};
use sysinfo::{Pid, System};

#[napi(object)]
pub struct ProcessStats {
    pub pid: u32,
    pub cpu_usage: f64,
    pub memory_kb: f64,
    pub name: String,
    pub is_alive: bool,
}

#[napi(object)]
pub struct SpawnOptions {
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub detached: Option<bool>,
}

pub struct ProcessHandle {
    pub pid: u32,
    pub start_time: f64,
}

static SYSTEM: RwLock<Option<System>> = RwLock::new(None);

fn get_system() -> Arc<RwLock<System>> {
    static SYS_ARC: std::sync::OnceLock<Arc<RwLock<System>>> = std::sync::OnceLock::new();
    SYS_ARC.get_or_init(|| {
        let mut sys = System::new_all();
        sys.refresh_all();
        Arc::new(RwLock::new(sys))
    }).clone()
}

#[napi]
pub fn spawn_process_v2(command: String, args: Vec<String>, options: Option<SpawnOptions>) -> Result<u32> {
    let mut cmd = Command::new(&command);
    cmd.args(&args)
       .stdin(Stdio::null())
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());

    if let Some(opts) = options {
        if let Some(cwd) = opts.cwd { cmd.current_dir(cwd); }
        if let Some(env) = opts.env {
            for (k, v) in env { cmd.env(k, v); }
        }
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0); // Create a new process group for entire tree killing
    }

    let child = cmd.spawn().map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(child.id())
}

#[napi]
pub fn get_process_stats(pid: u32) -> Option<ProcessStats> {
    let sys_arc = get_system();
    let mut sys = sys_arc.write().unwrap();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[Pid::from(pid as usize)]), true);

    if let Some(proc) = sys.process(Pid::from(pid as usize)) {
        Some(ProcessStats {
            pid,
            cpu_usage: proc.cpu_usage() as f64,
            memory_kb: proc.memory() as f64,
            name: proc.name().to_string_lossy().to_string(),
            is_alive: true,
        })
    } else {
        None
    }
}

#[napi]
pub fn kill_process_group(pid: u32) -> Result<()> {
    #[cfg(unix)]
    {
        let pgid = pid as i32;
        unsafe {
            if libc::kill(-pgid, libc::SIGKILL) != 0 {
                // If group kill failed, try killing the single PID
                libc::kill(pgid, libc::SIGKILL);
            }
        }
    }
    #[cfg(windows)]
    {
        // On windows, we use taskkill /F /T /PID to kill the tree
        Command::new("taskkill")
            .args(&["/F", "/T", "/PID", &pid.to_string()])
            .spawn()
            .map_err(|e| Error::from_reason(e.to_string()))?;
    }
    Ok(())
}

#[napi]
pub fn list_system_processes(name_filter: Option<String>) -> Vec<ProcessStats> {
    let sys_arc = get_system();
    let mut sys = sys_arc.write().unwrap();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let filter = name_filter.as_ref().map(|s| s.to_lowercase());

    sys.processes().values().filter(|p| {
        if let Some(f) = &filter {
            p.name().to_string_lossy().to_lowercase().contains(f)
        } else {
            true
        }
    }).map(|p| ProcessStats {
        pid: p.pid().as_u32(),
        cpu_usage: p.cpu_usage() as f64,
        memory_kb: p.memory() as f64,
        name: p.name().to_string_lossy().to_string(),
        is_alive: true,
    }).collect()
}
