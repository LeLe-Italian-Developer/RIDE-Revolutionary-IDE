/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

/**
 * Rust-backed diff service — thin TypeScript bridge to diff.rs native module.
 *
 * Provides high-performance Myers algorithm diffing at the line, word, and
 * character level, plus unified diff generation and similarity computation.
 * All logic is implemented in Rust; this file contains zero business logic.
 */

import { IRideSecurityService, IDiffResult } from '../common/rideSecurityService.js';

/**
 * Convenience wrapper around `IRideSecurityService` for diff operations.
 * Uses the shared native module loader — no separate binary loading needed.
 */
export class RideDiffService {

	constructor(private readonly _security: IRideSecurityService) { }

	/**
	 * Compute a line-level diff between two texts.
	 * Returns hunks with additions/deletions counts.
	 */
	computeDiff(original: string, modified: string): IDiffResult {
		return this._security.computeDiff(original, modified);
	}

	/**
	 * Generate a unified diff string (like `git diff` output).
	 */
	unifiedDiff(
		original: string,
		modified: string,
		originalLabel = 'original',
		modifiedLabel = 'modified',
	): string {
		return this._security.unifiedDiff(original, modified, originalLabel, modifiedLabel);
	}

	/**
	 * Compute similarity ratio between two texts (0.0 = completely different, 1.0 = identical).
	 */
	similarityRatio(textA: string, textB: string): number {
		return this._security.similarityRatio(textA, textB);
	}

	/**
	 * Quick check: are two texts identical?
	 */
	areIdentical(textA: string, textB: string): boolean {
		return this._security.similarityRatio(textA, textB) === 1.0;
	}

	/**
	 * Get a summary string of what changed.
	 */
	diffSummary(original: string, modified: string): string {
		const result = this._security.computeDiff(original, modified);
		if (result.isIdentical) {
			return 'No changes';
		}
		return `${result.additions} addition(s), ${result.deletions} deletion(s) across ${result.hunks.length} hunk(s)`;
	}
}
