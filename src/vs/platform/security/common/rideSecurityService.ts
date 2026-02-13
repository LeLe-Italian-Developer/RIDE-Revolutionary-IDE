/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Microsoft Corporation. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

import { createDecorator } from '../../instantiation/common/instantiation.js';

/**
 * Result of an encryption operation.
 */
export interface IEncryptResult {
	/** Hex-encoded ciphertext */
	ciphertext: string;
	/** Hex-encoded nonce/IV */
	nonce: string;
}

/**
 * RIDE Security Service — provides Rust-backed security primitives.
 *
 * All cryptographic operations are performed in native Rust code via napi-rs
 * for maximum performance and memory safety.
 */
export interface IRideSecurityService {
	readonly _serviceBrand: undefined;

	// --- Crypto ---
	/** Generate a random 256-bit encryption key (hex-encoded) */
	generateKey(): string;

	/** Encrypt plaintext with AES-256-GCM */
	encrypt(plaintext: string, keyHex: string): IEncryptResult;

	/** Decrypt ciphertext with AES-256-GCM */
	decrypt(ciphertextHex: string, nonceHex: string, keyHex: string): string;

	// --- Integrity ---
	/** Compute SHA-256 hash of a file */
	hashFile(filePath: string): string;

	/** Compute SHA-256 hash of a string */
	hashString(data: string): string;

	/** Verify a file's integrity against expected hash */
	verifyFileIntegrity(filePath: string, expectedHash: string): boolean;

	// --- Network ---
	/** Check if a URL is allowed by the network filter */
	isUrlAllowed(url: string): boolean;

	/** Add a domain to the custom allowlist */
	addAllowedDomain(domain: string): void;

	/** Remove a domain from the custom allowlist */
	removeAllowedDomain(domain: string): void;

	/** Get list of blocked domains */
	getBlockedDomains(): string[];

	/** Get list of default allowed domains */
	getDefaultAllowedDomains(): string[];
}

export const IRideSecurityService = createDecorator<IRideSecurityService>('rideSecurityService');
