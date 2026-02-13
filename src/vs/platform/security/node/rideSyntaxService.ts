/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

/**
 * Rust-backed syntax service — thin TypeScript bridge to syntax.rs native module.
 *
 * Provides bracket matching, indentation detection, line ending normalization,
 * and text statistics. All text analysis runs in native Rust for performance
 * on large files. This file contains zero business logic.
 */

import {
	IRideSecurityService,
	IBracketMatch,
	IIndentationInfo,
	ITextStats,
} from '../common/rideSecurityService.js';

/**
 * Convenience wrapper around `IRideSecurityService` for syntax analysis.
 * Uses the shared native module loader — no separate binary loading needed.
 */
export class RideSyntaxService {

	constructor(private readonly _security: IRideSecurityService) { }

	/**
	 * Find all bracket pairs in the text with depth information.
	 */
	matchBrackets(text: string): IBracketMatch[] {
		return this._security.matchBrackets(text);
	}

	/**
	 * Detect whether text uses tabs or spaces, and the tab size.
	 */
	detectIndentation(text: string): IIndentationInfo {
		return this._security.detectIndentation(text);
	}

	/**
	 * Normalize line endings to the specified style.
	 * @param style - 'lf', 'crlf', or 'cr' (default: 'lf')
	 */
	normalizeLineEndings(text: string, style: 'lf' | 'crlf' | 'cr' = 'lf'): string {
		return this._security.normalizeLineEndings(text, style);
	}

	/**
	 * Analyze text and return statistics (lines, words, characters, etc.).
	 */
	analyzeText(text: string): ITextStats {
		return this._security.analyzeText(text);
	}

	/**
	 * Get just the line count of text (fast path).
	 */
	countLines(text: string): number {
		return this._security.analyzeText(text).lines;
	}

	/**
	 * Check if the text has consistent indentation.
	 */
	hasConsistentIndentation(text: string): boolean {
		const info = this._security.detectIndentation(text);
		return info.confidence > 0.8;
	}

	/**
	 * Get a human-readable indentation description.
	 */
	describeIndentation(text: string): string {
		const info = this._security.detectIndentation(text);
		if (info.useTabs) {
			return 'Tabs';
		}
		return `${info.tabSize} spaces`;
	}
}
