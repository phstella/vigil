import { writable } from 'svelte/store';
import type { DirEntry, FilesState } from '$lib/types/store';

function createFilesStore() {
	const { subscribe, update, set } = writable<FilesState>({
		workspaceRoot: null,
		fileTree: [],
		expandedDirs: new Set<string>()
	});

	return {
		subscribe,
		set,

		/** Set the workspace root path and reset tree state. */
		setWorkspace(root: string) {
			update((s) => ({
				...s,
				workspaceRoot: root,
				fileTree: [],
				expandedDirs: new Set<string>()
			}));
		},

		/** Replace the top-level file tree entries. */
		updateTree(entries: DirEntry[]) {
			update((s) => ({ ...s, fileTree: entries }));
		},

		/** Toggle a directory's expanded state in the explorer. */
		toggleDir(path: string) {
			update((s) => {
				const next = new Set(s.expandedDirs);
				if (next.has(path)) {
					next.delete(path);
				} else {
					next.add(path);
				}
				return { ...s, expandedDirs: next };
			});
		}
	};
}

export const filesStore = createFilesStore();
