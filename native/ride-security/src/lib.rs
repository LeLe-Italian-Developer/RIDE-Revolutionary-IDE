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
//! - Core Editor Text Model (Piece Tree, Cursor, Range, Position)

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


// Phase 10: Editor Core Engine
mod position;
mod range;
mod selection;
mod text_model_types;
mod piece_tree;
mod text_model;
mod cursor;
mod edit_stack;
mod view_model;
mod tokenizer;
mod text_edit;
mod editor_config;
mod editor_core;

pub use position::*;
pub use range::*;
pub use selection::*;
pub use text_model_types::*;
pub use piece_tree::*;
pub use text_model::*;
pub use cursor::*;
pub use edit_stack::*;
pub use view_model::*;
pub use tokenizer::*;
pub use text_edit::*;
pub use editor_config::*;
pub use editor_core::*;

// Phase 11: Editor Contrib (Algorithms)
mod snippet_parser;
mod color_picker;
mod link_detector;
mod word_ops;
mod suggest;

pub use snippet_parser::*;
pub use color_picker::*;
pub use link_detector::*;
pub use word_ops::*;
pub use suggest::*;

// Phase 12: Platform Services
mod user_data_sync;
mod terminal_backend;
mod ext_mgmt;
mod file_service;
mod quickinput;
mod window_mgmt;
mod theme_engine;
mod contextkey_eval;
mod action_registry;
mod storage_engine;
mod remote;
mod update_service;
mod log_service;
mod env_service;
mod config_service;

pub use user_data_sync::*;
pub use terminal_backend::*;
pub use ext_mgmt::*;
pub use file_service::*;
pub use quickinput::*;
pub use window_mgmt::*;
pub use theme_engine::*;
pub use contextkey_eval::*;
pub use action_registry::*;
pub use storage_engine::*;
pub use remote::*;
pub use update_service::*;
pub use log_service::*;
pub use env_service::*;
pub use config_service::*;

// Phase 13: Workbench Services
mod keybinding_resolver;
mod ext_host;
mod working_copy;
mod search_service;
mod theme_service;
mod ext_mgmt_service;
mod text_file;
mod auth_service;
mod editor_service;
mod config_resolver;
mod preferences;
mod user_profile;
mod workspace;
mod history;

pub use keybinding_resolver::*;
pub use ext_host::*;
pub use working_copy::*;
pub use search_service::*;
pub use theme_service::*;
pub use ext_mgmt_service::*;
pub use text_file::*;
pub use auth_service::*;
pub use editor_service::*;
pub use config_resolver::*;
pub use preferences::*;
pub use user_profile::*;
pub use workspace::*;
pub use history::*;

// Phase 14: Workbench API Layer
mod ext_api_commands;
mod ext_api_types;
mod ext_host_documents;
mod ext_host_editors;
mod ext_host_languages;
mod ext_host_workspace;
mod ext_host_terminal;
mod ext_host_debug;
mod ext_host_scm;

pub use ext_api_commands::*;
pub use ext_api_types::*;
pub use ext_host_documents::*;
pub use ext_host_editors::*;
pub use ext_host_languages::*;
pub use ext_host_workspace::*;
pub use ext_host_terminal::*;
pub use ext_host_debug::*;
pub use ext_host_scm::*;

// Phase 15: Workbench Contrib — Large Features
mod chat_engine;
mod notebook_engine;
mod debug_engine;
mod terminal_engine;
mod testing_engine;
mod mcp_engine;

pub use chat_engine::*;
pub use notebook_engine::*;
pub use debug_engine::*;
pub use terminal_engine::*;
pub use testing_engine::*;
pub use mcp_engine::*;
