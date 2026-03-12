/**
 * Typed event listeners for all vigil:// event channels.
 *
 * Each function subscribes to a backend event and returns an unlisten
 * function that must be called to unsubscribe (e.g., in onDestroy).
 */

import { listenEvent } from './tauri';
import type { UnlistenFn } from '@tauri-apps/api/event';
import type {
	IndexUpdatedPayload,
	IndexReadyPayload,
	GitHunksPayload,
	StatusUpdatedPayload,
	FsRenamedPayload
} from '$lib/types/ipc';

/** Event channel names matching Rust `events/*.rs` constants. */
export const EVENTS = {
	INDEX_UPDATED: 'vigil://index-updated',
	INDEX_READY: 'vigil://index-ready',
	GIT_HUNKS: 'vigil://git-hunks',
	STATUS_UPDATED: 'vigil://status-updated',
	FS_RENAMED: 'vigil://fs-renamed'
} as const;

/** Subscribe to file index change notifications. */
export function onIndexUpdated(handler: (payload: IndexUpdatedPayload) => void): Promise<UnlistenFn> {
	return listenEvent<IndexUpdatedPayload>(EVENTS.INDEX_UPDATED, handler);
}

/** Subscribe to the one-time index ready notification after workspace open. */
export function onIndexReady(handler: (payload: IndexReadyPayload) => void): Promise<UnlistenFn> {
	return listenEvent<IndexReadyPayload>(EVENTS.INDEX_READY, handler);
}

/** Subscribe to git hunk change notifications for editor gutter updates. */
export function onGitHunks(handler: (payload: GitHunksPayload) => void): Promise<UnlistenFn> {
	return listenEvent<GitHunksPayload>(EVENTS.GIT_HUNKS, handler);
}

/** Subscribe to workspace status change notifications for the footer bar. */
export function onStatusUpdated(
	handler: (payload: StatusUpdatedPayload) => void
): Promise<UnlistenFn> {
	return listenEvent<StatusUpdatedPayload>(EVENTS.STATUS_UPDATED, handler);
}

/** Subscribe to file rename/move notifications. */
export function onFsRenamed(handler: (payload: FsRenamedPayload) => void): Promise<UnlistenFn> {
	return listenEvent<FsRenamedPayload>(EVENTS.FS_RENAMED, handler);
}
