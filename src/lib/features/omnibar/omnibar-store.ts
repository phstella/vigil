/**
 * Omnibar local UI state store.
 *
 * Manages the search query, live fuzzy-find results from the backend,
 * and keyboard selection index for the floating omnibar overlay.
 *
 * Calls the `fuzzy_find` IPC command with debouncing to meet the
 * <=80 ms first-result-render performance budget.
 */

import { fuzzyFind } from '$lib/ipc/search';
import { isVigilError } from '$lib/ipc/tauri';
import type { FuzzyMatch } from '$lib/types/ipc';

export interface OmnibarResult {
	/** Unique identifier for the result item. */
	id: string;
	/** Display name (file name). */
	name: string;
	/** Workspace-relative path. */
	path: string;
	/** File extension without the leading dot, or null. */
	ext: string | null;
	/** Match score from the fuzzy finder (higher is better). */
	score: number;
	/** Character positions in `display` that matched the query. */
	matchedIndices: number[];
	/** Formatted display string from the backend. */
	display: string;
	/** Entry type: file or directory. */
	kind: 'file' | 'dir';
}

/** Debounce delay in milliseconds for IPC calls. */
const DEBOUNCE_MS = 80;

/** Maximum results to request from the backend. */
const MAX_RESULTS = 50;

/** Convert a FuzzyMatch from the backend into an OmnibarResult. */
function toOmnibarResult(match: FuzzyMatch, index: number): OmnibarResult {
	const segments = match.path.split('/');
	const fileName = segments[segments.length - 1] ?? match.display;
	const dotIdx = fileName.lastIndexOf('.');
	const ext = dotIdx > 0 ? fileName.slice(dotIdx + 1).toLowerCase() : null;

	return {
		id: `${match.path}-${index}`,
		name: fileName,
		path: match.path,
		ext,
		score: match.score,
		matchedIndices: match.matched_indices,
		display: match.display,
		kind: match.kind
	};
}

function createOmnibarStore() {
	let query = $state('');
	let selectedIndex = $state(0);
	let results = $state<OmnibarResult[]>([]);
	let isLoading = $state(false);
	let error = $state<string | null>(null);

	/** Handle for the debounce timer. */
	let debounceTimer: ReturnType<typeof setTimeout> | null = null;

	/** Monotonically increasing request ID to discard stale responses. */
	let requestId = 0;

	/** Clear any pending debounce timer. */
	function clearDebounce() {
		if (debounceTimer !== null) {
			clearTimeout(debounceTimer);
			debounceTimer = null;
		}
	}

	/** Execute a fuzzy find query against the backend. */
	async function executeSearch(q: string, rid: number): Promise<void> {
		isLoading = true;
		error = null;

		try {
			const response = await fuzzyFind(q, MAX_RESULTS);
			// Discard stale responses if a newer query was issued.
			if (rid !== requestId) return;

			results = response.matches.map(toOmnibarResult);
			selectedIndex = 0;
		} catch (err: unknown) {
			// Discard stale error responses.
			if (rid !== requestId) return;

			if (isVigilError(err)) {
				// INDEX_UNAVAILABLE is expected when workspace is still scanning.
				if (err.code === 'INDEX_UNAVAILABLE' || err.code === 'WORKSPACE_NOT_OPEN') {
					error = null;
					results = [];
				} else {
					error = err.message;
				}
			} else {
				error = 'Search failed';
			}
		} finally {
			if (rid === requestId) {
				isLoading = false;
			}
		}
	}

	return {
		get query() {
			return query;
		},
		get selectedIndex() {
			return selectedIndex;
		},
		get results(): OmnibarResult[] {
			return results;
		},
		get isLoading() {
			return isLoading;
		},
		get error() {
			return error;
		},

		/**
		 * Update the search query and trigger a debounced IPC call.
		 * Empty queries clear the results immediately.
		 */
		setQuery(value: string) {
			query = value;
			selectedIndex = 0;
			clearDebounce();

			if (value.trim() === '') {
				// For empty query, still call backend (returns recent files per spec).
				requestId++;
				const rid = requestId;
				void executeSearch('', rid);
				return;
			}

			// Debounce non-empty queries.
			requestId++;
			const rid = requestId;
			debounceTimer = setTimeout(() => {
				debounceTimer = null;
				void executeSearch(value, rid);
			}, DEBOUNCE_MS);
		},

		/** Trigger an initial search when the omnibar opens (empty query = recent files). */
		initialize() {
			requestId++;
			const rid = requestId;
			void executeSearch('', rid);
		},

		/** Move selection to the next result, wrapping at the end. */
		selectNext() {
			if (results.length === 0) return;
			selectedIndex = (selectedIndex + 1) % results.length;
		},

		/** Move selection to the previous result, wrapping at the start. */
		selectPrev() {
			if (results.length === 0) return;
			selectedIndex = (selectedIndex - 1 + results.length) % results.length;
		},

		/** Return the currently selected result, or null if the list is empty. */
		selectCurrent(): OmnibarResult | null {
			if (results.length === 0) return null;
			return results[selectedIndex] ?? null;
		},

		/** Reset the store to its initial state. */
		reset() {
			clearDebounce();
			requestId++;
			query = '';
			selectedIndex = 0;
			results = [];
			isLoading = false;
			error = null;
		}
	};
}

export const omnibarStore = createOmnibarStore();
