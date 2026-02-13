/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! RIDE Security Native Module
//!
//! Provides high-performance, memory-safe security primitives for RIDE:
//! - AES-256-GCM encryption/decryption
//! - SHA-256 file integrity verification
//! - URL allowlist network filtering

mod crypto;
mod integrity;
mod network;

pub use crypto::*;
pub use integrity::*;
pub use network::*;
