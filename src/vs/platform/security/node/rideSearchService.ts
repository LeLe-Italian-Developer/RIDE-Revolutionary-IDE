/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

/**
 * Rust-backed search service — thin TypeScript bridge to search.rs native module.
 *
 * Provides parallel text search with regex support, gitignore awareness, and
 * configurable file filters. All search logic runs in Rust using rayon for
 * multi-threaded file scanning. This file contains zero business logic.
 */

import {
	IRideSecurityService,
	ISearchMatch,
	ISearchOptions,
	ISearchResult,
} from '../common/rideSecurityService.js';

/**
 * Convenience wrapper around `IRideSecurityService` for search operations.
 * Uses the shared native module loader — no separate binary loading needed.
 */
export class RideSearchService {

	constructor(private readonly _security: IRideSecurityService) { }

	/**
	 * Full-text search across a directory (parallel, gitignore-aware).
	 */
	searchFiles(directory: string, query: string, options?: ISearchOptions): ISearchResult {
		return this._security.searchFiles(directory, query, options);
	}

	/**
	 * Search within a single file.
	 */
	searchInFile(
		filePath: string,
		query: string,
		isRegex = false,
		caseInsensitive = false,
	): ISearchMatch[] {
		return this._security.searchInFile(filePath, query, isRegex, caseInsensitive);
	}

	/**
	 * Fast count of pattern matches across a directory (no line details).
	 */
	countMatches(directory: string, query: string, isRegex = false): number {
		return this._security.countMatches(directory, query, isRegex);
	}

	/**
	 * Case-insensitive literal search (convenience method).
	 */
	searchCaseInsensitive(directory: string, query: string): ISearchResult {
		return this._security.searchFiles(directory, query, { caseInsensitive: true });
	}

	/**
	 * Search only in filenames (not content).
	 */
	searchFilenames(directory: string, query: string): ISearchResult {
		return this._security.searchFiles(directory, query, {
			filenameOnly: true,
			caseInsensitive: true,
		});
	}

	/**
	 * Regex search with default limits.
	 */
	regexSearch(directory: string, pattern: string, maxResults = 1000): ISearchResult {
		return this._security.searchFiles(directory, pattern, {
			isRegex: true,
			maxResults,
		});
	}
}
