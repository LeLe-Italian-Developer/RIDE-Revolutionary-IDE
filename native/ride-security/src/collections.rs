/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Generic collection data structures and algorithms — Rust port of
//! `src/vs/base/common/arrays.ts`, `map.ts`, `linkedList.ts`, `cache.ts`, etc.
//!
//! Provides sorted arrays, LRU caches, trie maps, bitmasks, set operations,
//! ring buffers, and more.

use napi_derive::napi;
use napi::bindgen_prelude::*;
use std::collections::{HashMap, HashSet, VecDeque, BTreeMap};

// ─── Sorted array operations ───────────────────────────────────────────────

/// Binary search in a sorted array of f64 values. Returns the index where
/// the target would be inserted to maintain sort order.
#[napi]
pub fn binary_search_f64(arr: Vec<f64>, target: f64) -> i32 {
    let mut lo = 0i64;
    let mut hi = arr.len() as i64 - 1;
    while lo <= hi {
        let mid = lo + (hi - lo) / 2;
        let val = arr[mid as usize];
        if (val - target).abs() < f64::EPSILON {
            return mid as i32;
        }
        if val < target {
            lo = mid + 1;
        } else {
            hi = mid - 1;
        }
    }
    -(lo as i32 + 1) // negative = not found, the insertion point
}

/// Binary search in a sorted string array. Returns the exact index or
/// negative insertion point.
#[napi]
pub fn binary_search_string(arr: Vec<String>, target: String) -> i32 {
    match arr.binary_search(&target) {
        Ok(i) => i as i32,
        Err(i) => -(i as i32 + 1),
    }
}

/// Merge two sorted arrays into one sorted array, deduplicating.
#[napi]
pub fn merge_sorted(a: Vec<String>, b: Vec<String>) -> Vec<String> {
    let mut result = Vec::with_capacity(a.len() + b.len());
    let mut ai = a.into_iter().peekable();
    let mut bi = b.into_iter().peekable();

    loop {
        match (ai.peek(), bi.peek()) {
            (None, None) => break,
            (Some(_), None) => { result.push(ai.next().unwrap()); }
            (None, Some(_)) => { result.push(bi.next().unwrap()); }
            (Some(av), Some(bv)) => {
                match av.cmp(bv) {
                    std::cmp::Ordering::Less => { result.push(ai.next().unwrap()); }
                    std::cmp::Ordering::Greater => { result.push(bi.next().unwrap()); }
                    std::cmp::Ordering::Equal => {
                        result.push(ai.next().unwrap());
                        bi.next();
                    }
                }
            }
        }
    }
    result
}

/// Remove duplicates from a sorted string array.
#[napi]
pub fn deduplicate_sorted(arr: Vec<String>) -> Vec<String> {
    if arr.is_empty() {
        return arr;
    }
    let mut result = Vec::with_capacity(arr.len());
    result.push(arr[0].clone());
    for item in arr.iter().skip(1) {
        if item != result.last().unwrap() {
            result.push(item.clone());
        }
    }
    result
}

// ─── Array utilities ───────────────────────────────────────────────────────

/// Find the first index where predicate is true (using string matching).
/// Returns -1 if not found.
#[napi]
pub fn find_index(arr: Vec<String>, value: String) -> i32 {
    arr.iter().position(|x| x == &value).map(|i| i as i32).unwrap_or(-1)
}

/// Find the last index of a value in an array. Returns -1 if not found.
#[napi]
pub fn find_last_index(arr: Vec<String>, value: String) -> i32 {
    arr.iter().rposition(|x| x == &value).map(|i| i as i32).unwrap_or(-1)
}

/// Check if two string arrays are equal.
#[napi]
pub fn arrays_equal(a: Vec<String>, b: Vec<String>) -> bool {
    a == b
}

/// Flatten a nested array into a single array (one level deep).
#[napi]
pub fn flatten(arrays: Vec<Vec<String>>) -> Vec<String> {
    arrays.into_iter().flatten().collect()
}

/// Chunk an array into sub-arrays of the given size.
#[napi]
pub fn chunk_array(arr: Vec<String>, size: u32) -> Vec<Vec<String>> {
    arr.chunks(size as usize)
        .map(|c| c.to_vec())
        .collect()
}

/// Shuffle an array using Fisher-Yates algorithm.
#[napi]
pub fn shuffle(mut arr: Vec<String>) -> Vec<String> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Deterministic pseudo-random for reproducibility
    let mut seed: u64 = arr.len() as u64;
    for (i, item) in arr.iter().enumerate() {
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        seed = seed.wrapping_add(hasher.finish()).wrapping_mul(6364136223846793005).wrapping_add(i as u64);
    }

    let len = arr.len();
    for i in (1..len).rev() {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let j = (seed as usize) % (i + 1);
        arr.swap(i, j);
    }
    arr
}

/// Remove an element at index, returns the new array.
#[napi]
pub fn remove_at(arr: Vec<String>, index: u32) -> Vec<String> {
    let mut result = arr;
    let idx = index as usize;
    if idx < result.len() {
        result.remove(idx);
    }
    result
}

/// Insert an element at a specific index.
#[napi]
pub fn insert_at(arr: Vec<String>, index: u32, value: String) -> Vec<String> {
    let mut result = arr;
    let idx = (index as usize).min(result.len());
    result.insert(idx, value);
    result
}

/// Get unique elements from an array, preserving order.
#[napi]
pub fn unique(arr: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    arr.into_iter()
        .filter(|x| seen.insert(x.clone()))
        .collect()
}

/// Compute the intersection of two arrays.
#[napi]
pub fn intersect(a: Vec<String>, b: Vec<String>) -> Vec<String> {
    let set_b: HashSet<String> = b.into_iter().collect();
    a.into_iter().filter(|x| set_b.contains(x)).collect()
}

/// Compute the difference (a - b) of two arrays.
#[napi]
pub fn difference(a: Vec<String>, b: Vec<String>) -> Vec<String> {
    let set_b: HashSet<String> = b.into_iter().collect();
    a.into_iter().filter(|x| !set_b.contains(x)).collect()
}

/// Compute the symmetric difference of two arrays.
#[napi]
pub fn symmetric_difference(a: Vec<String>, b: Vec<String>) -> Vec<String> {
    let set_a: HashSet<String> = a.iter().cloned().collect();
    let set_b: HashSet<String> = b.iter().cloned().collect();
    let mut result: Vec<String> = set_a.difference(&set_b).cloned().collect();
    result.extend(set_b.difference(&set_a).cloned());
    result
}

/// Group elements by a specified prefix before a separator.
#[napi]
pub fn group_by_prefix(arr: Vec<String>, separator: String) -> HashMap<String, Vec<String>> {
    let mut groups: HashMap<String, Vec<String>> = HashMap::new();
    for item in arr {
        let key = item
            .find(&separator)
            .map(|pos| item[..pos].to_string())
            .unwrap_or_else(|| String::new());
        groups.entry(key).or_default().push(item);
    }
    groups
}

// ─── LRU Cache ─────────────────────────────────────────────────────────────

/// A simple LRU (Least Recently Used) cache with string keys and values.
#[napi]
pub struct LruCache {
    capacity: usize,
    order: VecDeque<String>,
    map: HashMap<String, String>,
}

#[napi]
impl LruCache {
    #[napi(constructor)]
    pub fn new(capacity: u32) -> Self {
        LruCache {
            capacity: capacity.max(1) as usize,
            order: VecDeque::new(),
            map: HashMap::new(),
        }
    }

    #[napi]
    pub fn get(&mut self, key: String) -> Option<String> {
        if let Some(value) = self.map.get(&key).cloned() {
            // Move to front (most recently used)
            self.order.retain(|k| k != &key);
            self.order.push_front(key);
            Some(value)
        } else {
            None
        }
    }

    #[napi]
    pub fn set(&mut self, key: String, value: String) -> Option<String> {
        let evicted;
        if self.map.contains_key(&key) {
            self.order.retain(|k| k != &key);
            evicted = None;
        } else if self.order.len() >= self.capacity {
            // Evict least recently used
            if let Some(old_key) = self.order.pop_back() {
                evicted = self.map.remove(&old_key);
            } else {
                evicted = None;
            }
        } else {
            evicted = None;
        }
        self.order.push_front(key.clone());
        self.map.insert(key, value);
        evicted
    }

    #[napi]
    pub fn has(&self, key: String) -> bool {
        self.map.contains_key(&key)
    }

    #[napi]
    pub fn delete(&mut self, key: String) -> bool {
        if self.map.remove(&key).is_some() {
            self.order.retain(|k| k != &key);
            true
        } else {
            false
        }
    }

    #[napi]
    pub fn size(&self) -> u32 {
        self.map.len() as u32
    }

    #[napi]
    pub fn clear(&mut self) {
        self.map.clear();
        self.order.clear();
    }

    #[napi]
    pub fn keys(&self) -> Vec<String> {
        self.order.iter().cloned().collect()
    }
}

// ─── Trie map ──────────────────────────────────────────────────────────────

/// A trie (prefix tree) map for efficient prefix lookups.
#[napi]
pub struct TrieMap {
    children: HashMap<char, Box<TrieMapNode>>,
}

struct TrieMapNode {
    children: HashMap<char, Box<TrieMapNode>>,
    value: Option<String>,
    is_end: bool,
}

impl TrieMapNode {
    fn new() -> Self {
        TrieMapNode {
            children: HashMap::new(),
            value: None,
            is_end: false,
        }
    }
}

#[napi]
impl TrieMap {
    #[napi(constructor)]
    pub fn new() -> Self {
        TrieMap {
            children: HashMap::new(),
        }
    }

    #[napi]
    pub fn set(&mut self, key: String, value: String) {
        let chars: Vec<char> = key.chars().collect();
        let mut current_children = &mut self.children;

        for (i, &ch) in chars.iter().enumerate() {
            if !current_children.contains_key(&ch) {
                current_children.insert(ch, Box::new(TrieMapNode::new()));
            }
            let node = current_children.get_mut(&ch).unwrap();
            if i == chars.len() - 1 {
                node.value = Some(value.clone());
                node.is_end = true;
            }
            current_children = &mut node.children;
        }
    }

    #[napi]
    pub fn get(&self, key: String) -> Option<String> {
        let chars: Vec<char> = key.chars().collect();
        let mut current_children = &self.children;

        for (i, &ch) in chars.iter().enumerate() {
            match current_children.get(&ch) {
                Some(node) => {
                    if i == chars.len() - 1 && node.is_end {
                        return node.value.clone();
                    }
                    current_children = &node.children;
                }
                None => return None,
            }
        }
        None
    }

    #[napi]
    pub fn has_prefix(&self, prefix: String) -> bool {
        let chars: Vec<char> = prefix.chars().collect();
        let mut current_children = &self.children;

        for &ch in &chars {
            match current_children.get(&ch) {
                Some(node) => {
                    current_children = &node.children;
                }
                None => return false,
            }
        }
        true
    }

    #[napi]
    pub fn find_by_prefix(&self, prefix: String) -> Vec<String> {
        let chars: Vec<char> = prefix.chars().collect();
        let mut current_children = &self.children;
        let mut node_opt = None;

        for &ch in &chars {
            match current_children.get(&ch) {
                Some(node) => {
                    node_opt = Some(node);
                    current_children = &node.children;
                }
                None => return Vec::new(),
            }
        }

        let mut results = Vec::new();
        if let Some(node) = node_opt {
            Self::collect_values(node, &prefix, &mut results);
        }
        results
    }

    fn collect_values(node: &TrieMapNode, prefix: &str, results: &mut Vec<String>) {
        if node.is_end {
            if let Some(v) = &node.value {
                results.push(v.clone());
            }
        }
        for (&ch, child) in &node.children {
            let new_prefix = format!("{}{}", prefix, ch);
            Self::collect_values(child, &new_prefix, results);
        }
    }
}

// ─── Ring buffer ───────────────────────────────────────────────────────────

/// A fixed-size ring buffer for strings.
#[napi]
pub struct RingBuffer {
    buffer: Vec<Option<String>>,
    head: usize,
    count: usize,
}

#[napi]
impl RingBuffer {
    #[napi(constructor)]
    pub fn new(capacity: u32) -> Self {
        let cap = capacity.max(1) as usize;
        RingBuffer {
            buffer: vec![None; cap],
            head: 0,
            count: 0,
        }
    }

    #[napi]
    pub fn push(&mut self, value: String) {
        let capacity = self.buffer.len();
        let index = (self.head + self.count) % capacity;
        self.buffer[index] = Some(value);
        if self.count < capacity {
            self.count += 1;
        } else {
            self.head = (self.head + 1) % capacity;
        }
    }

    #[napi]
    pub fn to_array(&self) -> Vec<String> {
        let capacity = self.buffer.len();
        let mut result = Vec::with_capacity(self.count);
        for i in 0..self.count {
            let idx = (self.head + i) % capacity;
            if let Some(ref val) = self.buffer[idx] {
                result.push(val.clone());
            }
        }
        result
    }

    #[napi]
    pub fn size(&self) -> u32 {
        self.count as u32
    }

    #[napi]
    pub fn clear(&mut self) {
        self.buffer.fill(None);
        self.head = 0;
        self.count = 0;
    }
}

// ─── BTree-based sorted map ────────────────────────────────────────────────

/// A sorted map backed by a B-tree. Keys are always sorted.
#[napi]
pub struct SortedMap {
    inner: BTreeMap<String, String>,
}

#[napi]
impl SortedMap {
    #[napi(constructor)]
    pub fn new() -> Self {
        SortedMap { inner: BTreeMap::new() }
    }

    #[napi]
    pub fn set(&mut self, key: String, value: String) {
        self.inner.insert(key, value);
    }

    #[napi]
    pub fn get(&self, key: String) -> Option<String> {
        self.inner.get(&key).cloned()
    }

    #[napi]
    pub fn delete(&mut self, key: String) -> bool {
        self.inner.remove(&key).is_some()
    }

    #[napi]
    pub fn has(&self, key: String) -> bool {
        self.inner.contains_key(&key)
    }

    #[napi]
    pub fn size(&self) -> u32 {
        self.inner.len() as u32
    }

    #[napi]
    pub fn keys(&self) -> Vec<String> {
        self.inner.keys().cloned().collect()
    }

    #[napi]
    pub fn values(&self) -> Vec<String> {
        self.inner.values().cloned().collect()
    }

    #[napi]
    pub fn first_key(&self) -> Option<String> {
        self.inner.keys().next().cloned()
    }

    #[napi]
    pub fn last_key(&self) -> Option<String> {
        self.inner.keys().next_back().cloned()
    }

    #[napi]
    pub fn range(&self, from: String, to: String) -> Vec<String> {
        self.inner
            .range(from..=to)
            .map(|(k, _)| k.clone())
            .collect()
    }

    #[napi]
    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

// ─── Bitmask utilities ─────────────────────────────────────────────────────

/// Create a bitmask from a list of bit positions.
#[napi]
pub fn create_bitmask(positions: Vec<u32>) -> u64 {
    let mut mask: u64 = 0;
    for pos in positions {
        if pos < 64 {
            mask |= 1u64 << pos;
        }
    }
    mask
}

/// Check if a bit is set in a bitmask.
#[napi]
pub fn has_bit(mask: f64, position: u32) -> bool {
    let m = mask as u64;
    if position >= 64 { return false; }
    (m & (1u64 << position)) != 0
}

/// Set a bit in a bitmask.
#[napi]
pub fn set_bit(mask: f64, position: u32) -> f64 {
    let m = mask as u64;
    if position >= 64 { return mask; }
    (m | (1u64 << position)) as f64
}

/// Clear a bit in a bitmask.
#[napi]
pub fn clear_bit(mask: f64, position: u32) -> f64 {
    let m = mask as u64;
    if position >= 64 { return mask; }
    (m & !(1u64 << position)) as f64
}

/// Count the number of set bits in a bitmask.
#[napi]
pub fn count_bits(mask: f64) -> u32 {
    (mask as u64).count_ones()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_search() {
        let arr = vec![1.0, 3.0, 5.0, 7.0, 9.0];
        assert_eq!(binary_search_f64(arr.clone(), 5.0), 2);
        assert!(binary_search_f64(arr, 4.0) < 0);
    }

    #[test]
    fn test_merge_sorted() {
        let result = merge_sorted(
            vec!["a".into(), "c".into(), "e".into()],
            vec!["b".into(), "c".into(), "d".into()],
        );
        assert_eq!(result, vec!["a", "b", "c", "d", "e"]);
    }

    #[test]
    fn test_unique() {
        let result = unique(vec!["a".into(), "b".into(), "a".into(), "c".into(), "b".into()]);
        assert_eq!(result, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_chunk() {
        let result = chunk_array(vec!["1".into(), "2".into(), "3".into(), "4".into(), "5".into()], 2);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], vec!["1", "2"]);
    }

    #[test]
    fn test_lru_cache() {
        let mut cache = LruCache::new(2);
        cache.set("a".into(), "1".into());
        cache.set("b".into(), "2".into());
        assert_eq!(cache.get("a".into()), Some("1".into()));
        cache.set("c".into(), "3".into()); // evicts "b"
        assert_eq!(cache.get("b".into()), None);
        assert_eq!(cache.get("c".into()), Some("3".into()));
    }

    #[test]
    fn test_ring_buffer() {
        let mut rb = RingBuffer::new(3);
        rb.push("a".into());
        rb.push("b".into());
        rb.push("c".into());
        rb.push("d".into()); // evicts "a"
        assert_eq!(rb.to_array(), vec!["b", "c", "d"]);
    }

    #[test]
    fn test_sorted_map() {
        let mut map = SortedMap::new();
        map.set("c".into(), "3".into());
        map.set("a".into(), "1".into());
        map.set("b".into(), "2".into());
        assert_eq!(map.keys(), vec!["a", "b", "c"]);
        assert_eq!(map.first_key(), Some("a".into()));
    }

    #[test]
    fn test_bitmask() {
        let mask = create_bitmask(vec![0, 2, 4]);
        assert!(has_bit(mask as f64, 0));
        assert!(!has_bit(mask as f64, 1));
        assert!(has_bit(mask as f64, 2));
        assert_eq!(count_bits(mask as f64), 3);
    }

    #[test]
    fn test_intersect_difference() {
        let a = vec!["1".into(), "2".into(), "3".into()];
        let b = vec!["2".into(), "3".into(), "4".into()];
        assert_eq!(intersect(a.clone(), b.clone()), vec!["2", "3"]);
        assert_eq!(difference(a, b), vec!["1"]);
    }
}
