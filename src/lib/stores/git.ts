import { writable } from 'svelte/store';
import type { GitHunk, GitState, GitStatusEntry, SyncState } from '$lib/types/store';

function createGitStore() {
	const { subscribe, update, set } = writable<GitState>({
		branch: null,
		syncState: 'unknown',
		hunks: new Map<string, GitHunk[]>(),
		statusEntries: []
	});

	return {
		subscribe,
		set,

		/** Set the current branch name and optional sync state. */
		setBranch(branch: string | null, syncState?: SyncState) {
			update((s) => ({
				...s,
				branch,
				syncState: syncState ?? s.syncState
			}));
		},

		/** Replace the hunk list for a specific file. */
		updateHunks(path: string, fileHunks: GitHunk[]) {
			update((s) => {
				const next = new Map(s.hunks);
				next.set(path, fileHunks);
				return { ...s, hunks: next };
			});
		},

		/** Replace the full list of status entries. */
		updateStatus(entries: GitStatusEntry[]) {
			update((s) => ({ ...s, statusEntries: entries }));
		},

		/** Remove cached hunks for a specific file (e.g. on deletion). */
		invalidateHunks(path: string) {
			update((s) => {
				const next = new Map(s.hunks);
				next.delete(path);
				return { ...s, hunks: next };
			});
		},

		/** Handle a file rename: move cached hunks from old to new path. */
		handleRename(oldPath: string, newPath: string) {
			update((s) => {
				const next = new Map(s.hunks);
				const oldHunks = next.get(oldPath);
				if (oldHunks) {
					next.delete(oldPath);
					next.set(newPath, oldHunks);
				}
				return { ...s, hunks: next };
			});
		}
	};
}

export const gitStore = createGitStore();
