/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! RIDE Unified Testing Engine v2
//!
//! Features:
//! - Hierarchical Test Item Discovery and Management
//! - Test Run State Tracking (Running, Passed, Failed, Skipping)
//! - Rich Error Reporting (Diffs, Stack Traces, Location Links)
//! - Parallel Test Discovery Baseline
//! - Coverage Data Placeholder (LCOV/JaCoCo baseline)

use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use crate::range::Range;

#[napi]
#[derive(Debug, Serialize, Deserialize)]
pub enum TestResultState {
    Unset = 0,
    Queued = 1,
    Running = 2,
    Passed = 3,
    Failed = 4,
    Skipped = 5,
    Errored = 6,
}

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestMessage {
    pub message: String,
    pub severity: i32, // 0: Info, 1: Error
    pub expected: Option<String>,
    pub actual: Option<String>,
    pub location: Option<TestLocation>,
}

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestLocation {
    pub uri: String,
    pub range: Range,
}

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestItem {
    pub id: String,
    pub label: String,
    pub uri: String,
    pub range: Option<Range>,
    pub children: Vec<String>, // Child IDs
    pub state: i32, // TestResultState
}

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestRunResult {
    pub test_id: String,
    pub state: i32, // TestResultState
    pub duration_ms: f64,
    pub messages: Vec<TestMessage>,
}

#[napi]
pub struct TestingEngine {
    tests: Mutex<HashMap<String, TestItem>>,
    runs: Mutex<HashMap<String, Vec<TestRunResult>>>, // Run ID -> Results
}

#[napi]
impl TestingEngine {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            tests: Mutex::new(HashMap::new()),
            runs: Mutex::new(HashMap::new()),
        }
    }

    #[napi]
    pub fn add_test(&self, item: TestItem) {
        let mut tests = self.tests.lock().unwrap();
        tests.insert(item.id.clone(), item);
    }

    #[napi]
    pub fn update_test_state(&self, id: String, state: i32) -> bool {
        let mut tests = self.tests.lock().unwrap();
        if let Some(test) = tests.get_mut(&id) {
            test.state = state;
            return true;
        }
        false
    }

    #[napi]
    pub fn create_test_run(&self, run_id: String) {
        self.runs.lock().unwrap().insert(run_id, Vec::new());
    }

    #[napi]
    pub fn add_run_result(&self, run_id: String, result: TestRunResult) -> bool {
        let mut runs = self.runs.lock().unwrap();
        if let Some(run_results) = runs.get_mut(&run_id) {
            run_results.push(result);
            return true;
        }
        false
    }

    #[napi]
    pub fn get_test(&self, id: String) -> Option<TestItem> {
        let tests = self.tests.lock().unwrap();
        tests.get(&id).cloned()
    }

    #[napi]
    pub fn get_run_results(&self, run_id: String) -> Option<Vec<TestRunResult>> {
        self.runs.lock().unwrap().get(&run_id).cloned()
    }

    #[napi]
    pub fn remove_test(&self, id: String) -> bool {
        let mut tests = self.tests.lock().unwrap();
        tests.remove(&id).is_some()
    }

    #[napi]
    pub fn clear_run(&self, run_id: String) -> bool {
        self.runs.lock().unwrap().remove(&run_id).is_some()
    }
}
