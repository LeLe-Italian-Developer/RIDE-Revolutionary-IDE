/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Date & time utilities — Rust port of `src/vs/base/common/date.ts` and `stopwatch.ts`.

use napi_derive::napi;
use napi::bindgen_prelude::*;
use std::time::{SystemTime, UNIX_EPOCH, Instant};
use std::sync::Mutex;

#[napi]
pub fn now_ms() -> f64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as f64
}

#[napi]
pub fn now_us() -> f64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_micros() as f64
}

#[napi]
pub fn format_iso(timestamp_ms: f64) -> String {
    let secs = (timestamp_ms / 1000.0) as i64;
    let nanos = ((timestamp_ms % 1000.0) * 1_000_000.0) as u32;
    let dt = chrono::DateTime::from_timestamp(secs, nanos);
    dt.map(|d| d.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()).unwrap_or_default()
}

#[napi]
pub fn parse_iso(iso: String) -> f64 {
    chrono::DateTime::parse_from_rfc3339(&iso)
        .map(|d| d.timestamp_millis() as f64)
        .unwrap_or(0.0)
}

#[napi]
pub fn format_relative(timestamp_ms: f64) -> String {
    let now = now_ms();
    let diff = now - timestamp_ms;
    if diff < 1000.0 { return "just now".into(); }
    if diff < 60_000.0 { return format!("{}s ago", (diff / 1000.0) as u32); }
    if diff < 3_600_000.0 { return format!("{}m ago", (diff / 60_000.0) as u32); }
    if diff < 86_400_000.0 { return format!("{}h ago", (diff / 3_600_000.0) as u32); }
    format!("{}d ago", (diff / 86_400_000.0) as u32)
}

#[napi]
pub fn format_duration(ms: f64) -> String {
    if ms < 1.0 { return format!("{:.0}µs", ms * 1000.0); }
    if ms < 1000.0 { return format!("{:.1}ms", ms); }
    if ms < 60_000.0 { return format!("{:.1}s", ms / 1000.0); }
    if ms < 3_600_000.0 { return format!("{:.1}m", ms / 60_000.0); }
    format!("{:.1}h", ms / 3_600_000.0)
}

static STOPWATCH_START: Mutex<Option<Instant>> = Mutex::new(None);

#[napi]
pub fn stopwatch_start() {
    *STOPWATCH_START.lock().unwrap() = Some(Instant::now());
}

#[napi]
pub fn stopwatch_elapsed_ms() -> f64 {
    STOPWATCH_START.lock().unwrap()
        .map(|s| s.elapsed().as_secs_f64() * 1000.0)
        .unwrap_or(0.0)
}

#[napi]
pub fn stopwatch_lap() -> f64 {
    let mut guard = STOPWATCH_START.lock().unwrap();
    let elapsed = guard.map(|s| s.elapsed().as_secs_f64() * 1000.0).unwrap_or(0.0);
    *guard = Some(Instant::now());
    elapsed
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_now() { assert!(now_ms() > 0.0); }
    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(500.0), "500.0ms");
        assert_eq!(format_duration(1500.0), "1.5s");
    }
    #[test]
    fn test_iso_roundtrip() {
        let ts = 1700000000000.0;
        let iso = format_iso(ts);
        let parsed = parse_iso(iso);
        assert!((parsed - ts).abs() < 1.0);
    }
}
