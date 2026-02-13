/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Lifecycle and error handling utilities — Rust port of `src/vs/base/common/lifecycle.ts`,
//! `errorHandling.ts`, and `errors.ts`.
//!
//! Provides disposable patterns, error types, cancellation, and resource management.

use napi_derive::napi;
use napi::bindgen_prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

// ─── Error types ───────────────────────────────────────────────────────────

/// Structured error information.
#[napi(object)]
pub struct ErrorInfo {
    pub name: String,
    pub message: String,
    pub stack: String,
    pub code: Option<String>,
    pub cause: Option<String>,
}

/// Create a structured error from a message and optional code.
#[napi]
pub fn create_error(message: String, code: Option<String>) -> ErrorInfo {
    ErrorInfo {
        name: "Error".to_string(),
        message: message.clone(),
        stack: format!("Error: {}", message),
        code,
        cause: None,
    }
}

/// Create a "not supported" error.
#[napi]
pub fn not_supported_error(feature: Option<String>) -> ErrorInfo {
    let msg = match feature {
        Some(f) => format!("{} is not supported", f),
        None => "Not supported".to_string(),
    };
    ErrorInfo {
        name: "NotSupportedError".to_string(),
        message: msg.clone(),
        stack: format!("NotSupportedError: {}", msg),
        code: Some("NOT_SUPPORTED".to_string()),
        cause: None,
    }
}

/// Create a "not implemented" error.
#[napi]
pub fn not_implemented_error(method: Option<String>) -> ErrorInfo {
    let msg = match method {
        Some(m) => format!("{} is not implemented", m),
        None => "Not implemented".to_string(),
    };
    ErrorInfo {
        name: "NotImplementedError".to_string(),
        message: msg.clone(),
        stack: format!("NotImplementedError: {}", msg),
        code: Some("NOT_IMPLEMENTED".to_string()),
        cause: None,
    }
}

/// Create a "cancelled" error.
#[napi]
pub fn cancelled_error() -> ErrorInfo {
    ErrorInfo {
        name: "CancelledError".to_string(),
        message: "Cancelled".to_string(),
        stack: "CancelledError: Cancelled".to_string(),
        code: Some("CANCELLED".to_string()),
        cause: None,
    }
}

/// Check if an error info represents a cancellation.
#[napi]
pub fn is_cancelled_error(error: ErrorInfo) -> bool {
    error.code.as_deref() == Some("CANCELLED") || error.name == "CancelledError"
}

/// Format an error for display.
#[napi]
pub fn format_error(error: ErrorInfo) -> String {
    let mut parts = vec![format!("{}: {}", error.name, error.message)];
    if let Some(code) = &error.code {
        parts.push(format!("  Code: {}", code));
    }
    if let Some(cause) = &error.cause {
        parts.push(format!("  Caused by: {}", cause));
    }
    parts.join("\n")
}

// ─── Cancellation Token ────────────────────────────────────────────────────

/// A token that can be used to signal cancellation of an operation.
#[napi]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

#[napi]
impl CancellationToken {
    #[napi(constructor)]
    pub fn new() -> Self {
        CancellationToken {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Check if cancellation has been requested.
    #[napi]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    /// Request cancellation.
    #[napi]
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    /// Reset the token (allow re-use).
    #[napi]
    pub fn reset(&self) {
        self.cancelled.store(false, Ordering::Relaxed);
    }
}

// ─── Disposable Store ──────────────────────────────────────────────────────

/// A store that manages disposable resources by string ID.
/// When disposed, all tracked resource IDs are returned for cleanup.
#[napi]
pub struct DisposableStore {
    resources: Vec<String>,
    disposed: bool,
}

#[napi]
impl DisposableStore {
    #[napi(constructor)]
    pub fn new() -> Self {
        DisposableStore {
            resources: Vec::new(),
            disposed: false,
        }
    }

    /// Register a resource ID for tracking.
    #[napi]
    pub fn add(&mut self, resource_id: String) -> bool {
        if self.disposed {
            return false;
        }
        self.resources.push(resource_id);
        true
    }

    /// Remove a resource ID from tracking.
    #[napi]
    pub fn remove(&mut self, resource_id: String) -> bool {
        if let Some(pos) = self.resources.iter().position(|r| r == &resource_id) {
            self.resources.remove(pos);
            true
        } else {
            false
        }
    }

    /// Dispose the store and return all tracked resource IDs for cleanup.
    #[napi]
    pub fn dispose(&mut self) -> Vec<String> {
        if self.disposed {
            return Vec::new();
        }
        self.disposed = true;
        std::mem::take(&mut self.resources)
    }

    /// Check if the store has been disposed.
    #[napi]
    pub fn is_disposed(&self) -> bool {
        self.disposed
    }

    /// Get the number of tracked resources.
    #[napi]
    pub fn size(&self) -> u32 {
        self.resources.len() as u32
    }

    /// Get all tracked resource IDs.
    #[napi]
    pub fn get_resources(&self) -> Vec<String> {
        self.resources.clone()
    }
}

// ─── Reference Counter ─────────────────────────────────────────────────────

/// A thread-safe reference counter for managing shared resource lifetimes.
#[napi]
pub struct RefCounter {
    counts: Arc<Mutex<HashMap<String, u32>>>,
}

#[napi]
impl RefCounter {
    #[napi(constructor)]
    pub fn new() -> Self {
        RefCounter {
            counts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Increment the reference count for a resource. Returns the new count.
    #[napi]
    pub fn acquire(&self, resource_id: String) -> u32 {
        let mut counts = self.counts.lock().unwrap();
        let count = counts.entry(resource_id).or_insert(0);
        *count += 1;
        *count
    }

    /// Decrement the reference count. Returns the new count.
    /// If the count reaches 0, the resource is removed and 0 is returned.
    #[napi]
    pub fn release(&self, resource_id: String) -> u32 {
        let mut counts = self.counts.lock().unwrap();
        if let Some(count) = counts.get_mut(&resource_id) {
            *count -= 1;
            if *count == 0 {
                counts.remove(&resource_id);
                return 0;
            }
            return *count;
        }
        0
    }

    /// Get the current reference count for a resource.
    #[napi]
    pub fn get_count(&self, resource_id: String) -> u32 {
        let counts = self.counts.lock().unwrap();
        *counts.get(&resource_id).unwrap_or(&0)
    }

    /// Get all resources with their reference counts.
    #[napi]
    pub fn all_counts(&self) -> HashMap<String, u32> {
        self.counts.lock().unwrap().clone()
    }
}

// ─── Retry logic ───────────────────────────────────────────────────────────

/// Configuration for retry attempts.
#[napi(object)]
pub struct RetryConfig {
    /// Maximum number of attempts (including the first try).
    pub max_attempts: u32,
    /// Initial delay between retries in ms.
    pub initial_delay_ms: u32,
    /// Factor to multiply delay by after each retry.
    pub backoff_factor: f64,
    /// Maximum delay between retries in ms.
    pub max_delay_ms: u32,
}

/// Compute the delay for a specific retry attempt.
#[napi]
pub fn compute_retry_delay(
    attempt: u32,
    initial_delay_ms: u32,
    backoff_factor: f64,
    max_delay_ms: u32,
) -> u32 {
    let delay = (initial_delay_ms as f64) * backoff_factor.powi(attempt as i32);
    (delay as u32).min(max_delay_ms)
}

/// Check if a retry should be attempted.
#[napi]
pub fn should_retry(attempt: u32, max_attempts: u32) -> bool {
    attempt < max_attempts
}

// ─── Timeout utilities ─────────────────────────────────────────────────────

/// Compute a timeout value with jitter (for preventing thundering herd).
#[napi]
pub fn timeout_with_jitter(base_ms: u32, jitter_factor: Option<f64>) -> u32 {
    let jitter = jitter_factor.unwrap_or(0.1);
    let uuid = uuid::Uuid::new_v4();
    let random = (uuid.as_bytes()[0] as f64) / 255.0;
    let variation = (base_ms as f64) * jitter * (2.0 * random - 1.0);
    ((base_ms as f64) + variation).max(0.0) as u32
}

/// Create an exponential backoff sequence of delays.
#[napi]
pub fn backoff_sequence(initial_ms: u32, factor: f64, count: u32, max_ms: Option<u32>) -> Vec<u32> {
    let max = max_ms.unwrap_or(u32::MAX);
    (0..count)
        .map(|i| {
            let delay = (initial_ms as f64) * factor.powi(i as i32);
            (delay as u32).min(max)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_types() {
        let err = create_error("test error".into(), Some("TEST_ERR".into()));
        assert_eq!(err.name, "Error");
        assert_eq!(err.message, "test error");
        assert_eq!(err.code, Some("TEST_ERR".into()));
    }

    #[test]
    fn test_cancellation_token() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled());
        token.reset();
        assert!(!token.is_cancelled());
    }

    #[test]
    fn test_disposable_store() {
        let mut store = DisposableStore::new();
        store.add("res1".into());
        store.add("res2".into());
        assert_eq!(store.size(), 2);

        let disposed = store.dispose();
        assert_eq!(disposed, vec!["res1", "res2"]);
        assert!(store.is_disposed());
        assert!(!store.add("res3".into())); // Can't add after dispose
    }

    #[test]
    fn test_ref_counter() {
        let rc = RefCounter::new();
        assert_eq!(rc.acquire("a".into()), 1);
        assert_eq!(rc.acquire("a".into()), 2);
        assert_eq!(rc.release("a".into()), 1);
        assert_eq!(rc.release("a".into()), 0);
    }

    #[test]
    fn test_retry_delay() {
        assert_eq!(compute_retry_delay(0, 100, 2.0, 10000), 100);
        assert_eq!(compute_retry_delay(1, 100, 2.0, 10000), 200);
        assert_eq!(compute_retry_delay(2, 100, 2.0, 10000), 400);
        assert_eq!(compute_retry_delay(10, 100, 2.0, 10000), 10000); // Capped
    }

    #[test]
    fn test_backoff_sequence() {
        let seq = backoff_sequence(100, 2.0, 5, Some(5000));
        assert_eq!(seq, vec![100, 200, 400, 800, 1600]);
    }
}
