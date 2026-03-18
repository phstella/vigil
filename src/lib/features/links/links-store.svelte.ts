/**
 * Links store -- manages backlinks state for the active note.
 *
 * Fetches backlinks via `get_backlinks` IPC when the active file changes,
 * and provides a refresh method for when content is saved (links may have
 * changed). Uses Svelte 5 fine-grained reactivity ($state).
 */

import { getBacklinks } from '$lib/ipc/links';
import { isVigilError } from '$lib/ipc/tauri';
import type { BacklinkRecord } from '$lib/types/ipc';

/** Debounce delay for backlinks refresh after save (ms). */
const REFRESH_DELAY_MS = 500;

function createLinksStore() {
	let activePath = $state<string | null>(null);
	let backlinks = $state<BacklinkRecord[]>([]);
	let isLoading = $state(false);
	let lastError = $state<string | null>(null);
	let isCollapsed = $state(false);

	let refreshTimer: ReturnType<typeof setTimeout> | null = null;

	function clearRefreshTimer() {
		if (refreshTimer !== null) {
			clearTimeout(refreshTimer);
			refreshTimer = null;
		}
	}

	/** Fetch backlinks for a given path from the backend. */
	async function fetchBacklinks(path: string): Promise<void> {
		isLoading = true;
		lastError = null;

		try {
			const response = await getBacklinks(path);
			// Only update if still on the same file (avoid race conditions)
			if (activePath === path) {
				backlinks = response.backlinks;
			}
		} catch (err: unknown) {
			if (activePath === path) {
				if (isVigilError(err)) {
					lastError = `Backlinks: ${err.message}`;
				} else {
					lastError = 'Failed to fetch backlinks';
				}
				backlinks = [];
			}
		} finally {
			if (activePath === path) {
				isLoading = false;
			}
		}
	}

	return {
		get activePath() {
			return activePath;
		},
		get backlinks() {
			return backlinks;
		},
		get isLoading() {
			return isLoading;
		},
		get lastError() {
			return lastError;
		},
		get isCollapsed() {
			return isCollapsed;
		},

		/** Set the active note and fetch its backlinks. */
		async setActivePath(path: string | null): Promise<void> {
			clearRefreshTimer();

			if (path === activePath) return;
			activePath = path;

			if (!path) {
				backlinks = [];
				isLoading = false;
				lastError = null;
				return;
			}

			await fetchBacklinks(path);
		},

		/**
		 * Schedule a backlinks refresh (e.g., after a save that may have
		 * changed link content). Debounced to avoid rapid fire.
		 */
		scheduleRefresh(): void {
			if (!activePath) return;
			clearRefreshTimer();
			const path = activePath;
			refreshTimer = setTimeout(() => {
				refreshTimer = null;
				if (activePath === path) {
					void fetchBacklinks(path);
				}
			}, REFRESH_DELAY_MS);
		},

		/** Immediately refresh backlinks for the current path. */
		async refresh(): Promise<void> {
			clearRefreshTimer();
			if (activePath) {
				await fetchBacklinks(activePath);
			}
		},

		/** Toggle collapsed state of the backlinks panel. */
		toggleCollapsed(): void {
			isCollapsed = !isCollapsed;
		},

		/** Reset the store to initial state. */
		reset(): void {
			clearRefreshTimer();
			activePath = null;
			backlinks = [];
			isLoading = false;
			lastError = null;
			isCollapsed = false;
		},

		/** Cancel any pending refresh timer. */
		cancelRefresh(): void {
			clearRefreshTimer();
		}
	};
}

export const linksStore = createLinksStore();
