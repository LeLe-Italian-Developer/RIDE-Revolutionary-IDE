/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Port scanning utilities — Rust port of `src/vs/base/node/ports.ts`.
//! Find free ports, check port availability, browser-restricted port list.

use napi_derive::napi;
use napi::bindgen_prelude::*;
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::time::Duration;

/// Find a free port starting from the given port number.
#[napi]
pub fn find_free_port(start_port: u32, max_attempts: Option<u32>, hostname: Option<String>) -> u32 {
    let max = max_attempts.unwrap_or(100);
    let host = hostname.unwrap_or_else(|| "127.0.0.1".to_string());

    for i in 0..max {
        let port = start_port + i;
        if port > 65535 { break; }
        let addr = format!("{}:{}", host, port);
        if let Ok(listener) = TcpListener::bind(&addr) {
            drop(listener);
            return port;
        }
    }
    0 // No free port found
}

/// Check if a specific port is free.
#[napi]
pub fn is_port_free(port: u32, hostname: Option<String>) -> bool {
    let host = hostname.unwrap_or_else(|| "127.0.0.1".to_string());
    let addr = format!("{}:{}", host, port);
    TcpListener::bind(&addr).is_ok()
}

/// Check if a specific port is in use (has something listening).
#[napi]
pub fn is_port_in_use(port: u32, hostname: Option<String>, timeout_ms: Option<u32>) -> bool {
    let host = hostname.unwrap_or_else(|| "127.0.0.1".to_string());
    let timeout = Duration::from_millis(timeout_ms.unwrap_or(1000) as u64);
    let addr: std::result::Result<SocketAddr, _> = format!("{}:{}", host, port).parse();
    match addr {
        Ok(a) => TcpStream::connect_timeout(&a, timeout).is_ok(),
        Err(_) => false,
    }
}

/// Find multiple free ports at once.
#[napi]
pub fn find_free_ports(count: u32, start_port: Option<u32>) -> Vec<u32> {
    let mut ports = Vec::new();
    let mut port = start_port.unwrap_or(9000);

    while ports.len() < count as usize && port <= 65535 {
        if is_port_free(port, None) {
            ports.push(port);
        }
        port += 1;
    }
    ports
}

/// Check if a port number is restricted by browsers.
#[napi]
pub fn is_browser_restricted_port(port: u32) -> bool {
    matches!(port,
        1 | 7 | 9 | 11 | 13 | 15 | 17 | 19 | 20 | 21 | 22 | 23 | 25 |
        37 | 42 | 43 | 53 | 69 | 77 | 79 | 87 | 95 |
        101 | 102 | 103 | 104 | 109 | 110 | 111 | 113 | 115 | 117 | 119 | 123 | 135 | 137 | 139 | 143 | 161 | 179 |
        389 | 427 | 465 |
        512 | 513 | 514 | 515 | 526 | 530 | 531 | 532 | 540 | 548 | 554 | 556 | 563 | 587 | 601 | 636 |
        989 | 990 | 993 | 995 |
        1719 | 1720 | 1723 |
        2049 |
        3659 |
        4045 |
        5060 | 5061 |
        6000 | 6566 | 6665 | 6666 | 6667 | 6668 | 6669 | 6697 |
        10080
    )
}

/// Get all browser-restricted port numbers.
#[napi]
pub fn get_browser_restricted_ports() -> Vec<u32> {
    vec![
        1, 7, 9, 11, 13, 15, 17, 19, 20, 21, 22, 23, 25, 37, 42, 43, 53, 69, 77, 79, 87, 95,
        101, 102, 103, 104, 109, 110, 111, 113, 115, 117, 119, 123, 135, 137, 139, 143, 161, 179,
        389, 427, 465,
        512, 513, 514, 515, 526, 530, 531, 532, 540, 548, 554, 556, 563, 587, 601, 636,
        989, 990, 993, 995,
        1719, 1720, 1723, 2049, 3659, 4045, 5060, 5061,
        6000, 6566, 6665, 6666, 6667, 6668, 6669, 6697, 10080,
    ]
}

/// Find a random free port in the ephemeral range.
#[napi]
pub fn find_random_free_port() -> u32 {
    // Bind to port 0 — OS assigns a random free port
    if let Ok(listener) = TcpListener::bind("127.0.0.1:0") {
        if let Ok(addr) = listener.local_addr() {
            return addr.port() as u32;
        }
    }
    0
}

/// Validate a port number is in valid range.
#[napi]
pub fn is_valid_port(port: u32) -> bool {
    port > 0 && port <= 65535
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_free_port() {
        let port = find_free_port(49152, Some(10), None);
        assert!(port > 0);
        assert!(port >= 49152);
    }

    #[test]
    fn test_random_free_port() {
        let port = find_random_free_port();
        assert!(port > 0);
    }

    #[test]
    fn test_browser_restricted() {
        assert!(is_browser_restricted_port(22));
        assert!(is_browser_restricted_port(80) == false);
        assert!(is_browser_restricted_port(443) == false);
        assert!(is_browser_restricted_port(6667));
    }

    #[test]
    fn test_valid_port() {
        assert!(is_valid_port(8080));
        assert!(!is_valid_port(0));
        assert!(!is_valid_port(70000));
    }

    #[test]
    fn test_find_multiple() {
        let ports = find_free_ports(3, Some(49152));
        assert_eq!(ports.len(), 3);
    }
}
