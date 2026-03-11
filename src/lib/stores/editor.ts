import { writable } from 'svelte/store';
import type { EditorState, OpenFile } from '$lib/types/store';

function createEditorStore() {
	const { subscribe, update, set } = writable<EditorState>({
		activeFile: null,
		openFiles: [],
		isDirty: false,
		content: '',
		language: 'plaintext'
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

				return {
					...s,
					activeFile: path,
					openFiles,
					isDirty: false,
					content,
					language
				};
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
						const next = openFiles[Math.min(idx, openFiles.length - 1)];
						activeFile = next.path;
						language = next.language;
						isDirty = next.isDirty;
						// Content must be loaded externally after switching;
						// we keep the previous content until then.
					}
				}

				return { ...s, activeFile, openFiles, content, language, isDirty };
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
		}
	};
}

export const editorStore = createEditorStore();
