import type { OmnibarMode } from '$lib/types/store';

export interface ParsedOmnibarQuery {
	rawQuery: string;
	mode: OmnibarMode;
	query: string;
	hasCommandPrefix: boolean;
}

/** Parse the omnibar query and let a leading `>` force command mode. */
export function parseOmnibarQuery(rawQuery: string, fallbackMode: OmnibarMode): ParsedOmnibarQuery {
	const leftTrimmed = rawQuery.trimStart();
	if (leftTrimmed.startsWith('>')) {
		return {
			rawQuery,
			mode: 'command',
			query: leftTrimmed.slice(1).trimStart(),
			hasCommandPrefix: true
		};
	}

	return {
		rawQuery,
		mode: fallbackMode,
		query: rawQuery,
		hasCommandPrefix: false
	};
}

/** Normalize the visible input when switching into command mode. */
export function ensureCommandQuery(rawQuery: string): string {
	if (rawQuery.trimStart().startsWith('>')) return rawQuery;
	const trimmed = rawQuery.trim();
	return trimmed ? `> ${trimmed}` : '>';
}
