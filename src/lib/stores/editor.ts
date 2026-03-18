import { writable } from 'svelte/store';
import type { EditorState, OpenFile } from '$lib/types/store';
import { isMarkdownFile } from '$lib/utils/file-routing';

function createEditorStore() {
	const { subscribe, update, set } = writable<EditorState>({
		activeFile: null,
		openFiles: [],
		isDirty: false,
		content: '',
		language: 'plaintext',
		noteFile: null,
		noteContent: '',
		codeFile: null,
		codeContent: '',
		codeLanguage: 'plaintext',
		conflictFiles: new Set<string>()
	});

	return {
		subscribe,
		set,

		/** Open a file (or switch to it if already open). */
		openFile(path: string, content: string, language: string) {
			update((s) => {
				const exists = s.openFiles.some((f) => f.path === path);
				const openFiles: OpenFile[] = exists
					? s.openFiles
					: [...s.openFiles, { path, isDirty: false, language }];

				const base = {
					...s,
					activeFile: path,
					openFiles,
					isDirty: false,
					content,
					language
				};

				// Route to the correct pane based on file type
				if (isMarkdownFile(path)) {
					return {
						...base,
						noteFile: path,
						noteContent: content
					};
				} else {
					return {
						...base,
						codeFile: path,
						codeContent: content,
						codeLanguage: language
					};
				}
			});
		},

		/**
		 * Open a file with automatic pane routing.
		 * .md files go to the center note pane; code files go to the right code pane.
		 * This is the primary entry point for file-type routing (task 3.4).
		 */
		openFileRouted(path: string, content: string, language: string) {
			this.openFile(path, content, language);
		},

		/** Close a file tab. If it was active, activate an adjacent tab or clear. */
		closeFile(path: string) {
			update((s) => {
				const idx = s.openFiles.findIndex((f) => f.path === path);
				if (idx === -1) return s;

				const openFiles = s.openFiles.filter((f) => f.path !== path);
				let activeFile = s.activeFile;
				let content = s.content;
				let language = s.language;
				let isDirty = s.isDirty;

				if (s.activeFile === path) {
					if (openFiles.length === 0) {
						activeFile = null;
						content = '';
						language = 'plaintext';
						isDirty = false;
					} else {
						const next = openFiles[Math.min(idx, openFiles.length - 1)];
						activeFile = next.path;
						language = next.language;
						isDirty = next.isDirty;
						// Content must be loaded externally after switching;
						// we keep the previous content until then.
					}
				}

				// Clear the appropriate pane if its file was closed
				const result = { ...s, activeFile, openFiles, content, language, isDirty };
				if (isMarkdownFile(path) && s.noteFile === path) {
					result.noteFile = null;
					result.noteContent = '';
				}
				if (!isMarkdownFile(path) && s.codeFile === path) {
					result.codeFile = null;
					result.codeContent = '';
					result.codeLanguage = 'plaintext';
				}

				return result;
			});
		},

		/** Mark the active file as dirty or clean. */
		setDirty(dirty: boolean) {
			update((s) => {
				const openFiles = s.openFiles.map((f) =>
					f.path === s.activeFile ? { ...f, isDirty: dirty } : f
				);
				return { ...s, isDirty: dirty, openFiles };
			});
		},

		/** Replace the active file's content buffer. */
		updateContent(content: string) {
			update((s) => ({ ...s, content }));
		},

		/**
		 * Mark a file as changed-on-disk (conflict).
		 * Only flags it if the file is currently open.
		 */
		markConflict(path: string) {
			update((s) => {
				const isOpen = s.openFiles.some((f) => f.path === path);
				if (!isOpen) return s;
				const next = new Set(s.conflictFiles);
				next.add(path);
				return { ...s, conflictFiles: next };
			});
		},

		/** Clear the conflict flag for a file (e.g. after user reloads). */
		clearConflict(path: string) {
			update((s) => {
				const next = new Set(s.conflictFiles);
				next.delete(path);
				return { ...s, conflictFiles: next };
			});
		},

		/**
		 * Realign note-pane identity to the note actually loaded in noteStore.
		 * Used when a note switch is blocked (e.g., save/open failure) so UI
		 * path and persistence target stay consistent.
		 */
		syncNoteIdentity(path: string, content: string) {
			update((s) => {
				const exists = s.openFiles.some((f) => f.path === path);
				const openFiles: OpenFile[] = exists
					? s.openFiles
					: [...s.openFiles, { path, isDirty: false, language: 'markdown' }];

				return {
					...s,
					activeFile: path,
					openFiles,
					content,
					language: 'markdown',
					noteFile: path,
					noteContent: content
				};
			});
		},

		/**
		 * Handle a file rename for open editor tabs.
		 * Updates the path in openFiles and activeFile if affected.
		 */
		handleRename(oldPath: string, newPath: string) {
			update((s) => {
				const wasActive = s.activeFile === oldPath;
				const openFiles = s.openFiles.map((f) =>
					f.path === oldPath ? { ...f, path: newPath } : f
				);
				const conflictFiles = new Set(s.conflictFiles);
				if (conflictFiles.has(oldPath)) {
					conflictFiles.delete(oldPath);
					conflictFiles.add(newPath);
				}
				return {
					...s,
					activeFile: wasActive ? newPath : s.activeFile,
					openFiles,
					conflictFiles
				};
			});
		},

		/**
		 * Handle a file deletion: close the tab if the file was open.
		 */
		handleDelete(path: string) {
			update((s) => {
				const isOpen = s.openFiles.some((f) => f.path === path);
				if (!isOpen) return s;
				// Delegate to closeFile logic
				const idx = s.openFiles.findIndex((f) => f.path === path);
				if (idx === -1) return s;

				const openFiles = s.openFiles.filter((f) => f.path !== path);
				let activeFile = s.activeFile;
				let content = s.content;
				let language = s.language;
				let isDirty = s.isDirty;
				const conflictFiles = new Set(s.conflictFiles);
				conflictFiles.delete(path);

				if (s.activeFile === path) {
					if (openFiles.length === 0) {
						activeFile = null;
						content = '';
						language = 'plaintext';
						isDirty = false;
					} else {
						const next = openFiles[Math.min(idx, openFiles.length - 1)];
						activeFile = next.path;
						language = next.language;
						isDirty = next.isDirty;
					}
				}

				return { ...s, activeFile, openFiles, content, language, isDirty, conflictFiles };
			});
		}
	};
}

export const editorStore = createEditorStore();
