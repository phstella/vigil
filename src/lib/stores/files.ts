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
		},

		/**
		 * Invalidate cached entries for a given path.
		 * Removes matching entries from fileTree (parent dir must be re-fetched).
		 * Called when the backend emits index-updated with deleted/changed paths.
		 */
		invalidatePath(path: string) {
			update((s) => ({
				...s,
				fileTree: s.fileTree.filter((e) => e.path !== path)
			}));
		},

		/**
		 * Mark a directory as needing refresh by collapsing and re-expanding it.
		 * This signals the explorer to re-fetch its contents.
		 */
		markDirDirty(dirPath: string) {
			update((s) => {
				const next = new Set(s.expandedDirs);
				// Force removal so next expand triggers a fresh list_dir
				next.delete(dirPath);
				return { ...s, expandedDirs: next };
			});
		},

		/**
		 * Handle a file rename: update any tree entries whose path matches
		 * the old path to use the new path.
		 */
		handleRename(oldPath: string, newPath: string) {
			update((s) => ({
				...s,
				fileTree: s.fileTree.map((e) => {
					if (e.path === oldPath) {
						const name = newPath.split('/').pop() ?? e.name;
						const dot = name.lastIndexOf('.');
						const ext = dot >= 0 ? name.slice(dot + 1) : null;
						return { ...e, path: newPath, name, ext };
					}
					return e;
				})
			}));
		}
	};
}

export const filesStore = createFilesStore();
