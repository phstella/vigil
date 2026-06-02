/**
 * Omnibar local UI state store.
 *
 * Manages the search query, live fuzzy-find results from the backend,
 * and keyboard selection index for the floating omnibar overlay.
 *
 * Supports three modes:
 * - **file**: fuzzy filename search via `fuzzy_find` IPC (Ctrl+P)
 * - **content**: phrase/snippet search via `search_content` IPC (Ctrl+Shift+F)
 * - **command**: local internal actions via `> command` or Ctrl+Shift+P
 *
 * Calls the appropriate IPC command with debouncing to meet performance budgets:
 * - File mode: <=80 ms first-result-render
 * - Content mode: <=150 ms median result render
 */

import { fuzzyFind, searchContent } from '$lib/ipc/search';
import { isVigilError } from '$lib/ipc/tauri';
import { ensureCommandQuery, parseOmnibarQuery } from './omnibar-parser';
import type { FuzzyMatch, ContentMatch } from '$lib/types/ipc';
import type { OmnibarMode } from '$lib/types/store';

/** Internal command definition supplied by the app shell. */
export interface OmnibarCommand {
	/** Stable command identifier. */
	id: string;
	/** Primary command label. */
	title: string;
	/** Secondary context shown under the title. */
	subtitle?: string;
	/** Extra searchable aliases. */
	keywords?: string[];
	/** Execute the command action. */
	run: () => void | Promise<void>;
}

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

/** Result from command mode. */
export interface OmnibarCommandResult {
	/** Discriminant tag. */
	type: 'command';
	/** Unique identifier for the result item. */
	id: string;
	/** Stable command identifier. */
	commandId: string;
	/** Primary command label. */
	title: string;
	/** Secondary context shown under the title. */
	subtitle: string;
	/** Extra searchable aliases. */
	keywords: string[];
	/** Local match score. */
	score: number;
	/** Execute the command action. */
	run: () => void | Promise<void>;
}

/** Union type for all omnibar results. */
export type OmnibarResult = OmnibarFileResult | OmnibarContentResult | OmnibarCommandResult;

/**
 * Debounce delay in milliseconds for file search IPC calls.
 * Leading edge fires immediately on first keystroke; subsequent keystrokes
 * within the window are debounced to avoid flooding the backend.
 */
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

/** Score a command against a user query. Higher is better; 0 means no match. */
function scoreCommand(command: OmnibarCommand, query: string): number {
	const normalizedQuery = query.trim().toLowerCase();
	if (normalizedQuery === '') return 1;

	const tokens = normalizedQuery.split(/\s+/).filter(Boolean);
	const title = command.title.toLowerCase();
	const subtitle = command.subtitle?.toLowerCase() ?? '';
	const keywords = command.keywords?.map((keyword) => keyword.toLowerCase()) ?? [];
	const haystack = [command.id.toLowerCase(), title, subtitle, ...keywords].join(' ');

	if (!tokens.every((token) => haystack.includes(token))) return 0;

	return tokens.reduce((score, token) => {
		if (title.startsWith(token)) return score + 100;
		if (title.includes(token)) return score + 70;
		if (keywords.some((keyword) => keyword.startsWith(token))) return score + 50;
		return score + 25;
	}, 0);
}

/** Convert an OmnibarCommand into an OmnibarCommandResult. */
function toCommandResult(
	command: OmnibarCommand,
	query: string,
	index: number
): OmnibarCommandResult | null {
	const score = scoreCommand(command, query);
	if (score === 0) return null;

	return {
		type: 'command',
		id: `command-${command.id}-${index}`,
		commandId: command.id,
		title: command.title,
		subtitle: command.subtitle ?? '',
		keywords: command.keywords ?? [],
		score,
		run: command.run
	};
}

function createOmnibarStore() {
	let query = $state('');
	let mode = $state<OmnibarMode>('file');
	let selectedIndex = $state(0);
	let results = $state<OmnibarResult[]>([]);
	let isLoading = $state(false);
	let error = $state<string | null>(null);
	let commands: OmnibarCommand[] = [];

	/** Handle for the debounce timer. */
	let debounceTimer: ReturnType<typeof setTimeout> | null = null;

	/** Monotonically increasing request ID to discard stale responses. */
	let requestId = 0;

	/**
	 * Tracks whether we've already fired a leading-edge search in the current
	 * debounce window. Reset to false when the debounce timer expires or is cleared.
	 */
	let leadingEdgeFired = false;

	/** Clear any pending debounce timer. */
	function clearDebounce() {
		if (debounceTimer !== null) {
			clearTimeout(debounceTimer);
			debounceTimer = null;
		}
		leadingEdgeFired = false;
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

	/** Execute a local command search. */
	function executeCommandSearch(q: string): void {
		isLoading = false;
		error = null;
		results = commands
			.map((command, index) => toCommandResult(command, q, index))
			.filter((result): result is OmnibarCommandResult => result !== null)
			.sort((a, b) => b.score - a.score || a.title.localeCompare(b.title));
		selectedIndex = 0;
	}

	/** Execute a search using the appropriate backend based on current mode. */
	function executeSearch(searchMode: OmnibarMode, q: string, rid: number): void {
		if (searchMode === 'command') {
			executeCommandSearch(q);
		} else if (searchMode === 'content') {
			void executeContentSearch(q, rid);
		} else {
			void executeFileSearch(q, rid);
		}
	}

	/** Get the appropriate debounce delay for the current mode. */
	function getDebounceMs(searchMode: OmnibarMode): number {
		return searchMode === 'content' ? CONTENT_DEBOUNCE_MS : FILE_DEBOUNCE_MS;
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

		/** Replace available internal commands. */
		setCommands(nextCommands: OmnibarCommand[]) {
			commands = nextCommands;
			if (mode === 'command') {
				const parsed = parseOmnibarQuery(query, mode);
				requestId++;
				executeCommandSearch(parsed.query);
			}
		},

		/**
		 * Update the search query and trigger a debounced IPC call.
		 * Empty queries in file mode fetch recent files; in content mode, clear results.
		 *
		 * Performance (Task 3.11): Uses leading-edge debounce for file mode --
		 * the first keystroke fires immediately so the user sees results within
		 * the 80ms budget, while subsequent rapid keystrokes are trailing-debounced.
		 */
		setQuery(value: string) {
			query = value;
			selectedIndex = 0;
			clearDebounce();

			const parsed = parseOmnibarQuery(value, mode);
			if (parsed.mode !== mode) {
				mode = parsed.mode;
			}

			if (parsed.mode === 'command') {
				requestId++;
				executeCommandSearch(parsed.query);
				return;
			}

			if (parsed.query.trim() === '') {
				if (parsed.mode === 'file') {
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

			requestId++;
			const rid = requestId;

			// Leading-edge: fire immediately on first keystroke in file mode
			// so results appear within the 80ms budget.
			if (parsed.mode === 'file' && !leadingEdgeFired) {
				leadingEdgeFired = true;
				executeSearch(parsed.mode, parsed.query, rid);
				// Set a trailing timer to catch the final query after rapid typing
				debounceTimer = setTimeout(() => {
					debounceTimer = null;
					leadingEdgeFired = false;
					// Only re-fire if the query changed since the leading call
					if (query !== value) {
						requestId++;
						const latest = parseOmnibarQuery(query, mode);
						executeSearch(latest.mode, latest.query, requestId);
					}
				}, getDebounceMs(parsed.mode));
				return;
			}

			// Trailing-edge debounce for subsequent keystrokes and content mode.
			debounceTimer = setTimeout(() => {
				debounceTimer = null;
				leadingEdgeFired = false;
				executeSearch(parsed.mode, parsed.query, rid);
			}, getDebounceMs(parsed.mode));
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

			if (newMode === 'command') {
				query = ensureCommandQuery(query);
				const parsed = parseOmnibarQuery(query, newMode);
				requestId++;
				executeCommandSearch(parsed.query);
				return;
			}

			const parsed = parseOmnibarQuery(query, newMode);
			if (parsed.hasCommandPrefix) {
				query = parsed.query;
			}

			// Re-execute with current query in the new mode.
			if (query.trim() === '' && newMode === 'file') {
				requestId++;
				const rid = requestId;
				void executeFileSearch('', rid);
			} else if (query.trim() !== '') {
				requestId++;
				const rid = requestId;
				executeSearch(newMode, query, rid);
			}
		},

		/** Trigger an initial search when the omnibar opens. */
		initialize(initialMode: OmnibarMode = 'file') {
			mode = initialMode;
			if (initialMode === 'command') {
				query = ensureCommandQuery(query);
				requestId++;
				executeCommandSearch(parseOmnibarQuery(query, initialMode).query);
			} else if (initialMode === 'file') {
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
