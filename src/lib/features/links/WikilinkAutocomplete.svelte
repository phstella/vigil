<script lang="ts">
	/**
	 * WikilinkAutocomplete -- Dropdown triggered by `[[` in the editor textarea.
	 *
	 * Listens for `[[` typing in the parent textarea, fuzzy-searches workspace
	 * notes via IPC, and inserts the selected link. Positioned absolutely relative
	 * to the parent container.
	 */

	import { fuzzyFind } from '$lib/ipc/search';
	import type { FuzzyMatch } from '$lib/types/ipc';

	let {
		query,
		visible,
		anchorX = 0,
		anchorY = 0,
		onSelect,
		onClose
	}: {
		/** The partial query text after `[[`. */
		query: string;
		/** Whether the autocomplete dropdown is visible. */
		visible: boolean;
		/** X position for the dropdown anchor. */
		anchorX?: number;
		/** Y position for the dropdown anchor. */
		anchorY?: number;
		/** Called when user selects a match. Passes the note name (stem). */
		onSelect: (noteName: string) => void;
		/** Called when the autocomplete should close (Esc, click outside). */
		onClose: () => void;
	} = $props();

	let matches = $state<FuzzyMatch[]>([]);
	let selectedIndex = $state(0);
	let isSearching = $state(false);

	/** Debounce timer for search. */
	let searchTimer: ReturnType<typeof setTimeout> | null = null;

	const DEBOUNCE_MS = 80;
	const MAX_RESULTS = 10;

	// React to query changes
	$effect(() => {
		if (!visible) {
			matches = [];
			selectedIndex = 0;
			return;
		}

		if (searchTimer) clearTimeout(searchTimer);

		const q = query;
		searchTimer = setTimeout(() => {
			searchTimer = null;
			void performSearch(q);
		}, DEBOUNCE_MS);
	});

	async function performSearch(q: string) {
		isSearching = true;
		try {
			// Filter to .md files by searching and filtering results
			const response = await fuzzyFind(q, MAX_RESULTS * 2);
			matches = response.matches.filter((m) => m.path.endsWith('.md')).slice(0, MAX_RESULTS);
			selectedIndex = 0;
		} catch {
			matches = [];
		} finally {
			isSearching = false;
		}
	}

	/**
	 * Extract a note name from a path: strip .md extension and use just
	 * the filename stem for simple cases, or the full path for nested files.
	 */
	function noteNameFromPath(path: string): string {
		// Remove .md extension
		const withoutExt = path.replace(/\.md$/, '');
		return withoutExt;
	}

	function handleSelect(match: FuzzyMatch) {
		onSelect(noteNameFromPath(match.path));
	}

	/** Handle keyboard navigation within the autocomplete. */
	export function handleKeydown(e: KeyboardEvent): boolean {
		if (!visible) return false;

		switch (e.key) {
			case 'ArrowDown':
				e.preventDefault();
				selectedIndex = Math.min(selectedIndex + 1, matches.length - 1);
				return true;
			case 'ArrowUp':
				e.preventDefault();
				selectedIndex = Math.max(selectedIndex - 1, 0);
				return true;
			case 'Enter':
			case 'Tab':
				if (matches.length > 0) {
					e.preventDefault();
					handleSelect(matches[selectedIndex]);
					return true;
				}
				return false;
			case 'Escape':
				e.preventDefault();
				onClose();
				return true;
			default:
				return false;
		}
	}
</script>

{#if visible && (matches.length > 0 || isSearching)}
	<div
		class="absolute z-50 w-64 rounded-md border border-surface-border bg-surface-raised shadow-lg"
		style="left: {anchorX}px; top: {anchorY}px;"
		role="listbox"
		aria-label="Link suggestions"
	>
		{#if isSearching && matches.length === 0}
			<div class="px-3 py-2 text-xs text-text-muted">Searching...</div>
		{/if}

		{#each matches as match, i (match.path)}
			<button
				class="flex w-full cursor-pointer items-center gap-2 px-3 py-1.5 text-left text-xs transition-colors"
				class:bg-accent-muted={i === selectedIndex}
				class:text-text-primary={i === selectedIndex}
				class:text-text-secondary={i !== selectedIndex}
				class:hover:bg-surface-overlay={i !== selectedIndex}
				role="option"
				aria-selected={i === selectedIndex}
				onclick={() => handleSelect(match)}
				onmouseenter={() => (selectedIndex = i)}
			>
				<svg
					class="h-3 w-3 shrink-0 text-text-muted"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
				>
					<path
						d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
					/>
				</svg>
				<span class="truncate">{match.display || match.path}</span>
			</button>
		{/each}

		{#if !isSearching && matches.length === 0 && query.length > 0}
			<div class="px-3 py-2 text-xs text-text-muted">
				No matching notes. Press Enter to create link.
			</div>
		{/if}
	</div>
{/if}
