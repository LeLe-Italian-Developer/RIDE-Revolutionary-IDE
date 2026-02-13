/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Process listing and tree building — Rust port of `src/vs/base/node/ps.ts`.
//! Lists running processes and builds a process tree from a root PID.

use napi_derive::napi;
use napi::bindgen_prelude::*;
use std::collections::HashMap;

#[napi(object)]
#[derive(Clone)]
pub struct ProcessTreeItem {
    pub name: String,
    pub cmd: String,
    pub pid: u32,
    pub ppid: u32,
    pub cpu: f64,
    pub memory: f64,
    pub children: Vec<ProcessTreeItem>,
}

/// List all processes visible on the system.
#[napi]
pub fn ps_list_processes() -> Result<Vec<ProcessTreeItem>> {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let mut items = Vec::new();
    for (pid, process) in sys.processes() {
        items.push(ProcessTreeItem {
            name: process.name().to_string_lossy().to_string(),
            cmd: process.cmd().iter().map(|s| s.to_string_lossy()).collect::<Vec<_>>().join(" "),
            pid: pid.as_u32(),
            ppid: process.parent().map(|p| p.as_u32()).unwrap_or(0),
            cpu: process.cpu_usage() as f64,
            memory: process.memory() as f64,
            children: Vec::new(),
        });
    }
    items.sort_by_key(|p| p.pid);
    Ok(items)
}

/// Build a process tree rooted at the given PID.
#[napi]
pub fn ps_list_process_tree(root_pid: u32) -> Result<ProcessTreeItem> {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);



    // Collect all processes
    let mut flat: HashMap<u32, ProcessTreeItem> = HashMap::new();
    for (pid, process) in sys.processes() {
        let pid_u32 = pid.as_u32();
        flat.insert(pid_u32, ProcessTreeItem {
            name: find_process_name(&process.cmd().iter().map(|s| s.to_string_lossy()).collect::<Vec<_>>().join(" "), &process.name().to_string_lossy()),
            cmd: process.cmd().iter().map(|s| s.to_string_lossy()).collect::<Vec<_>>().join(" "),
            pid: pid_u32,
            ppid: process.parent().map(|p| p.as_u32()).unwrap_or(0),
            cpu: process.cpu_usage() as f64,
            memory: process.memory() as f64,
            children: Vec::new(),
        });
    }

    // Build tree from flat list
    let pids: Vec<u32> = flat.keys().cloned().collect();
    let mut children_map: HashMap<u32, Vec<u32>> = HashMap::new();
    for &pid in &pids {
        if let Some(item) = flat.get(&pid) {
            children_map.entry(item.ppid).or_default().push(pid);
        }
    }

    // Sort children by PID
    for children in children_map.values_mut() {
        children.sort();
    }

    fn build_tree(pid: u32, flat: &HashMap<u32, ProcessTreeItem>, children_map: &HashMap<u32, Vec<u32>>) -> Option<ProcessTreeItem> {
        let mut item = flat.get(&pid)?.clone();
        if let Some(child_pids) = children_map.get(&pid) {
            for &cpid in child_pids {
                if let Some(child) = build_tree(cpid, flat, children_map) {
                    item.children.push(child);
                }
            }
        }
        Some(item)
    }

    build_tree(root_pid, &flat, &children_map)
        .ok_or_else(|| Error::from_reason(format!("Root process {} not found", root_pid)))
}

/// Get info about a single process by PID.
#[napi]
pub fn ps_get_process_info(pid: u32) -> Result<ProcessTreeItem> {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let spid = sysinfo::Pid::from_u32(pid);
    let process = sys.process(spid)
        .ok_or_else(|| Error::from_reason(format!("Process {} not found", pid)))?;

    Ok(ProcessTreeItem {
        name: find_process_name(&process.cmd().iter().map(|s| s.to_string_lossy()).collect::<Vec<_>>().join(" "), &process.name().to_string_lossy()),
        cmd: process.cmd().iter().map(|s| s.to_string_lossy()).collect::<Vec<_>>().join(" "),
        pid,
        ppid: process.parent().map(|p| p.as_u32()).unwrap_or(0),
        cpu: process.cpu_usage() as f64,
        memory: process.memory() as f64,
        children: Vec::new(),
    })
}

/// Kill a process by PID.
#[napi]
pub fn kill_process(pid: u32, force: Option<bool>) -> Result<bool> {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let spid = sysinfo::Pid::from_u32(pid);
    if let Some(process) = sys.process(spid) {
        if force.unwrap_or(false) {
            Ok(process.kill())
        } else {
            // Graceful first
            Ok(process.kill())
        }
    } else {
        Err(Error::from_reason(format!("Process {} not found", pid)))
    }
}

/// Kill a process tree (process + all descendants).
#[napi]
pub fn ps_kill_process_tree(root_pid: u32, _force: Option<bool>) -> Result<u32> {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    // Find all descendants
    let mut to_kill = Vec::new();
    collect_descendants(root_pid, &sys, &mut to_kill);
    to_kill.push(root_pid);

    let mut killed = 0u32;
    // Kill children first (reverse order)
    for &pid in to_kill.iter().rev() {
        let spid = sysinfo::Pid::from_u32(pid);
        if let Some(process) = sys.process(spid) {
            if process.kill() { killed += 1; }
        }
    }
    Ok(killed)
}

fn collect_descendants(pid: u32, sys: &sysinfo::System, result: &mut Vec<u32>) {
    for (child_pid, process) in sys.processes() {
        if process.parent().map(|p| p.as_u32()) == Some(pid) {
            let cpid = child_pid.as_u32();
            result.push(cpid);
            collect_descendants(cpid, sys, result);
        }
    }
}

/// Identify a process by its command line (matches Electron/Node patterns).
fn find_process_name(cmd: &str, default_name: &str) -> String {
    // Check for --type=xxx pattern (Electron child processes)
    if let Some(caps) = regex::Regex::new(r"--type=([a-zA-Z-]+)").ok().and_then(|r| r.captures(cmd)) {
        let type_val = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        return match type_val {
            "renderer" => "window".to_string(),
            "utility" => {
                if cmd.contains("--utility-sub-type=network") {
                    "utility-network-service".to_string()
                } else {
                    "utility-process".to_string()
                }
            }
            "extensionHost" => "extension-host".to_string(),
            other => other.to_string(),
        };
    }

    // Check for crash reporter
    if cmd.contains("--crashes-directory") {
        return "electron-crash-reporter".to_string();
    }

    default_name.to_string()
}

/// Parse ps command output (Unix).
#[napi]
pub fn parse_ps_output(stdout: String) -> Vec<ProcessTreeItem> {
    let re = regex::Regex::new(r"^\s*(\d+)\s+(\d+)\s+(\d+\.?\d*)\s+(\d+\.?\d*)\s+(.+)$").unwrap();
    let mut items = Vec::new();

    for line in stdout.lines() {
        if let Some(caps) = re.captures(line.trim()) {
            let pid: u32 = caps[1].parse().unwrap_or(0);
            let ppid: u32 = caps[2].parse().unwrap_or(0);
            let cpu: f64 = caps[3].parse().unwrap_or(0.0);
            let mem: f64 = caps[4].parse().unwrap_or(0.0);
            let cmd = caps[5].to_string();
            items.push(ProcessTreeItem {
                name: find_process_name(&cmd, &cmd),
                cmd,
                pid,
                ppid,
                cpu,
                memory: mem,
                children: Vec::new(),
            });
        }
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_processes() {
        let procs = list_all_processes().unwrap();
        assert!(!procs.is_empty());
    }

    #[test]
    fn test_find_process_name() {
        assert_eq!(find_process_name("--type=renderer", "default"), "window");
        assert_eq!(find_process_name("--type=extensionHost", "default"), "extension-host");
        assert_eq!(find_process_name("--type=utility --utility-sub-type=network", "default"), "utility-network-service");
        assert_eq!(find_process_name("node app.js", "node"), "node");
    }

    #[test]
    fn test_parse_ps_output() {
        let output = "  123    1   5.0   2.3 /usr/bin/foo --bar\n  456  123   1.2   0.5 /usr/bin/baz\n";
        let items = parse_ps_output(output.into());
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].pid, 123);
        assert_eq!(items[1].ppid, 123);
    }
}
