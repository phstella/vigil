<script lang="ts">
	/**
	 * FileTree -- Recursive tree rendering of TreeNode[].
	 * Renders a FileTreeNode for each entry, recursing into expanded directories.
	 */

	import type { TreeNode } from './explorer-store';
	import FileTreeNode from './FileTreeNode.svelte';
	import FileTree from './FileTree.svelte';

	let {
		nodes,
		depth = 0,
		expandedDirs,
		selectedFile,
		onToggle,
		onSelect
	}: {
		nodes: TreeNode[];
		depth?: number;
		expandedDirs: Set<string>;
		selectedFile: string | null;
		onToggle: (path: string) => void;
		onSelect: (path: string) => void;
	} = $props();
</script>

<ul class="list-none p-0" role={depth === 0 ? 'tree' : 'group'}>
	{#each nodes as node (node.path)}
		{@const expanded = node.kind === 'dir' && expandedDirs.has(node.path)}
		<li role="none">
			<FileTreeNode
				{node}
				{depth}
				isExpanded={expanded}
				isSelected={selectedFile === node.path}
				{onToggle}
				{onSelect}
			/>
			{#if expanded && node.children && node.children.length > 0}
				<FileTree
					nodes={node.children}
					depth={depth + 1}
					{expandedDirs}
					{selectedFile}
					{onToggle}
					{onSelect}
				/>
			{/if}
		</li>
	{/each}
</ul>
