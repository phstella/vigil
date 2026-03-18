<script lang="ts">
	/**
	 * FileTreeNode -- Single tree entry: folder (expandable) or file (selectable).
	 * Indentation scales with depth. Folders show a rotating chevron; files show
	 * a document icon. Selected file is highlighted.
	 */

	import type { TreeNode } from './explorer-store.svelte';

	let {
		node,
		depth = 0,
		isExpanded = false,
		isSelected = false,
		onToggle,
		onSelect
	}: {
		node: TreeNode;
		depth?: number;
		isExpanded?: boolean;
		isSelected?: boolean;
		onToggle: (path: string) => void;
		onSelect: (path: string) => void;
	} = $props();

	const isDir = $derived(node.kind === 'dir');
	const indent = $derived(depth * 16);

	function handleClick() {
		if (isDir) {
			onToggle(node.path);
		} else {
			onSelect(node.path);
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' || event.key === ' ') {
			event.preventDefault();
			handleClick();
		}
	}
</script>

<button
	class="flex w-full items-center gap-1 rounded-sm px-1 py-0.5 text-left text-sm transition-colors
		{isSelected
		? 'bg-accent/15 text-accent'
		: 'text-text-secondary hover:bg-surface-overlay hover:text-text-primary'}"
	style="padding-left: {indent + 4}px"
	onclick={handleClick}
	onkeydown={handleKeydown}
	role="treeitem"
	aria-expanded={isDir ? isExpanded : undefined}
	aria-selected={isSelected}
	title={node.path}
>
	{#if isDir}
		<!-- Chevron icon: rotates 90deg when expanded -->
		<svg
			class="h-3.5 w-3.5 shrink-0 transition-transform duration-150 {isExpanded
				? 'rotate-90'
				: 'rotate-0'}"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
		>
			<path d="M9 6l6 6-6 6" />
		</svg>
		<!-- Folder icon -->
		<svg
			class="h-4 w-4 shrink-0 text-accent-muted"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="1.5"
		>
			<path d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
		</svg>
	{:else}
		<!-- Spacer to align with chevron -->
		<span class="inline-block h-3.5 w-3.5 shrink-0"></span>
		<!-- Document icon -->
		<svg
			class="h-4 w-4 shrink-0 text-text-muted"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="1.5"
		>
			<path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8l-6-6z" />
			<polyline points="14,2 14,8 20,8" />
		</svg>
	{/if}

	<span class="truncate">{node.name}</span>
</button>
