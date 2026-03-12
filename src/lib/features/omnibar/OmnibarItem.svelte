<script lang="ts">
	/**
	 * OmnibarItem -- A single result row in the omnibar results list.
	 *
	 * Supports two result types:
	 * - **file**: Displays file icon, file name with fuzzy-match highlighting,
	 *   workspace-relative path, and a match score badge.
	 * - **content**: Displays file icon, file name with line number,
	 *   preview snippet with match highlighting, and a score badge.
	 *
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
	function fileIcon(ext: string | null, kind?: string): string {
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
	 * and matched character indices (for file mode).
	 */
	function highlightByIndices(
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

	/**
	 * Build an array of { text, highlighted } spans from a preview line
	 * and column range (for content mode).
	 */
	function highlightByColumns(
		preview: string,
		startCol: number,
		endCol: number
	): Array<{ text: string; highlighted: boolean }> {
		const safeStart = Math.max(0, Math.min(startCol, preview.length));
		const safeEnd = Math.max(safeStart, Math.min(endCol, preview.length));

		const segments: Array<{ text: string; highlighted: boolean }> = [];
		if (safeStart > 0) {
			segments.push({ text: preview.slice(0, safeStart), highlighted: false });
		}
		if (safeEnd > safeStart) {
			segments.push({ text: preview.slice(safeStart, safeEnd), highlighted: true });
		}
		if (safeEnd < preview.length) {
			segments.push({ text: preview.slice(safeEnd), highlighted: false });
		}
		if (segments.length === 0) {
			segments.push({ text: preview, highlighted: false });
		}

		return segments;
	}
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
	<span class="shrink-0 text-sm" aria-hidden="true">{fileIcon(item.ext, item.type === 'file' ? item.kind : undefined)}</span>
	<span class="flex min-w-0 flex-1 flex-col">
		{#if item.type === 'file'}
			<!-- File mode: name with fuzzy-match highlighting -->
			{@const nameSegments = highlightByIndices(item.name, item.matchedIndices)}
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
		{:else}
			<!-- Content mode: file name + line number, preview with match highlighting -->
			<span class="truncate text-sm font-medium">
				{item.name}<span class="ml-1 text-xs text-text-muted">:{item.lineNumber}</span>
			</span>
			{@const previewSegments = highlightByColumns(item.preview, item.lineStartCol, item.lineEndCol)}
			<span class="truncate text-xs text-text-muted font-mono">
				{#each previewSegments as seg, idx (idx)}
					{#if seg.highlighted}
						<span class="text-accent font-semibold">{seg.text}</span>
					{:else}
						{seg.text}
					{/if}
				{/each}
			</span>
			<span class="truncate text-[10px] text-text-muted">{item.path}</span>
		{/if}
	</span>
	{#if item.score > 0}
		<span
			class="shrink-0 rounded-sm bg-surface-overlay px-1.5 py-0.5 text-[10px] tabular-nums text-text-muted"
			title="Match score"
		>
			{Math.round(item.score)}
		</span>
	{/if}
</button>
