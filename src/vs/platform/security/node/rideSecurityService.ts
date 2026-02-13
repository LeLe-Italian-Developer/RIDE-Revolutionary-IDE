/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Microsoft Corporation. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

import * as crypto from 'crypto';
import * as fs from 'fs';
import { createRequire } from 'node:module';
import { IEncryptResult, IRideSecurityService } from '../common/rideSecurityService.js';

/**
 * Node.js implementation of IRideSecurityService that delegates to the
 * Rust native module (ride-security) via napi-rs bindings.
 *
 * Falls back to Node.js crypto if the native module is not available
 * (e.g., during development or on unsupported platforms).
 */
export class RideSecurityService implements IRideSecurityService {

	declare readonly _serviceBrand: undefined;

	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	private _native: any = null;
	private _initialized = false;

	constructor() {
		this._tryLoadNative();
	}

	private _tryLoadNative(): void {
		if (this._initialized) {
			return;
		}
		this._initialized = true;

		try {
			const platform = process.platform;
			const arch = process.arch;
			const suffix = platform === 'win32' ? 'msvc' : platform === 'linux' ? 'gnu' : '';
			const name = `ride-security.${platform}-${arch}${suffix ? '-' + suffix : ''}.node`;
			// Native .node modules require dynamic loading via createRequire
			const nativeRequire = createRequire(import.meta.url);
			this._native = nativeRequire(`../../../../native/ride-security/${name}`);
		} catch {
			console.warn('[RIDE Security] Native module not available, running in fallback mode');
			this._native = null;
		}
	}

	// --- Crypto ---

	generateKey(): string {
		if (this._native?.generateKey) {
			return this._native.generateKey();
		}
		return crypto.randomBytes(32).toString('hex');
	}

	encrypt(plaintext: string, keyHex: string): IEncryptResult {
		if (this._native?.encrypt) {
			return this._native.encrypt(plaintext, keyHex);
		}
		const key = Buffer.from(keyHex, 'hex');
		const iv = crypto.randomBytes(12);
		const cipher = crypto.createCipheriv('aes-256-gcm', key, iv);
		let encrypted = cipher.update(plaintext, 'utf8', 'hex');
		encrypted += cipher.final('hex');
		const tag = (cipher as crypto.CipherGCM).getAuthTag().toString('hex');
		return { ciphertext: encrypted + tag, nonce: iv.toString('hex') };
	}

	decrypt(ciphertextHex: string, nonceHex: string, keyHex: string): string {
		if (this._native?.decrypt) {
			return this._native.decrypt(ciphertextHex, nonceHex, keyHex);
		}
		const key = Buffer.from(keyHex, 'hex');
		const iv = Buffer.from(nonceHex, 'hex');
		const tagLength = 32; // 16 bytes = 32 hex chars
		const ct = ciphertextHex.slice(0, -tagLength);
		const tag = Buffer.from(ciphertextHex.slice(-tagLength), 'hex');
		const decipher = crypto.createDecipheriv('aes-256-gcm', key, iv);
		(decipher as crypto.DecipherGCM).setAuthTag(tag);
		let decrypted = decipher.update(ct, 'hex', 'utf8');
		decrypted += decipher.final('utf8');
		return decrypted;
	}

	// --- Integrity ---

	hashFile(filePath: string): string {
		if (this._native?.hashFile) {
			return this._native.hashFile(filePath);
		}
		const content = fs.readFileSync(filePath);
		return crypto.createHash('sha256').update(content).digest('hex');
	}

	hashString(data: string): string {
		if (this._native?.hashString) {
			return this._native.hashString(data);
		}
		return crypto.createHash('sha256').update(data).digest('hex');
	}

	verifyFileIntegrity(filePath: string, expectedHash: string): boolean {
		if (this._native?.verifyFileIntegrity) {
			return this._native.verifyFileIntegrity(filePath, expectedHash);
		}
		const actualHash = this.hashFile(filePath);
		return actualHash === expectedHash.toLowerCase();
	}

	// --- Network ---

	isUrlAllowed(url: string): boolean {
		if (this._native?.isUrlAllowed) {
			return this._native.isUrlAllowed(url);
		}
		return true;
	}

	addAllowedDomain(domain: string): void {
		this._native?.addAllowedDomain?.(domain);
	}

	removeAllowedDomain(domain: string): void {
		this._native?.removeAllowedDomain?.(domain);
	}

	getBlockedDomains(): string[] {
		return this._native?.getBlockedDomains?.() ?? [];
	}

	getDefaultAllowedDomains(): string[] {
		return this._native?.getDefaultAllowedDomains?.() ?? [];
	}
}
