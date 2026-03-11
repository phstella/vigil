<script lang="ts">
	/**
	 * OmnibarItem -- A single result row in the omnibar results list.
	 *
	 * Displays a file icon, file name, and workspace-relative path.
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
	function fileIcon(ext: string | null): string {
		switch (ext) {
			case 'md':
				return '\u{1F4DD}';
			case 'ts':
			case 'js':
				return '\u{1F4C4}';
			case 'css':
				return '\u{1F3A8}';
			default:
				return '\u{1F4C1}';
		}
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
	<span class="shrink-0 text-sm" aria-hidden="true">{fileIcon(item.ext)}</span>
	<span class="flex min-w-0 flex-1 flex-col">
		<span class="truncate text-sm font-medium">{item.name}</span>
		<span class="truncate text-xs text-text-muted">{item.path}</span>
	</span>
</button>
