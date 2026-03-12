/**
 * Note editor UI state store with IPC-backed open/save/autosave lifecycle.
 *
 * Tracks the active markdown file path, content buffer, dirty state, etag
 * for optimistic concurrency, an autosave timer that debounces writes
 * after the user stops typing, and view mode (edit vs preview toggle via Ctrl+.).
 */

import { readFile, writeFile } from '$lib/ipc/files';
import { isVigilError } from '$lib/ipc/tauri';

/** Autosave debounce delay in milliseconds. */
const AUTOSAVE_DELAY_MS = 2000;

/** View mode for the markdown editor. */
export type NoteViewMode = 'edit' | 'preview';

export interface NoteEditorState {
	/** Workspace-relative path of the active note, or null. */
	filePath: string | null;
	/** Raw markdown content of the active note. */
	content: string;
	/** Whether the note has unsaved modifications. */
	isDirty: boolean;
	/** Whether a save operation is currently in progress. */
	isSaving: boolean;
	/** Last error message from a save attempt, or null. */
	lastError: string | null;
	/** Current view mode: 'edit' for raw markdown, 'preview' for rendered. */
	viewMode: NoteViewMode;
}

function createNoteStore() {
	let filePath = $state<string | null>(null);
	let content = $state('');
	let isDirty = $state(false);
	let isSaving = $state(false);
	let lastError = $state<string | null>(null);
	let viewMode = $state<NoteViewMode>('edit');

	/** Content hash (etag) from the last read or successful write. */
	let etag: string | null = null;

	/** Handle for the autosave debounce timer. */
	let autosaveTimer: ReturnType<typeof setTimeout> | null = null;

	/** Clear any pending autosave timer. */
	function clearAutosave() {
		if (autosaveTimer !== null) {
			clearTimeout(autosaveTimer);
			autosaveTimer = null;
		}
	}

	/** Schedule an autosave after the debounce delay. */
	function scheduleAutosave() {
		clearAutosave();
		autosaveTimer = setTimeout(() => {
			autosaveTimer = null;
			// Only autosave if dirty and not already saving.
			if (isDirty && !isSaving && filePath) {
				void performSave();
			}
		}, AUTOSAVE_DELAY_MS);
	}

	/** Internal save implementation that writes to the backend. */
	async function performSave(): Promise<boolean> {
		if (!filePath || !isDirty) return true;
		if (isSaving) return false;

		isSaving = true;
		lastError = null;

		try {
			const response = await writeFile(filePath, content, etag);
			etag = response.etag;
			isDirty = false;
			isSaving = false;
			return true;
		} catch (err: unknown) {
			isSaving = false;
			if (isVigilError(err)) {
				lastError = `Save failed: ${err.message} (${err.code})`;
			} else {
				lastError = 'Save failed: unknown error';
			}
			return false;
		}
	}

	return {
		get filePath() {
			return filePath;
		},
		get content() {
			return content;
		},
		get isDirty() {
			return isDirty;
		},
		get isSaving() {
			return isSaving;
		},
		get lastError() {
			return lastError;
		},
		get viewMode() {
			return viewMode;
		},

		/**
		 * Open a markdown file by reading it from the backend.
		 * Replaces the current buffer entirely.
		 */
		async open(path: string): Promise<boolean> {
			// If switching files while dirty, attempt to save the current one first.
			if (filePath && isDirty && filePath !== path) {
				clearAutosave();
				await performSave();
			}

			clearAutosave();
			lastError = null;

			try {
				const response = await readFile(path);
				filePath = path;
				content = response.content;
				etag = response.etag;
				isDirty = false;
				isSaving = false;
				return true;
			} catch (err: unknown) {
				if (isVigilError(err)) {
					lastError = `Open failed: ${err.message} (${err.code})`;
				} else {
					lastError = 'Open failed: unknown error';
				}
				return false;
			}
		},

		/**
		 * Load content directly (for when content is already available,
		 * e.g., from the editor store or initial props).
		 */
		load(path: string, text: string, fileEtag?: string) {
			clearAutosave();
			filePath = path;
			content = text;
			etag = fileEtag ?? null;
			isDirty = false;
			isSaving = false;
			lastError = null;
		},

		/** Update the content buffer, mark dirty, and schedule autosave. */
		updateContent(text: string) {
			content = text;
			isDirty = true;
			lastError = null;
			scheduleAutosave();
		},

		/** Explicitly save the current buffer to disk via IPC. */
		async save(): Promise<boolean> {
			clearAutosave();
			return performSave();
		},

		/** Clear dirty state without writing (use with caution). */
		markClean() {
			isDirty = false;
			clearAutosave();
		},

		/** Toggle between edit and preview view modes. */
		toggleViewMode() {
			viewMode = viewMode === 'edit' ? 'preview' : 'edit';
		},

		/** Set the view mode explicitly. */
		setViewMode(mode: NoteViewMode) {
			viewMode = mode;
		},

		/** Reset the store to its initial empty state. */
		reset() {
			clearAutosave();
			filePath = null;
			content = '';
			etag = null;
			isDirty = false;
			isSaving = false;
			lastError = null;
			viewMode = 'edit';
		},

		/** Cancel any pending autosave (e.g., on component teardown). */
		cancelAutosave() {
			clearAutosave();
		}
	};
}

export const noteStore = createNoteStore();
