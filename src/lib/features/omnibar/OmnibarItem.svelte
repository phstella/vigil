<script lang="ts">
	/**
	 * OmnibarItem -- A single result row in the omnibar results list.
	 *
	 * Displays a file icon, file name with fuzzy-match highlighting,
	 * workspace-relative path, and a match score badge.
	 * Highlights the row when it is the currently selected item.
	 */

	import type { OmnibarResult } from './omnibar-store';

	let {
		item,
		isSelected = false,
		onclick
	}: {
		item: OmnibarResult;
		isSelected?: boolean;
		onclick?: () => void;
	} = $props();

	/** Return a simple icon character based on file extension. */
	function fileIcon(ext: string | null, kind: string): string {
		if (kind === 'dir') return '\u{1F4C2}';
		switch (ext) {
			case 'md':
			case 'markdown':
			case 'mdx':
				return '\u{1F4DD}';
			case 'ts':
			case 'tsx':
			case 'js':
			case 'jsx':
				return '\u{1F4C4}';
			case 'css':
			case 'scss':
				return '\u{1F3A8}';
			case 'rs':
				return '\u{2699}';
			case 'json':
			case 'toml':
			case 'yaml':
			case 'yml':
				return '\u{2699}';
			default:
				return '\u{1F4C1}';
		}
	}

	/**
	 * Build an array of { text, highlighted } spans from a display string
	 * and matched character indices.
	 */
	function highlightSegments(
		display: string,
		indices: number[]
	): Array<{ text: string; highlighted: boolean }> {
		if (indices.length === 0) {
			return [{ text: display, highlighted: false }];
		}

		const indexSet = new Set(indices);
		const segments: Array<{ text: string; highlighted: boolean }> = [];
		let current = '';
		let currentHighlighted = false;

		for (let i = 0; i < display.length; i++) {
			const isMatch = indexSet.has(i);
			if (i === 0) {
				currentHighlighted = isMatch;
				current = display[i];
			} else if (isMatch === currentHighlighted) {
				current += display[i];
			} else {
				segments.push({ text: current, highlighted: currentHighlighted });
				current = display[i];
				currentHighlighted = isMatch;
			}
		}
		if (current) {
			segments.push({ text: current, highlighted: currentHighlighted });
		}

		return segments;
	}

	let nameSegments = $derived(highlightSegments(item.name, item.matchedIndices));
</script>

<button
	class="flex w-full items-center gap-3 px-3 py-2 text-left transition-colors {isSelected
		? 'bg-accent/15 text-text-primary'
		: 'text-text-secondary hover:bg-surface-overlay'}"
	{onclick}
	type="button"
	role="option"
	aria-selected={isSelected}
>
	<span class="shrink-0 text-sm" aria-hidden="true">{fileIcon(item.ext, item.kind)}</span>
	<span class="flex min-w-0 flex-1 flex-col">
		<span class="truncate text-sm font-medium">
			{#each nameSegments as seg, idx (idx)}
				{#if seg.highlighted}
					<span class="text-accent">{seg.text}</span>
				{:else}
					{seg.text}
				{/if}
			{/each}
		</span>
		<span class="truncate text-xs text-text-muted">{item.path}</span>
	</span>
	{#if item.score > 0}
		<span
			class="shrink-0 rounded-sm bg-surface-overlay px-1.5 py-0.5 text-[10px] tabular-nums text-text-muted"
			title="Match score"
		>
			{item.score}
		</span>
	{/if}
</button>
