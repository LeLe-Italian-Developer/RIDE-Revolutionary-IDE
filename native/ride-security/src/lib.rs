/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! RIDE Native Engine
//!
//! High-performance Rust backend providing:
//! - AES-256-GCM encryption/decryption
//! - SHA-256 file integrity verification
//! - URL allowlist network filtering
//! - File system watching with debouncing
//! - Parallel text search engine
//! - Myers diff algorithm
//! - Fuzzy workspace indexing
//! - Process lifecycle management
//! - Structured logging with rotation
//! - Encrypted configuration store
//! - Extension package verification
//! - Syntax analysis utilities
//! - ZSTD/ZIP compression engine

mod crypto;
mod integrity;
mod network;
mod fs_watcher;
mod search;
mod diff;
mod indexer;
mod process;
mod logger;
mod config;
mod extension_verify;
mod syntax;
mod compression;

// Phase 8: Base utilities
mod strings;
mod paths;
mod collections;
mod glob_engine;
mod hash_utils;
mod lifecycle;
mod json_parser;
mod platform_utils;
mod color;
mod date;
mod types;
mod buffer;
mod label;
mod marshalling;
mod async_utils;

pub use crypto::*;
pub use integrity::*;
pub use network::*;
pub use fs_watcher::*;
pub use search::*;
pub use diff::*;
pub use indexer::*;
pub use process::*;
pub use logger::*;
pub use config::*;
pub use extension_verify::*;
pub use syntax::*;
pub use compression::*;

// Phase 8: Base utilities
pub use strings::*;
pub use paths::*;
pub use collections::*;
pub use glob_engine::*;
pub use hash_utils::*;
pub use lifecycle::*;
pub use json_parser::*;
pub use platform_utils::*;
pub use color::*;
pub use date::*;
pub use types::*;
pub use buffer::*;
pub use label::*;
pub use marshalling::*;
pub use async_utils::*;

// Phase 9: Base Node utilities
mod pfs;
mod zip_utils;
mod ps;
mod terminals;
mod ports;

pub use pfs::*;
pub use zip_utils::*;
pub use ps::*;
pub use terminals::*;
pub use ports::*;
