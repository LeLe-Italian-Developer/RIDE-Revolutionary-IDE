/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

/**
 * Rust-backed workspace service — thin TypeScript bridge to multiple native modules:
 *   - fs_watcher.rs  (file watching)
 *   - indexer.rs     (workspace file indexing)
 *   - process.rs     (process management)
 *   - logger.rs      (structured logging)
 *   - config.rs      (encrypted configuration)
 *   - extension_verify.rs (extension verification)
 *
 * All operations are performed in native Rust. This file contains zero business logic.
 */

import {
	IRideSecurityService,
	IConfigEntry,
	IExtensionAuditResult,
	IExtensionVerifyResult,
	IFileMetadata,
	IFsEvent,
	ILogEntry,
	IProcessInfo,
	IWatchHandle,
} from '../common/rideSecurityService.js';

/**
 * Convenience wrapper around `IRideSecurityService` for workspace operations:
 * file watching, indexing, process management, logging, config, and extension verification.
 */
export class RideWorkspaceService {

	constructor(private readonly _security: IRideSecurityService) { }

	// ─── File Watching ─────────────────────────────────────────────────

	/**
	 * Start watching a directory for changes.
	 * @param path - Directory to watch
	 * @param recursive - Whether to watch subdirectories (default: true)
	 * @param debounceMs - Debounce interval in ms (default: 100)
	 */
	startWatching(path: string, recursive = true, debounceMs = 100): IWatchHandle {
		return this._security.startWatching(path, recursive, debounceMs);
	}

	/**
	 * Stop a file watcher.
	 */
	stopWatching(watchId: string): boolean {
		return this._security.stopWatching(watchId);
	}

	/**
	 * Get buffered file system events from a watcher and clear the buffer.
	 */
	getWatchEvents(watchId: string): IFsEvent[] {
		return this._security.getWatchEvents(watchId);
	}

	// ─── Indexer ───────────────────────────────────────────────────────

	/**
	 * Index all files in a workspace directory.
	 * @returns The number of files indexed
	 */
	indexWorkspace(rootPath: string, excludeGlobs?: string[]): number {
		return this._security.indexWorkspace(rootPath, excludeGlobs);
	}

	/**
	 * Fuzzy find files by name in the workspace index.
	 */
	fuzzyFind(query: string, maxResults = 50): IFileMetadata[] {
		return this._security.fuzzyFind(query, maxResults);
	}

	/**
	 * Get metadata for a specific file from the workspace index.
	 */
	getFileInfo(filePath: string): IFileMetadata | null {
		return this._security.getFileInfo(filePath);
	}

	// ─── Process Manager ───────────────────────────────────────────────

	/**
	 * Spawn a managed child process.
	 */
	spawnProcess(command: string, args?: string[], cwd?: string): IProcessInfo {
		return this._security.spawnProcess(command, args, cwd);
	}

	/**
	 * Kill a managed process by its ID.
	 */
	killProcess(processId: string): boolean {
		return this._security.killProcess(processId);
	}

	/**
	 * List all currently managed processes.
	 */
	listProcesses(): IProcessInfo[] {
		return this._security.listProcesses();
	}

	/**
	 * Kill all managed processes (cleanup).
	 */
	killAllProcesses(): number {
		const procs = this._security.listProcesses();
		let killed = 0;
		for (const p of procs) {
			if (p.running && this._security.killProcess(p.id)) {
				killed++;
			}
		}
		return killed;
	}

	// ─── Logger ────────────────────────────────────────────────────────

	/**
	 * Log a structured message with level and optional source.
	 */
	log(level: 'debug' | 'info' | 'warn' | 'error', message: string, source?: string): void {
		this._security.logMessage(level, message, source);
	}

	/**
	 * Get recent log entries, optionally filtered by level.
	 */
	getLogs(maxEntries = 100, level?: string): ILogEntry[] {
		return this._security.getLogs(maxEntries, level);
	}

	/**
	 * Clear all log entries.
	 */
	clearLogs(): void {
		this._security.clearLogs();
	}

	// ─── Config ────────────────────────────────────────────────────────

	/**
	 * Get a configuration value by key.
	 */
	configGet(key: string): string | null {
		return this._security.configGet(key);
	}

	/**
	 * Set a configuration value. Pass encrypt=true for sensitive values.
	 */
	configSet(key: string, value: string, encrypt = false): void {
		this._security.configSet(key, value, encrypt);
	}

	/**
	 * Delete a configuration value.
	 */
	configDelete(key: string): boolean {
		return this._security.configDelete(key);
	}

	/**
	 * List all configuration entries.
	 */
	configList(): IConfigEntry[] {
		return this._security.configList();
	}

	// ─── Extension Verification ────────────────────────────────────────

	/**
	 * Verify the integrity of a VSIX extension package.
	 */
	verifyExtension(vsixPath: string, expectedHash?: string): IExtensionVerifyResult {
		return this._security.verifyExtension(vsixPath, expectedHash);
	}

	/**
	 * Audit an extension's permissions and assess its risk level.
	 */
	auditExtension(vsixPath: string): IExtensionAuditResult {
		return this._security.auditExtension(vsixPath);
	}

	/**
	 * Verify and audit an extension in one call.
	 */
	fullExtensionCheck(vsixPath: string, expectedHash?: string): {
		verification: IExtensionVerifyResult;
		audit: IExtensionAuditResult;
	} {
		return {
			verification: this._security.verifyExtension(vsixPath, expectedHash),
			audit: this._security.auditExtension(vsixPath),
		};
	}
}
