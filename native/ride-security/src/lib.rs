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
