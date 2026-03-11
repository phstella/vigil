/**
 * Note editor UI state store.
 *
 * Provides local state for the NoteEditor skeleton component.
 * Tracks the active markdown file path, content buffer, and dirty state.
 * Will be extended with Tiptap integration in a future epic.
 */

export interface NoteEditorState {
	/** Workspace-relative path of the active note, or null. */
	filePath: string | null;
	/** Raw markdown content of the active note. */
	content: string;
	/** Whether the note has unsaved modifications. */
	isDirty: boolean;
}

function createNoteStore() {
	let filePath = $state<string | null>(null);
	let content = $state('');
	let isDirty = $state(false);

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

		/** Load a markdown file into the note editor. */
		load(path: string, text: string) {
			filePath = path;
			content = text;
			isDirty = false;
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

		/** Reset the store to its initial empty state. */
		reset() {
			filePath = null;
			content = '';
			isDirty = false;
		}
	};
}

export const noteStore = createNoteStore();
