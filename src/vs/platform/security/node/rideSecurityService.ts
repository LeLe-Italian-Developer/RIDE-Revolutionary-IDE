/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

import { createRequire } from 'node:module';
import {
	IArchiveEntry,
	IBracketMatch,
	IConfigEntry,
	IDiffResult,
	IEncryptResult,
	IExtensionAuditResult,
	IExtensionVerifyResult,
	IFileMetadata,
	IFsEvent,
	IIndentationInfo,
	ILogEntry,
	IProcessInfo,
	IRideSecurityService,
	ISearchMatch,
	ISearchOptions,
	ISearchResult,
	ITextStats,
	IWatchHandle,
} from '../common/rideSecurityService.js';

/**
 * Node.js implementation of IRideSecurityService that delegates ALL calls
 * to the Rust native module (ride-security) via napi-rs bindings.
 *
 * No TypeScript fallbacks — Rust is the sole implementation. If the native
 * module is not available, methods throw a clear error.
 */
export class RideSecurityService implements IRideSecurityService {

	declare readonly _serviceBrand: undefined;

	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	private _native: any;

	constructor() {
		this._native = this._loadNative();
	}

	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	private _loadNative(): any {
		try {
			const platform = process.platform;
			const arch = process.arch;
			const suffix = platform === 'win32' ? 'msvc' : platform === 'linux' ? 'gnu' : '';
			const name = `ride-security.${platform}-${arch}${suffix ? '-' + suffix : ''}.node`;
			const nativeRequire = createRequire(import.meta.url);
			return nativeRequire(`../../../../native/ride-security/${name}`);
		} catch {
			console.error('[RIDE] Native Rust module not available. RIDE security features are disabled.');
			return null;
		}
	}

	private _requireNative(): void {
		if (!this._native) {
			throw new Error('[RIDE] Native Rust module is required but not loaded');
		}
	}

	// ═══════════════════════════════════════════════════════════════════
	// Crypto
	// ═══════════════════════════════════════════════════════════════════

	generateKey(): string {
		this._requireNative();
		return this._native.generateKey();
	}

	encrypt(plaintext: string, keyHex: string): IEncryptResult {
		this._requireNative();
		return this._native.encrypt(plaintext, keyHex);
	}

	decrypt(ciphertextHex: string, nonceHex: string, keyHex: string): string {
		this._requireNative();
		return this._native.decrypt(ciphertextHex, nonceHex, keyHex);
	}

	// ═══════════════════════════════════════════════════════════════════
	// Integrity
	// ═══════════════════════════════════════════════════════════════════

	hashFile(filePath: string): string {
		this._requireNative();
		return this._native.hashFile(filePath);
	}

	hashString(data: string): string {
		this._requireNative();
		return this._native.hashString(data);
	}

	verifyFileIntegrity(filePath: string, expectedHash: string): boolean {
		this._requireNative();
		return this._native.verifyFileIntegrity(filePath, expectedHash);
	}

	// ═══════════════════════════════════════════════════════════════════
	// Network
	// ═══════════════════════════════════════════════════════════════════

	isUrlAllowed(url: string): boolean {
		this._requireNative();
		return this._native.isUrlAllowed(url);
	}

	addAllowedDomain(domain: string): void {
		this._requireNative();
		this._native.addAllowedDomain(domain);
	}

	removeAllowedDomain(domain: string): void {
		this._requireNative();
		this._native.removeAllowedDomain(domain);
	}

	getBlockedDomains(): string[] {
		this._requireNative();
		return this._native.getBlockedDomains();
	}

	getDefaultAllowedDomains(): string[] {
		this._requireNative();
		return this._native.getDefaultAllowedDomains();
	}

	// ═══════════════════════════════════════════════════════════════════
	// Diff (delegates to diff.rs)
	// ═══════════════════════════════════════════════════════════════════

	computeDiff(original: string, modified: string): IDiffResult {
		this._requireNative();
		return this._native.computeDiff(original, modified);
	}

	unifiedDiff(original: string, modified: string, originalLabel?: string, modifiedLabel?: string): string {
		this._requireNative();
		return this._native.unifiedDiff(original, modified, originalLabel, modifiedLabel);
	}

	similarityRatio(textA: string, textB: string): number {
		this._requireNative();
		return this._native.similarityRatio(textA, textB);
	}

	// ═══════════════════════════════════════════════════════════════════
	// Search (delegates to search.rs)
	// ═══════════════════════════════════════════════════════════════════

	searchFiles(directory: string, query: string, options?: ISearchOptions): ISearchResult {
		this._requireNative();
		return this._native.searchFiles(directory, query, options);
	}

	searchInFile(filePath: string, query: string, isRegex?: boolean, caseInsensitive?: boolean): ISearchMatch[] {
		this._requireNative();
		return this._native.searchInFile(filePath, query, isRegex, caseInsensitive);
	}

	countMatches(directory: string, query: string, isRegex?: boolean): number {
		this._requireNative();
		return this._native.countMatches(directory, query, isRegex);
	}

	// ═══════════════════════════════════════════════════════════════════
	// Compression (delegates to compression.rs)
	// ═══════════════════════════════════════════════════════════════════

	compress(data: string, level?: number): Buffer {
		this._requireNative();
		return this._native.compress(data, level);
	}

	decompress(data: Buffer): string {
		this._requireNative();
		return this._native.decompress(data);
	}

	createZipArchive(outputPath: string, files: string[], basePath?: string): string {
		this._requireNative();
		return this._native.createZipArchive(outputPath, files, basePath);
	}

	extractArchive(archivePath: string, outputDir: string): string[] {
		this._requireNative();
		return this._native.extractArchive(archivePath, outputDir);
	}

	listArchive(archivePath: string): IArchiveEntry[] {
		this._requireNative();
		return this._native.listArchive(archivePath);
	}

	// ═══════════════════════════════════════════════════════════════════
	// Syntax (delegates to syntax.rs)
	// ═══════════════════════════════════════════════════════════════════

	matchBrackets(text: string): IBracketMatch[] {
		this._requireNative();
		return this._native.matchBrackets(text);
	}

	detectIndentation(text: string): IIndentationInfo {
		this._requireNative();
		return this._native.detectIndentation(text);
	}

	normalizeLineEndings(text: string, style?: string): string {
		this._requireNative();
		return this._native.normalizeLineEndings(text, style);
	}

	analyzeText(text: string): ITextStats {
		this._requireNative();
		return this._native.analyzeText(text);
	}

	// ═══════════════════════════════════════════════════════════════════
	// File Watching (delegates to fs_watcher.rs)
	// ═══════════════════════════════════════════════════════════════════

	startWatching(path: string, recursive?: boolean, debounceMs?: number): IWatchHandle {
		this._requireNative();
		return this._native.startWatching(path, recursive, debounceMs);
	}

	stopWatching(watchId: string): boolean {
		this._requireNative();
		return this._native.stopWatching(watchId);
	}

	getWatchEvents(watchId: string): IFsEvent[] {
		this._requireNative();
		return this._native.getWatchEvents(watchId);
	}

	// ═══════════════════════════════════════════════════════════════════
	// Indexer (delegates to indexer.rs)
	// ═══════════════════════════════════════════════════════════════════

	indexWorkspace(rootPath: string, excludeGlobs?: string[]): number {
		this._requireNative();
		return this._native.indexWorkspace(rootPath, excludeGlobs);
	}

	fuzzyFind(query: string, maxResults?: number): IFileMetadata[] {
		this._requireNative();
		return this._native.fuzzyFind(query, maxResults);
	}

	getFileInfo(filePath: string): IFileMetadata | null {
		this._requireNative();
		return this._native.getFileInfo(filePath) ?? null;
	}

	// ═══════════════════════════════════════════════════════════════════
	// Process Manager (delegates to process.rs)
	// ═══════════════════════════════════════════════════════════════════

	spawnProcess(command: string, args?: string[], cwd?: string): IProcessInfo {
		this._requireNative();
		return this._native.spawnProcess(command, args, cwd);
	}

	killProcess(processId: string): boolean {
		this._requireNative();
		return this._native.killProcess(processId);
	}

	listProcesses(): IProcessInfo[] {
		this._requireNative();
		return this._native.listProcesses();
	}

	// ═══════════════════════════════════════════════════════════════════
	// Logger (delegates to logger.rs)
	// ═══════════════════════════════════════════════════════════════════

	logMessage(level: string, message: string, source?: string): void {
		this._requireNative();
		this._native.logMessage(level, message, source);
	}

	getLogs(maxEntries?: number, level?: string): ILogEntry[] {
		this._requireNative();
		return this._native.getLogs(maxEntries, level);
	}

	clearLogs(): void {
		this._requireNative();
		this._native.clearLogs();
	}

	// ═══════════════════════════════════════════════════════════════════
	// Config (delegates to config.rs)
	// ═══════════════════════════════════════════════════════════════════

	configGet(key: string): string | null {
		this._requireNative();
		return this._native.configGet(key) ?? null;
	}

	configSet(key: string, value: string, encrypt?: boolean): void {
		this._requireNative();
		this._native.configSet(key, value, encrypt);
	}

	configDelete(key: string): boolean {
		this._requireNative();
		return this._native.configDelete(key);
	}

	configList(): IConfigEntry[] {
		this._requireNative();
		return this._native.configList();
	}

	// ═══════════════════════════════════════════════════════════════════
	// Extension Verification (delegates to extension_verify.rs)
	// ═══════════════════════════════════════════════════════════════════

	verifyExtension(vsixPath: string, expectedHash?: string): IExtensionVerifyResult {
		this._requireNative();
		return this._native.verifyExtension(vsixPath, expectedHash);
	}

	auditExtension(vsixPath: string): IExtensionAuditResult {
		this._requireNative();
		return this._native.auditExtension(vsixPath);
	}
}
