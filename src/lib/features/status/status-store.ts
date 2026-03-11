import { writable } from 'svelte/store';
import type { WorkspaceStatus } from '$lib/types/ipc';

const MOCK_STATUS: WorkspaceStatus = {
	branch: 'main',
	sync_state: 'synced',
	notes_count: 42,
	tags_count: 15,
	files_count: 128,
	version: '0.0.1',
	last_index_update_ms: Date.now()
};

function createStatusStore() {
	const { subscribe, set, update } = writable<WorkspaceStatus>(MOCK_STATUS);

	return {
		subscribe,

		/** Replace status wholesale (for future IPC wiring). */
		updateStatus(status: WorkspaceStatus) {
			set(status);
		},

		/** Patch individual fields without replacing the entire status. */
		patch(partial: Partial<WorkspaceStatus>) {
			update((s) => ({ ...s, ...partial }));
		}
	};
}

export const statusStore = createStatusStore();
