/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rayon::prelude::*;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};

#[napi(object)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchServiceResult {
    pub file: String,
    pub line: u32,
    pub preview: String,
}

#[napi]
pub struct SearchService {
    num_threads: usize,
}

#[napi]
impl SearchService {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            num_threads: num_cpus::get(),
        }
    }

    #[napi]
    pub fn text_search(&self, root: String, pattern: String, include_pattern: Option<String>) -> Vec<SearchServiceResult> {
        let re = match Regex::new(&pattern) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        let mut walker_builder = WalkBuilder::new(&root);
        walker_builder.hidden(true).git_ignore(true);
        if let Some(inc) = include_pattern {
            let mut ov_builder = ignore::overrides::OverrideBuilder::new(&root);
            let _ = ov_builder.add(&inc);
            if let Ok(ov) = ov_builder.build() {
                walker_builder.overrides(ov);
            }
        }

        // Build the list of files to search
        let files: Vec<std::path::PathBuf> = walker_builder.build()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .map(|e| e.into_path())
            .collect();

        // Perform parallel search across files
        files.into_par_iter()
            .map(|path| {
                let mut matches = Vec::new();
                if let Ok(file) = File::open(&path) {
                    let reader = BufReader::new(file);
                    for (idx, line_res) in reader.lines().enumerate() {
                        if let Ok(line) = line_res {
                            if re.is_match(&line) {
                                matches.push(SearchServiceResult {
                                    file: path.to_string_lossy().to_string(),
                                    line: (idx + 1) as u32,
                                    preview: line.trim().to_string(),
                                });
                            }
                        }
                        // Limit results per file to avoid explosion
                        if matches.len() > 100 { break; }
                    }
                }
                matches
            })
            .flatten()
            .collect()
    }

    #[napi]
    pub fn file_search(&self, root: String, query: String) -> Vec<String> {
        let q = query.to_lowercase();
        let walker = WalkBuilder::new(&root)
            .hidden(true)
            .git_ignore(true)
            .build();

        walker.filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .filter_map(|e| {
                let path_str = e.path().to_string_lossy().to_string();
                if path_str.to_lowercase().contains(&q) {
                    Some(path_str)
                } else {
                    None
                }
            })
            .collect()
    }
}
