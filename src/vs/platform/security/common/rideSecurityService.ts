/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

import { createDecorator } from '../../instantiation/common/instantiation.js';

// ─── Crypto & Integrity Types ────────────────────────────────────────────────

/** Result of an encryption operation. */
export interface IEncryptResult {
	/** Hex-encoded ciphertext */
	ciphertext: string;
	/** Hex-encoded nonce/IV */
	nonce: string;
}

// ─── Diff Types ──────────────────────────────────────────────────────────────

/** A single change hunk from a diff operation. */
export interface IDiffHunk {
	oldStart: number;
	oldCount: number;
	newStart: number;
	newCount: number;
	content: string;
}

/** Result of a diff computation. */
export interface IDiffResult {
	hunks: IDiffHunk[];
	additions: number;
	deletions: number;
	isIdentical: boolean;
}

// ─── Search Types ────────────────────────────────────────────────────────────

/** A single search match within a file. */
export interface ISearchMatch {
	filePath: string;
	lineNumber: number;
	column: number;
	lineContent: string;
	matchText: string;
	matchLength: number;
}

/** Options for search operations. */
export interface ISearchOptions {
	isRegex?: boolean;
	caseInsensitive?: boolean;
	includeGlobs?: string[];
	excludeGlobs?: string[];
	maxResults?: number;
	respectGitignore?: boolean;
	filenameOnly?: boolean;
	maxFileSize?: number;
	wholeWord?: boolean;
}

/** Result summary for a search operation. */
export interface ISearchResult {
	matches: ISearchMatch[];
	filesScanned: number;
	filesWithMatches: number;
	totalMatches: number;
	truncated: boolean;
	durationMs: number;
}

// ─── Compression Types ───────────────────────────────────────────────────────

/** An entry within an archive. */
export interface IArchiveEntry {
	name: string;
	size: number;
	compressedSize: number;
	isDirectory: boolean;
}

// ─── Syntax Types ────────────────────────────────────────────────────────────

/** Result of bracket matching analysis. */
export interface IBracketMatch {
	open: number;
	close: number;
	bracketType: string;
	depth: number;
}

/** Result of indentation detection. */
export interface IIndentationInfo {
	useTabs: boolean;
	tabSize: number;
	confidence: number;
}

/** Result of text analysis. */
export interface ITextStats {
	lines: number;
	words: number;
	characters: number;
	blankLines: number;
	maxLineLength: number;
	averageLineLength: number;
}

// ─── Workspace Types ─────────────────────────────────────────────────────────

/** File metadata from the workspace indexer. */
export interface IFileMetadata {
	path: string;
	relativePath: string;
	size: number;
	modifiedMs: number;
	extension: string;
	isDirectory: boolean;
}

/** File system event from the watcher. */
export interface IFsEvent {
	eventType: string;
	path: string;
	isDirectory: boolean;
	timestampMs: number;
}

/** Watch handle returned by startWatching. */
export interface IWatchHandle {
	id: string;
	path: string;
	recursive: boolean;
}

/** Process info. */
export interface IProcessInfo {
	id: string;
	pid: number;
	command: string;
	running: boolean;
}

/** A single structured log entry. */
export interface ILogEntry {
	level: string;
	message: string;
	timestampMs: number;
	source?: string;
}

// ─── Extension Verification Types ────────────────────────────────────────────

/** Result of extension verification. */
export interface IExtensionVerifyResult {
	valid: boolean;
	expectedHash: string;
	actualHash: string;
	sizeBytes: number;
}

/** Result of extension permission audit. */
export interface IExtensionAuditResult {
	extensionId: string;
	permissions: string[];
	riskLevel: string;
	warnings: string[];
}

// ─── Configuration Types ─────────────────────────────────────────────────────

/** Configuration store entry. */
export interface IConfigEntry {
	key: string;
	value: string;
	encrypted: boolean;
}

// ═══════════════════════════════════════════════════════════════════════════════
// Service Interface
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * RIDE Security Service — provides Rust-backed security, search, diff,
 * compression, syntax analysis, and workspace management primitives.
 *
 * All operations are performed in native Rust code via napi-rs for maximum
 * performance and memory safety. No TypeScript fallbacks — Rust is the sole
 * implementation.
 */
export interface IRideSecurityService {
	readonly _serviceBrand: undefined;

	// ─── Crypto ────────────────────────────────────────────────────────
	/** Generate a random 256-bit encryption key (hex-encoded) */
	generateKey(): string;
	/** Encrypt plaintext with AES-256-GCM */
	encrypt(plaintext: string, keyHex: string): IEncryptResult;
	/** Decrypt ciphertext with AES-256-GCM */
	decrypt(ciphertextHex: string, nonceHex: string, keyHex: string): string;

	// ─── Integrity ─────────────────────────────────────────────────────
	/** Compute SHA-256 hash of a file */
	hashFile(filePath: string): string;
	/** Compute SHA-256 hash of a string */
	hashString(data: string): string;
	/** Verify a file's integrity against expected hash */
	verifyFileIntegrity(filePath: string, expectedHash: string): boolean;

	// ─── Network ───────────────────────────────────────────────────────
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

	// ─── Diff ──────────────────────────────────────────────────────────
	/** Compute line-level diff between two texts */
	computeDiff(original: string, modified: string): IDiffResult;
	/** Generate unified diff format string */
	unifiedDiff(original: string, modified: string, originalLabel?: string, modifiedLabel?: string): string;
	/** Compute similarity ratio (0.0–1.0) between two texts */
	similarityRatio(textA: string, textB: string): number;

	// ─── Search ────────────────────────────────────────────────────────
	/** Search for text across all files in a directory */
	searchFiles(directory: string, query: string, options?: ISearchOptions): ISearchResult;
	/** Search within a single file */
	searchInFile(filePath: string, query: string, isRegex?: boolean, caseInsensitive?: boolean): ISearchMatch[];
	/** Fast count of pattern occurrences in a directory */
	countMatches(directory: string, query: string, isRegex?: boolean): number;

	// ─── Compression ───────────────────────────────────────────────────
	/** Compress data with ZSTD */
	compress(data: string, level?: number): Buffer;
	/** Decompress ZSTD data */
	decompress(data: Buffer): string;
	/** Create a ZIP archive from files */
	createZipArchive(outputPath: string, files: string[], basePath?: string): string;
	/** Extract a ZIP archive */
	extractArchive(archivePath: string, outputDir: string): string[];
	/** List entries in a ZIP archive */
	listArchive(archivePath: string): IArchiveEntry[];

	// ─── Syntax ────────────────────────────────────────────────────────
	/** Find matching bracket pairs in text */
	matchBrackets(text: string): IBracketMatch[];
	/** Detect indentation style of text */
	detectIndentation(text: string): IIndentationInfo;
	/** Normalize line endings to the specified style */
	normalizeLineEndings(text: string, style?: string): string;
	/** Analyze text statistics */
	analyzeText(text: string): ITextStats;

	// ─── Workspace: File Watching ──────────────────────────────────────
	/** Start watching a directory for changes */
	startWatching(path: string, recursive?: boolean, debounceMs?: number): IWatchHandle;
	/** Stop a file watcher */
	stopWatching(watchId: string): boolean;
	/** Get buffered file system events from a watcher */
	getWatchEvents(watchId: string): IFsEvent[];

	// ─── Workspace: Indexer ────────────────────────────────────────────
	/** Index all files in a workspace directory */
	indexWorkspace(rootPath: string, excludeGlobs?: string[]): number;
	/** Fuzzy find files by name in the index */
	fuzzyFind(query: string, maxResults?: number): IFileMetadata[];
	/** Get metadata for a specific file from the index */
	getFileInfo(filePath: string): IFileMetadata | null;

	// ─── Workspace: Processes ──────────────────────────────────────────
	/** Spawn a managed process */
	spawnProcess(command: string, args?: string[], cwd?: string): IProcessInfo;
	/** Kill a managed process by ID */
	killProcess(processId: string): boolean;
	/** List all managed processes */
	listProcesses(): IProcessInfo[];

	// ─── Workspace: Logger ─────────────────────────────────────────────
	/** Log a structured message */
	logMessage(level: string, message: string, source?: string): void;
	/** Get recent log entries */
	getLogs(maxEntries?: number, level?: string): ILogEntry[];
	/** Clear all log entries */
	clearLogs(): void;

	// ─── Workspace: Config ─────────────────────────────────────────────
	/** Get a configuration value */
	configGet(key: string): string | null;
	/** Set a configuration value (optionally encrypted) */
	configSet(key: string, value: string, encrypt?: boolean): void;
	/** Delete a configuration value */
	configDelete(key: string): boolean;
	/** List all configuration keys */
	configList(): IConfigEntry[];

	// ─── Extension Verification ────────────────────────────────────────
	/** Verify integrity of an extension package */
	verifyExtension(vsixPath: string, expectedHash?: string): IExtensionVerifyResult;
	/** Audit extension permissions and risk level */
	auditExtension(vsixPath: string): IExtensionAuditResult;
}

export const IRideSecurityService = createDecorator<IRideSecurityService>('rideSecurityService');
