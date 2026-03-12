/**
 * Omnibar local UI state store.
 *
 * Manages the search query, live fuzzy-find results from the backend,
 * and keyboard selection index for the floating omnibar overlay.
 *
 * Supports two modes:
 * - **file**: fuzzy filename search via `fuzzy_find` IPC (Ctrl+P)
 * - **content**: phrase/snippet search via `search_content` IPC (Ctrl+Shift+F)
 *
 * Calls the appropriate IPC command with debouncing to meet performance budgets:
 * - File mode: <=80 ms first-result-render
 * - Content mode: <=150 ms median result render
 */

import { fuzzyFind, searchContent } from '$lib/ipc/search';
import { isVigilError } from '$lib/ipc/tauri';
import type { FuzzyMatch, ContentMatch } from '$lib/types/ipc';
import type { OmnibarMode } from '$lib/types/store';

/** Result from file (fuzzy) search mode. */
export interface OmnibarFileResult {
	/** Discriminant tag. */
	type: 'file';
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

/** Result from content (phrase/snippet) search mode. */
export interface OmnibarContentResult {
	/** Discriminant tag. */
	type: 'content';
	/** Unique identifier for the result item. */
	id: string;
	/** Workspace-relative file path. */
	path: string;
	/** File name extracted from path. */
	name: string;
	/** File extension without the leading dot, or null. */
	ext: string | null;
	/** 1-based line number of the match. */
	lineNumber: number;
	/** Start column of the match within the line. */
	lineStartCol: number;
	/** End column of the match within the line. */
	lineEndCol: number;
	/** Context line(s) around the match. */
	preview: string;
	/** Relevance score (higher is better). */
	score: number;
}

/** Union type for all omnibar results. */
export type OmnibarResult = OmnibarFileResult | OmnibarContentResult;

/** Debounce delay in milliseconds for file search IPC calls. */
const FILE_DEBOUNCE_MS = 80;

/** Debounce delay in milliseconds for content search IPC calls. */
const CONTENT_DEBOUNCE_MS = 150;

/** Maximum results to request from the backend. */
const MAX_RESULTS = 50;

/** Convert a FuzzyMatch from the backend into an OmnibarFileResult. */
function toFileResult(match: FuzzyMatch, index: number): OmnibarFileResult {
	const segments = match.path.split('/');
	const fileName = segments[segments.length - 1] ?? match.display;
	const dotIdx = fileName.lastIndexOf('.');
	const ext = dotIdx > 0 ? fileName.slice(dotIdx + 1).toLowerCase() : null;

	return {
		type: 'file',
		id: `file-${match.path}-${index}`,
		name: fileName,
		path: match.path,
		ext,
		score: match.score,
		matchedIndices: match.matched_indices,
		display: match.display,
		kind: match.kind
	};
}

/** Convert a ContentMatch from the backend into an OmnibarContentResult. */
function toContentResult(match: ContentMatch, index: number): OmnibarContentResult {
	const segments = match.path.split('/');
	const fileName = segments[segments.length - 1] ?? match.path;
	const dotIdx = fileName.lastIndexOf('.');
	const ext = dotIdx > 0 ? fileName.slice(dotIdx + 1).toLowerCase() : null;

	return {
		type: 'content',
		id: `content-${match.path}-${match.line_number}-${index}`,
		path: match.path,
		name: fileName,
		ext,
		lineNumber: match.line_number,
		lineStartCol: match.line_start_col,
		lineEndCol: match.line_end_col,
		preview: match.preview,
		score: match.score
	};
}

function createOmnibarStore() {
	let query = $state('');
	let mode = $state<OmnibarMode>('file');
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
	async function executeFileSearch(q: string, rid: number): Promise<void> {
		isLoading = true;
		error = null;

		try {
			const response = await fuzzyFind(q, MAX_RESULTS);
			// Discard stale responses if a newer query was issued.
			if (rid !== requestId) return;

			results = response.matches.map(toFileResult);
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

	/** Execute a content search query against the backend. */
	async function executeContentSearch(q: string, rid: number): Promise<void> {
		isLoading = true;
		error = null;

		try {
			const response = await searchContent(q, MAX_RESULTS);
			// Discard stale responses if a newer query was issued.
			if (rid !== requestId) return;

			results = response.matches.map(toContentResult);
			selectedIndex = 0;
		} catch (err: unknown) {
			// Discard stale error responses.
			if (rid !== requestId) return;

			if (isVigilError(err)) {
				if (err.code === 'INDEX_UNAVAILABLE' || err.code === 'WORKSPACE_NOT_OPEN') {
					error = null;
					results = [];
				} else {
					error = err.message;
				}
			} else {
				error = 'Content search failed';
			}
		} finally {
			if (rid === requestId) {
				isLoading = false;
			}
		}
	}

	/** Execute a search using the appropriate backend based on current mode. */
	function executeSearch(q: string, rid: number): void {
		if (mode === 'content') {
			void executeContentSearch(q, rid);
		} else {
			void executeFileSearch(q, rid);
		}
	}

	/** Get the appropriate debounce delay for the current mode. */
	function getDebounceMs(): number {
		return mode === 'content' ? CONTENT_DEBOUNCE_MS : FILE_DEBOUNCE_MS;
	}

	return {
		get query() {
			return query;
		},
		get mode(): OmnibarMode {
			return mode;
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
		 * Empty queries in file mode fetch recent files; in content mode, clear results.
		 */
		setQuery(value: string) {
			query = value;
			selectedIndex = 0;
			clearDebounce();

			if (value.trim() === '') {
				if (mode === 'file') {
					// For empty query in file mode, still call backend (returns recent files per spec).
					requestId++;
					const rid = requestId;
					void executeFileSearch('', rid);
				} else {
					// Content search requires a non-empty query.
					results = [];
					isLoading = false;
					error = null;
				}
				return;
			}

			// Debounce non-empty queries.
			requestId++;
			const rid = requestId;
			debounceTimer = setTimeout(() => {
				debounceTimer = null;
				executeSearch(value, rid);
			}, getDebounceMs());
		},

		/**
		 * Switch the omnibar search mode.
		 * Clears current results and re-runs the search with the current query.
		 */
		setMode(newMode: OmnibarMode) {
			if (mode === newMode) return;
			mode = newMode;
			clearDebounce();
			results = [];
			selectedIndex = 0;

			// Re-execute with current query in the new mode.
			if (query.trim() === '' && newMode === 'file') {
				requestId++;
				const rid = requestId;
				void executeFileSearch('', rid);
			} else if (query.trim() !== '') {
				requestId++;
				const rid = requestId;
				executeSearch(query, rid);
			}
		},

		/** Trigger an initial search when the omnibar opens. */
		initialize(initialMode: OmnibarMode = 'file') {
			mode = initialMode;
			if (initialMode === 'file') {
				requestId++;
				const rid = requestId;
				void executeFileSearch('', rid);
			}
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
			mode = 'file';
			selectedIndex = 0;
			results = [];
			isLoading = false;
			error = null;
		}
	};
}

export const omnibarStore = createOmnibarStore();
