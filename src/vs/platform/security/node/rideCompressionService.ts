/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

/**
 * Rust-backed compression service — thin TypeScript bridge to compression.rs.
 *
 * Provides ZSTD compression/decompression, ZIP archive creation/extraction,
 * and archive listing. All compression logic runs in native Rust for maximum
 * throughput. This file contains zero business logic.
 */

import {
	IRideSecurityService,
	IArchiveEntry,
} from '../common/rideSecurityService.js';

/**
 * Convenience wrapper around `IRideSecurityService` for compression operations.
 * Uses the shared native module loader — no separate binary loading needed.
 */
export class RideCompressionService {

	constructor(private readonly _security: IRideSecurityService) { }

	/**
	 * Compress a string with ZSTD at the given level (1-22, default 3).
	 */
	compress(data: string, level?: number): Buffer {
		return this._security.compress(data, level);
	}

	/**
	 * Decompress ZSTD-compressed data back to a string.
	 */
	decompress(data: Buffer): string {
		return this._security.decompress(data);
	}

	/**
	 * Create a ZIP archive from a list of files.
	 * @param outputPath - Where to write the .zip file
	 * @param files - Absolute paths to include
	 * @param basePath - Optional base to make paths relative within the archive
	 * @returns The path to the created archive
	 */
	createZipArchive(outputPath: string, files: string[], basePath?: string): string {
		return this._security.createZipArchive(outputPath, files, basePath);
	}

	/**
	 * Extract all files from a ZIP archive.
	 * @returns List of extracted file paths
	 */
	extractArchive(archivePath: string, outputDir: string): string[] {
		return this._security.extractArchive(archivePath, outputDir);
	}

	/**
	 * List entries in a ZIP archive without extracting.
	 */
	listArchive(archivePath: string): IArchiveEntry[] {
		return this._security.listArchive(archivePath);
	}

	/**
	 * Get the total uncompressed size of an archive.
	 */
	getArchiveSize(archivePath: string): number {
		const entries = this._security.listArchive(archivePath);
		return entries.reduce((sum, e) => sum + e.size, 0);
	}

	/**
	 * Get the compression ratio of an archive.
	 */
	getCompressionRatio(archivePath: string): number {
		const entries = this._security.listArchive(archivePath);
		const totalSize = entries.reduce((sum, e) => sum + e.size, 0);
		const compressedSize = entries.reduce((sum, e) => sum + e.compressedSize, 0);
		return totalSize > 0 ? compressedSize / totalSize : 1;
	}
}
