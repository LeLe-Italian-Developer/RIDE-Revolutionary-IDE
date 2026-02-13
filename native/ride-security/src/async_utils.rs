/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Async utilities — Rust port of `src/vs/base/common/async.ts`.
//! Debounce, throttle, rate limiting, and sequencing primitives.

use napi_derive::napi;
use napi::bindgen_prelude::*;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

/// Configuration for debounce/throttle behavior.
#[napi(object)]
pub struct TimingConfig {
    pub delay_ms: u32,
    pub max_wait_ms: Option<u32>,
    pub leading: Option<bool>,
    pub trailing: Option<bool>,
}

/// A sequence queue that tracks ordered execution.
#[napi]
pub struct SequenceQueue {
    queue: Arc<Mutex<VecDeque<String>>>,
    max_size: usize,
}

#[napi]
impl SequenceQueue {
    #[napi(constructor)]
    pub fn new(max_size: Option<u32>) -> Self {
        SequenceQueue {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            max_size: max_size.unwrap_or(1000) as usize,
        }
    }

    #[napi]
    pub fn enqueue(&self, item: String) -> bool {
        let mut q = self.queue.lock().unwrap();
        if q.len() >= self.max_size { return false; }
        q.push_back(item);
        true
    }

    #[napi]
    pub fn dequeue(&self) -> Option<String> {
        self.queue.lock().unwrap().pop_front()
    }

    #[napi]
    pub fn peek(&self) -> Option<String> {
        self.queue.lock().unwrap().front().cloned()
    }

    #[napi]
    pub fn size(&self) -> u32 {
        self.queue.lock().unwrap().len() as u32
    }

    #[napi]
    pub fn is_empty(&self) -> bool {
        self.queue.lock().unwrap().is_empty()
    }

    #[napi]
    pub fn clear(&self) {
        self.queue.lock().unwrap().clear();
    }

    #[napi]
    pub fn drain(&self) -> Vec<String> {
        self.queue.lock().unwrap().drain(..).collect()
    }
}

/// A simple rate limiter using token bucket algorithm.
#[napi]
pub struct RateLimiter {
    tokens: Arc<Mutex<f64>>,
    max_tokens: f64,
    refill_rate: f64,
    last_refill: Arc<Mutex<f64>>,
}

#[napi]
impl RateLimiter {
    #[napi(constructor)]
    pub fn new(max_tokens: f64, refill_per_second: f64) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default()
            .as_secs_f64();
        RateLimiter {
            tokens: Arc::new(Mutex::new(max_tokens)),
            max_tokens,
            refill_rate: refill_per_second,
            last_refill: Arc::new(Mutex::new(now)),
        }
    }

    #[napi]
    pub fn try_acquire(&self, count: Option<f64>) -> bool {
        let n = count.unwrap_or(1.0);
        self.refill();
        let mut tokens = self.tokens.lock().unwrap();
        if *tokens >= n { *tokens -= n; true } else { false }
    }

    #[napi]
    pub fn available_tokens(&self) -> f64 {
        self.refill();
        *self.tokens.lock().unwrap()
    }

    fn refill(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default()
            .as_secs_f64();
        let mut last = self.last_refill.lock().unwrap();
        let elapsed = now - *last;
        let new_tokens = elapsed * self.refill_rate;
        let mut tokens = self.tokens.lock().unwrap();
        *tokens = (*tokens + new_tokens).min(self.max_tokens);
        *last = now;
    }

    #[napi]
    pub fn reset(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default()
            .as_secs_f64();
        *self.tokens.lock().unwrap() = self.max_tokens;
        *self.last_refill.lock().unwrap() = now;
    }
}

/// A barrier that tracks pending operations.
#[napi]
pub struct Barrier {
    count: Arc<Mutex<u32>>,
    target: u32,
}

#[napi]
impl Barrier {
    #[napi(constructor)]
    pub fn new(target: u32) -> Self {
        Barrier { count: Arc::new(Mutex::new(0)), target }
    }

    #[napi]
    pub fn signal(&self) -> bool {
        let mut count = self.count.lock().unwrap();
        *count += 1;
        *count >= self.target
    }

    #[napi]
    pub fn is_complete(&self) -> bool {
        *self.count.lock().unwrap() >= self.target
    }

    #[napi]
    pub fn progress(&self) -> f64 {
        if self.target == 0 { return 1.0; }
        *self.count.lock().unwrap() as f64 / self.target as f64
    }

    #[napi]
    pub fn reset(&self) {
        *self.count.lock().unwrap() = 0;
    }
}

/// Idle value — computes a value lazily and caches it.
#[napi]
pub struct IdleValue {
    value: Arc<Mutex<Option<String>>>,
}

#[napi]
impl IdleValue {
    #[napi(constructor)]
    pub fn new() -> Self {
        IdleValue { value: Arc::new(Mutex::new(None)) }
    }

    #[napi]
    pub fn set(&self, value: String) {
        *self.value.lock().unwrap() = Some(value);
    }

    #[napi]
    pub fn get(&self) -> Option<String> {
        self.value.lock().unwrap().clone()
    }

    #[napi]
    pub fn is_set(&self) -> bool {
        self.value.lock().unwrap().is_some()
    }

    #[napi]
    pub fn clear(&self) {
        *self.value.lock().unwrap() = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sequence_queue() {
        let q = SequenceQueue::new(Some(3));
        assert!(q.enqueue("a".into()));
        assert!(q.enqueue("b".into()));
        assert!(q.enqueue("c".into()));
        assert!(!q.enqueue("d".into())); // full
        assert_eq!(q.dequeue(), Some("a".into()));
        assert_eq!(q.size(), 2);
    }
    #[test]
    fn test_rate_limiter() {
        let rl = RateLimiter::new(5.0, 10.0);
        assert!(rl.try_acquire(None));
        assert!(rl.try_acquire(Some(4.0)));
        assert!(!rl.try_acquire(None)); // depleted
    }
    #[test]
    fn test_barrier() {
        let b = Barrier::new(3);
        assert!(!b.signal());
        assert!(!b.signal());
        assert!(b.signal()); // 3rd signal completes
        assert!(b.is_complete());
    }
}
