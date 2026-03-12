/**
 * File-watcher event handler -- wires backend filesystem events to UI stores.
 *
 * Subscribes to:
 *   - vigil://index-updated   -> explorer tree invalidation, editor conflict detection
 *   - vigil://index-ready     -> status store initial counts, search readiness
 *   - vigil://git-hunks       -> git store hunk cache update
 *   - vigil://status-updated  -> status store refresh
 *   - vigil://fs-renamed      -> editor tab rename, explorer tree update, git hunk rename
 *
 * Call `initFileWatcher()` on mount and `teardownFileWatcher()` on destroy.
 */

import type { UnlistenFn } from '@tauri-apps/api/event';
import {
	onIndexUpdated,
	onIndexReady,
	onGitHunks,
	onStatusUpdated,
	onFsRenamed
} from '$lib/ipc/events';
import { filesStore } from '$lib/stores/files';
import { editorStore } from '$lib/stores/editor';
import { gitStore } from '$lib/stores/git';
import { statusStore } from '$lib/features/status/status-store';
import { explorerStore } from '$lib/features/explorer/explorer-store';
import type {
	IndexUpdatedPayload,
	IndexReadyPayload,
	GitHunksPayload,
	StatusUpdatedPayload,
	FsRenamedPayload
} from '$lib/types/ipc';

/** Accumulated unlisten functions for teardown. */
let unlisteners: UnlistenFn[] = [];

// ---------------------------------------------------------------------------
// Event handlers
// ---------------------------------------------------------------------------

/**
 * Handle index-updated: invalidate explorer cache and flag conflicts in editor.
 *
 * For each changed path:
 *   - "created"  -> parent directory needs refresh
 *   - "changed"  -> parent directory needs refresh, open editor gets conflict flag
 *   - "deleted"  -> remove from tree, close if open in editor, invalidate git hunks
 */
function handleIndexUpdated(payload: IndexUpdatedPayload): void {
	const refreshedDirs = new Set<string>();

	for (const change of payload.changes) {
		const parentDir = change.path.includes('/')
			? change.path.substring(0, change.path.lastIndexOf('/'))
			: '';

		switch (change.change_type) {
			case 'created':
				// Parent directory needs to re-fetch its children
				if (!refreshedDirs.has(parentDir)) {
					filesStore.markDirDirty(parentDir);
					refreshedDirs.add(parentDir);
				}
				// Bridge to visible explorer tree
				void explorerStore.handleIndexChange(change.path, 'created');
				break;

			case 'changed':
				// Flag open file as conflicted (changed on disk)
				editorStore.markConflict(change.path);
				// Parent may need refresh too (e.g. size/date changed)
				if (!refreshedDirs.has(parentDir)) {
					filesStore.markDirDirty(parentDir);
					refreshedDirs.add(parentDir);
				}
				// Bridge to visible explorer tree
				void explorerStore.handleIndexChange(change.path, 'changed');
				break;

			case 'deleted':
				filesStore.invalidatePath(change.path);
				editorStore.handleDelete(change.path);
				gitStore.invalidateHunks(change.path);
				if (!refreshedDirs.has(parentDir)) {
					filesStore.markDirDirty(parentDir);
					refreshedDirs.add(parentDir);
				}
				// Bridge to visible explorer tree
				void explorerStore.handleIndexChange(change.path, 'deleted');
				break;
		}
	}
}

/**
 * Handle index-ready: update status store with initial scan results.
 */
function handleIndexReady(payload: IndexReadyPayload): void {
	statusStore.patch({
		files_count: payload.files_count,
		notes_count: payload.notes_count,
		last_index_update_ms: payload.timestamp_ms
	});
}

/**
 * Handle git-hunks: update the git store hunk cache for the affected file.
 */
function handleGitHunks(payload: GitHunksPayload): void {
	gitStore.updateHunks(payload.path, payload.hunks);
}

/**
 * Handle status-updated: replace the full status snapshot in the store.
 */
function handleStatusUpdated(payload: StatusUpdatedPayload): void {
	statusStore.updateStatus(payload.status);
}

/**
 * Handle fs-renamed: update open editor tabs, explorer tree, and git hunk cache.
 */
function handleFsRenamed(payload: FsRenamedPayload): void {
	editorStore.handleRename(payload.old_path, payload.new_path);
	filesStore.handleRename(payload.old_path, payload.new_path);
	gitStore.handleRename(payload.old_path, payload.new_path);
	// Bridge to visible explorer tree
	explorerStore.handleRename(payload.old_path, payload.new_path);
}

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------

/**
 * Subscribe to all file-watcher events from the Rust backend.
 * Returns a promise that resolves when all subscriptions are active.
 */
export async function initFileWatcher(): Promise<void> {
	const listeners = await Promise.all([
		onIndexUpdated(handleIndexUpdated),
		onIndexReady(handleIndexReady),
		onGitHunks(handleGitHunks),
		onStatusUpdated(handleStatusUpdated),
		onFsRenamed(handleFsRenamed)
	]);

	unlisteners = listeners;
}

/**
 * Unsubscribe from all file-watcher events. Safe to call multiple times.
 */
export function teardownFileWatcher(): void {
	for (const unlisten of unlisteners) {
		unlisten();
	}
	unlisteners = [];
}
