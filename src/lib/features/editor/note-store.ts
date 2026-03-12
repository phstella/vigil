/**
 * Note editor UI state store.
 *
 * Provides local state for the NoteEditor skeleton component.
 * Tracks the active markdown file path, content buffer, dirty state,
 * and view mode (edit vs preview toggle via Ctrl+.).
 */

/** View mode for the markdown editor. */
export type NoteViewMode = 'edit' | 'preview';

export interface NoteEditorState {
	/** Workspace-relative path of the active note, or null. */
	filePath: string | null;
	/** Raw markdown content of the active note. */
	content: string;
	/** Whether the note has unsaved modifications. */
	isDirty: boolean;
	/** Current view mode: 'edit' for raw markdown, 'preview' for rendered. */
	viewMode: NoteViewMode;
}

function createNoteStore() {
	let filePath = $state<string | null>(null);
	let content = $state('');
	let isDirty = $state(false);
	let viewMode = $state<NoteViewMode>('edit');

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
		get viewMode() {
			return viewMode;
		},

		/** Load a markdown file into the note editor. */
		load(path: string, text: string) {
			filePath = path;
			content = text;
			isDirty = false;
			// Preserve current view mode across file loads
		},

		/** Update the content buffer and mark as dirty. */
		updateContent(text: string) {
			content = text;
			isDirty = true;
		},

		/** Clear dirty state (e.g. after save). */
		markClean() {
			isDirty = false;
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
			filePath = null;
			content = '';
			isDirty = false;
			viewMode = 'edit';
		}
	};
}

export const noteStore = createNoteStore();
