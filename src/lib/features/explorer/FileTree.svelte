<script lang="ts">
	/**
	 * FileTree -- Virtualized tree rendering of TreeNode[].
	 *
	 * Performance (Task 3.11):
	 * Flattens the tree into a visible-nodes list and renders only the
	 * rows that fall within the scroll viewport. This keeps DOM node count
	 * bounded regardless of workspace size (critical for 10k+ file trees,
	 * per the <=1500ms first-usable-tree budget).
	 *
	 * For small trees (< VIRTUALIZE_THRESHOLD nodes), renders all nodes
	 * directly to avoid measurement overhead.
	 */

	import type { TreeNode } from './explorer-store.svelte';
	import FileTreeNode from './FileTreeNode.svelte';

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

	/** Height of each tree row in pixels. */
	const ROW_HEIGHT = 28;

	/** Only virtualize when the flattened node count exceeds this threshold. */
	const VIRTUALIZE_THRESHOLD = 200;

	/** Extra rows to render above and below the visible window. */
	const OVERSCAN = 10;

	interface FlatNode {
		node: TreeNode;
		depth: number;
	}

	/**
	 * Flatten the tree into a list of visible nodes (respecting expanded state).
	 * This is the key to virtualization -- we can index into this list by row number.
	 */
	function flattenTree(
		roots: TreeNode[],
		expandedSet: Set<string>,
		baseDepth: number
	): FlatNode[] {
		const result: FlatNode[] = [];
		function walk(items: TreeNode[], d: number) {
			for (const item of items) {
				result.push({ node: item, depth: d });
				if (item.kind === 'dir' && expandedSet.has(item.path) && item.children) {
					walk(item.children, d + 1);
				}
			}
		}
		walk(roots, baseDepth);
		return result;
	}

	let flatNodes = $derived(flattenTree(nodes, expandedDirs, depth));
	let useVirtualization = $derived(flatNodes.length >= VIRTUALIZE_THRESHOLD);
	let totalHeight = $derived(flatNodes.length * ROW_HEIGHT);

	// Scroll state for virtualized mode
	let scrollTop = $state(0);
	let containerHeight = $state(400); // will be measured
	let scrollContainer: HTMLDivElement | undefined = $state();

	function handleScroll() {
		if (scrollContainer) {
			scrollTop = scrollContainer.scrollTop;
		}
	}

	// Compute visible window
	let startIndex = $derived(Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN));
	let endIndex = $derived(
		Math.min(flatNodes.length, Math.ceil((scrollTop + containerHeight) / ROW_HEIGHT) + OVERSCAN)
	);
	let visibleNodes = $derived(flatNodes.slice(startIndex, endIndex));
	let offsetY = $derived(startIndex * ROW_HEIGHT);

	// Measure container height on mount
	$effect(() => {
		if (scrollContainer) {
			const observer = new ResizeObserver((entries) => {
				for (const entry of entries) {
					containerHeight = entry.contentRect.height;
				}
			});
			observer.observe(scrollContainer);
			return () => observer.disconnect();
		}
	});
</script>

{#if useVirtualization}
	<!-- Virtualized mode: only render visible rows -->
	<div
		bind:this={scrollContainer}
		class="h-full overflow-y-auto"
		onscroll={handleScroll}
		role={depth === 0 ? 'tree' : 'group'}
	>
		<div style="height: {totalHeight}px; position: relative;">
			<div style="position: absolute; top: {offsetY}px; left: 0; right: 0;">
				{#each visibleNodes as { node, depth: d } (node.path)}
					{@const expanded = node.kind === 'dir' && expandedDirs.has(node.path)}
					<div style="height: {ROW_HEIGHT}px;">
						<FileTreeNode
							{node}
							depth={d}
							isExpanded={expanded}
							isSelected={selectedFile === node.path}
							{onToggle}
							{onSelect}
						/>
					</div>
				{/each}
			</div>
		</div>
	</div>
{:else}
	<!-- Non-virtualized mode for small trees: simple recursive rendering -->
	<ul class="list-none p-0" role={depth === 0 ? 'tree' : 'group'}>
		{#each flatNodes as { node, depth: d } (node.path)}
			{@const expanded = node.kind === 'dir' && expandedDirs.has(node.path)}
			<li role="none">
				<FileTreeNode
					{node}
					depth={d}
					isExpanded={expanded}
					isSelected={selectedFile === node.path}
					{onToggle}
					{onSelect}
				/>
			</li>
		{/each}
	</ul>
{/if}
