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

	/**
	 * Save the current active tab's content into its openFiles entry
	 * so it can be restored when switching back.
	 */
	function cacheActiveContent(s: EditorState): OpenFile[] {
		if (!s.activeFile) return s.openFiles;
		return s.openFiles.map((f) =>
			f.path === s.activeFile ? { ...f, content: s.content, isDirty: s.isDirty } : f
		);
	}

	/**
	 * Restore state fields from a tab's cached data.
	 */
	function restoreFromTab(
		s: EditorState,
		tab: OpenFile
	): Partial<EditorState> {
		const base: Partial<EditorState> = {
			activeFile: tab.path,
			content: tab.content,
			language: tab.language,
			isDirty: tab.isDirty
		};

		if (isMarkdownFile(tab.path)) {
			return {
				...base,
				noteFile: tab.path,
				noteContent: tab.content
			};
		} else {
			return {
				...base,
				codeFile: tab.path,
				codeContent: tab.content,
				codeLanguage: tab.language
			};
		}
	}

	return {
		subscribe,
		set,

		/** Open a file (or switch to it if already open). */
		openFile(path: string, content: string, language: string) {
			update((s) => {
				// Cache current tab's content before switching
				const cachedFiles = cacheActiveContent(s);

				const exists = cachedFiles.some((f) => f.path === path);
				const openFiles: OpenFile[] = exists
					? cachedFiles.map((f) =>
							f.path === path ? { ...f, content, language } : f
						)
					: [...cachedFiles, { path, isDirty: false, language, content }];

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

		/**
		 * Activate an already-open tab by path, restoring its cached content.
		 * Returns false if the tab is not open. Does not trigger IPC -- content
		 * is restored from the in-memory cache built by openFile/updateContent.
		 */
		activateTab(path: string) {
			update((s) => {
				if (s.activeFile === path) return s;
				const tab = s.openFiles.find((f) => f.path === path);
				if (!tab) return s;

				// Cache current tab's content before switching
				const openFiles = cacheActiveContent(s);

				return {
					...s,
					openFiles,
					...restoreFromTab(s, tab)
				};
			});
		},

		/**
		 * Cycle to the next open tab. Wraps around at the end.
		 */
		nextTab() {
			update((s) => {
				if (s.openFiles.length <= 1) return s;
				const idx = s.openFiles.findIndex((f) => f.path === s.activeFile);
				const nextIdx = (idx + 1) % s.openFiles.length;
				const tab = s.openFiles[nextIdx];

				const openFiles = cacheActiveContent(s);
				return { ...s, openFiles, ...restoreFromTab(s, tab) };
			});
		},

		/**
		 * Cycle to the previous open tab. Wraps around at the beginning.
		 */
		prevTab() {
			update((s) => {
				if (s.openFiles.length <= 1) return s;
				const idx = s.openFiles.findIndex((f) => f.path === s.activeFile);
				const prevIdx = (idx - 1 + s.openFiles.length) % s.openFiles.length;
				const tab = s.openFiles[prevIdx];

				const openFiles = cacheActiveContent(s);
				return { ...s, openFiles, ...restoreFromTab(s, tab) };
			});
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
						// Activate the nearest remaining tab, preferring the one to the left
						const next = openFiles[Math.min(idx, openFiles.length - 1)];
						activeFile = next.path;
						content = next.content;
						language = next.language;
						isDirty = next.isDirty;
					}
				}

				// Clear the appropriate pane if its file was closed
				const result = { ...s, activeFile, openFiles, content, language, isDirty };
				if (isMarkdownFile(path) && s.noteFile === path) {
					result.noteFile = activeFile && isMarkdownFile(activeFile) ? activeFile : null;
					result.noteContent =
						activeFile && isMarkdownFile(activeFile) ? content : '';
				}
				if (!isMarkdownFile(path) && s.codeFile === path) {
					result.codeFile = activeFile && !isMarkdownFile(activeFile) ? activeFile : null;
					result.codeContent =
						activeFile && !isMarkdownFile(activeFile) ? content : '';
					result.codeLanguage =
						activeFile && !isMarkdownFile(activeFile) ? language : 'plaintext';
				}

				// If the new active file is a note, set note pane identity
				if (activeFile && isMarkdownFile(activeFile)) {
					result.noteFile = activeFile;
					result.noteContent = content;
				}
				// If the new active file is a code file, set code pane identity
				if (activeFile && !isMarkdownFile(activeFile)) {
					result.codeFile = activeFile;
					result.codeContent = content;
					result.codeLanguage = language;
				}

				return result;
			});
		},

		/**
		 * Close the currently active tab.
		 * Convenience wrapper around closeFile for keyboard shortcut use.
		 */
		closeActiveTab() {
			update((s) => {
				if (!s.activeFile) return s;
				// Re-use the closeFile logic inline to avoid calling update twice
				return s;
			});
			// Read current active file and delegate
			let currentActive: string | null = null;
			const unsub = subscribe((s) => {
				currentActive = s.activeFile;
			});
			unsub();
			if (currentActive) {
				this.closeFile(currentActive);
			}
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

		/** Replace the active file's content buffer and update tab cache. */
		updateContent(content: string) {
			update((s) => {
				const openFiles = s.openFiles.map((f) =>
					f.path === s.activeFile ? { ...f, content } : f
				);
				return { ...s, content, openFiles };
			});
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
					? s.openFiles.map((f) =>
							f.path === path ? { ...f, content } : f
						)
					: [...s.openFiles, { path, isDirty: false, language: 'markdown', content }];

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
						content = next.content;
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
