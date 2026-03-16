/**
 * Git gutter decoration logic for the Monaco code editor.
 *
 * Converts GitHunk[] from the backend into Monaco editor decorations
 * applied to the glyph margin. Provides a debounced refresh coordinator
 * that fetches hunks via IPC and updates decorations.
 *
 * Performance target: git hunk refresh <= 200 ms for files under 5k lines
 * (see docs/specs/editor-performance-budget.md).
 */

import type * as Monaco from 'monaco-editor';
import type { GitHunk } from '$lib/types/ipc';
import { getGitHunks } from '$lib/ipc/git';
import { gitStore } from '$lib/stores/git';

// ---------------------------------------------------------------------------
// CSS class names for gutter decoration styling
// ---------------------------------------------------------------------------

/** CSS class applied to glyph margin for added lines. */
export const GUTTER_ADDED_CLASS = 'vigil-gutter-added';

/** CSS class applied to glyph margin for modified lines. */
export const GUTTER_MODIFIED_CLASS = 'vigil-gutter-modified';

/** CSS class applied to glyph margin for deleted lines (single-line indicator). */
export const GUTTER_DELETED_CLASS = 'vigil-gutter-deleted';

// ---------------------------------------------------------------------------
// Decoration computation
// ---------------------------------------------------------------------------

/**
 * Convert an array of GitHunk objects into Monaco editor decoration options.
 *
 * Each hunk maps to one or more line decorations in the glyph margin:
 * - `added`: green bar on each added line
 * - `modified`: teal/blue bar on each modified line
 * - `deleted`: red triangle on the line where deletions occurred
 */
export function hunksToDecorations(hunks: GitHunk[]): Monaco.editor.IModelDeltaDecoration[] {
	const decorations: Monaco.editor.IModelDeltaDecoration[] = [];

	for (const hunk of hunks) {
		const glyphClass = hunkToGlyphClass(hunk.change_type);

		if (hunk.change_type === 'deleted') {
			// Deleted hunks show a single indicator at the start line
			// (the line in the working copy where content was removed)
			decorations.push({
				range: {
					startLineNumber: Math.max(1, hunk.start_line),
					startColumn: 1,
					endLineNumber: Math.max(1, hunk.start_line),
					endColumn: 1
				},
				options: {
					isWholeLine: true,
					glyphMarginClassName: glyphClass,
					glyphMarginHoverMessage: { value: 'Deleted lines' }
				}
			});
		} else {
			// Added or modified hunks span a range of lines
			decorations.push({
				range: {
					startLineNumber: hunk.start_line,
					startColumn: 1,
					endLineNumber: hunk.end_line,
					endColumn: 1
				},
				options: {
					isWholeLine: true,
					glyphMarginClassName: glyphClass,
					glyphMarginHoverMessage: {
						value: hunk.change_type === 'added' ? 'Added lines' : 'Modified lines'
					}
				}
			});
		}
	}

	return decorations;
}

/** Map a hunk change type to the corresponding glyph margin CSS class. */
function hunkToGlyphClass(changeType: GitHunk['change_type']): string {
	switch (changeType) {
		case 'added':
			return GUTTER_ADDED_CLASS;
		case 'modified':
			return GUTTER_MODIFIED_CLASS;
		case 'deleted':
			return GUTTER_DELETED_CLASS;
	}
}

// ---------------------------------------------------------------------------
// Gutter controller -- manages decorations and refresh lifecycle
// ---------------------------------------------------------------------------

/**
 * GutterController manages git gutter decorations for a single Monaco editor
 * instance. It handles fetching hunks, computing decorations, applying them,
 * and debouncing refresh requests.
 */
export class GutterController {
	private editor: Monaco.editor.IStandaloneCodeEditor;
	private decorationIds: string[] = [];
	private currentPath: string | null = null;
	private debounceTimer: ReturnType<typeof setTimeout> | null = null;
	private disposed = false;

	/** Debounce delay in ms for edit-triggered refreshes. */
	private static readonly DEBOUNCE_MS = 300;

	constructor(editor: Monaco.editor.IStandaloneCodeEditor) {
		this.editor = editor;
	}

	/**
	 * Set the active file path. Triggers an immediate hunk refresh.
	 * Call this when the file changes (file switch).
	 */
	setFilePath(path: string | null): void {
		this.currentPath = path;
		if (path) {
			this.refreshImmediate();
		} else {
			this.clearDecorations();
		}
	}

	/**
	 * Schedule a debounced refresh. Call this on content edits
	 * to avoid hammering the backend on every keystroke.
	 */
	scheduleRefresh(): void {
		if (this.disposed) return;
		this.cancelPending();
		this.debounceTimer = setTimeout(() => {
			this.refreshImmediate();
		}, GutterController.DEBOUNCE_MS);
	}

	/**
	 * Apply hunks directly (e.g., from a backend push event).
	 * Skips IPC call since hunks are already available.
	 */
	applyHunks(hunks: GitHunk[]): void {
		if (this.disposed) return;
		const decorations = hunksToDecorations(hunks);
		this.decorationIds = this.editor.deltaDecorations(this.decorationIds, decorations);
	}

	/**
	 * Fetch hunks from the backend and apply decorations immediately.
	 * Also updates the git store cache.
	 */
	async refreshImmediate(): Promise<void> {
		if (this.disposed || !this.currentPath) return;
		const path = this.currentPath;

		try {
			const response = await getGitHunks(path);
			// Guard against stale responses after file switch
			if (this.currentPath !== path || this.disposed) return;

			gitStore.updateHunks(path, response.hunks);
			this.applyHunks(response.hunks);
		} catch {
			// Silently ignore errors (e.g., GIT_UNAVAILABLE for non-git workspaces,
			// or FILE_NOT_FOUND for new unsaved files). Decorations are simply cleared.
			if (this.currentPath === path && !this.disposed) {
				this.clearDecorations();
			}
		}
	}

	/** Remove all gutter decorations from the editor. */
	clearDecorations(): void {
		if (this.decorationIds.length > 0) {
			this.decorationIds = this.editor.deltaDecorations(this.decorationIds, []);
		}
	}

	/** Cancel any pending debounced refresh. */
	private cancelPending(): void {
		if (this.debounceTimer !== null) {
			clearTimeout(this.debounceTimer);
			this.debounceTimer = null;
		}
	}

	/** Clean up resources. Must be called when the editor is destroyed. */
	dispose(): void {
		this.disposed = true;
		this.cancelPending();
		this.clearDecorations();
	}
}
