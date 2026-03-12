<script lang="ts">
	/**
	 * GraphView -- Interactive force-directed graph visualization for the sidebar.
	 *
	 * Features:
	 * - Force-directed layout (spring-charge simulation)
	 * - Pan via mouse drag on canvas background
	 * - Zoom via mouse wheel
	 * - Node selection on click (highlights node + connected edges)
	 * - Node drag to reposition
	 * - Double-click node to open the file
	 * - Hover tooltip with note path
	 */

	import { onMount, onDestroy } from 'svelte';
	import {
		graphStore,
		selectedEdges,
		selectedNeighbors,
		type LayoutNode,
		type GraphState
	} from './graph-store';
	import { editorStore } from '$lib/stores/editor';

	// -----------------------------------------------------------------------
	// Reactive graphData from stores
	// -----------------------------------------------------------------------

	let graphData: GraphState = $state({
		nodes: [],
		edges: [],
		selectedNodeId: null,
		hoveredNodeId: null,
		transform: { offsetX: 0, offsetY: 0, scale: 1 },
		isSimulating: false,
		isLoading: false,
		error: null,
		usingMockData: false
	});

	let selEdges: Set<string> = $state(new Set());
	let selNeighbors: Set<string> = $state(new Set());

	let unsubGraphData: (() => void) | null = null;
	let unsubEdges: (() => void) | null = null;
	let unsubNeighbors: (() => void) | null = null;

	// -----------------------------------------------------------------------
	// DOM refs and drag state
	// -----------------------------------------------------------------------

	let svgEl: SVGSVGElement | undefined = $state(undefined);
	let containerEl: HTMLDivElement | undefined = $state(undefined);

	/** Drag mode: 'pan' for background drag, 'node' for node drag. */
	let dragMode: 'pan' | 'node' | null = $state(null);
	let dragNodeId: string | null = $state(null);
	let dragStartX = 0;
	let dragStartY = 0;
	let dragStartOffsetX = 0;
	let dragStartOffsetY = 0;
	let dragStartNodeX = 0;
	let dragStartNodeY = 0;

	// -----------------------------------------------------------------------
	// Node styling constants
	// -----------------------------------------------------------------------

	const NODE_RADIUS = 8;
	const SELECTED_RADIUS = 11;
	const FONT_SIZE = 10;

	// -----------------------------------------------------------------------
	// Event handlers
	// -----------------------------------------------------------------------

	function handleWheel(e: WheelEvent) {
		e.preventDefault();
		const delta = e.deltaY > 0 ? -1 : 1;
		graphStore.zoom(delta, e.offsetX, e.offsetY);
	}

	function handleMouseDown(e: MouseEvent) {
		if (e.button !== 0) return;
		// Background pan
		dragMode = 'pan';
		dragStartX = e.clientX;
		dragStartY = e.clientY;
		dragStartOffsetX = graphData.transform.offsetX;
		dragStartOffsetY = graphData.transform.offsetY;
	}

	function handleNodeMouseDown(e: MouseEvent, node: LayoutNode) {
		e.stopPropagation();
		if (e.button !== 0) return;

		dragMode = 'node';
		dragNodeId = node.id;
		dragStartX = e.clientX;
		dragStartY = e.clientY;
		dragStartNodeX = node.x;
		dragStartNodeY = node.y;

		// Pin the node while dragging
		graphStore.pinNode(node.id, true);
	}

	function handleMouseMove(e: MouseEvent) {
		if (!dragMode) return;

		if (dragMode === 'pan') {
			const dx = e.clientX - dragStartX;
			const dy = e.clientY - dragStartY;
			graphStore.pan(
				dragStartOffsetX + dx - graphData.transform.offsetX,
				dragStartOffsetY + dy - graphData.transform.offsetY
			);
		} else if (dragMode === 'node' && dragNodeId) {
			const dx = (e.clientX - dragStartX) / graphData.transform.scale;
			const dy = (e.clientY - dragStartY) / graphData.transform.scale;
			graphStore.moveNode(dragNodeId, dragStartNodeX + dx, dragStartNodeY + dy);
		}
	}

	function handleMouseUp() {
		if (dragMode === 'node' && dragNodeId) {
			// Unpin after drag if moved less than 3px (was a click, not drag)
			const node = graphData.nodes.find((n) => n.id === dragNodeId);
			if (node) {
				const dx = Math.abs(node.x - dragStartNodeX);
				const dy = Math.abs(node.y - dragStartNodeY);
				if (dx < 3 && dy < 3) {
					graphStore.pinNode(dragNodeId, false);
				}
			}
		}
		dragMode = null;
		dragNodeId = null;
	}

	function handleNodeClick(e: MouseEvent, node: LayoutNode) {
		e.stopPropagation();
		// Only select if not dragged significantly
		const dx = Math.abs(e.clientX - dragStartX);
		const dy = Math.abs(e.clientY - dragStartY);
		if (dx < 5 && dy < 5) {
			graphStore.selectNode(graphData.selectedNodeId === node.id ? null : node.id);
		}
	}

	function handleNodeDblClick(e: MouseEvent, node: LayoutNode) {
		e.stopPropagation();
		e.preventDefault();
		// Open the note file
		const language = node.path.endsWith('.md') ? 'markdown' : 'plaintext';
		editorStore.openFile(node.path, '', language);
	}

	function handleNodeKeydown(e: KeyboardEvent, node: LayoutNode) {
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			graphStore.selectNode(graphData.selectedNodeId === node.id ? null : node.id);
		}
	}

	function handleSvgKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			graphStore.selectNode(null);
		}
	}

	function handleBackgroundClick() {
		graphStore.selectNode(null);
	}

	function handleNodeEnter(node: LayoutNode) {
		graphStore.hoverNode(node.id);
	}

	function handleNodeLeave() {
		graphStore.hoverNode(null);
	}

	// -----------------------------------------------------------------------
	// Node appearance helpers
	// -----------------------------------------------------------------------

	function nodeRadius(node: LayoutNode): number {
		return graphData.selectedNodeId === node.id ? SELECTED_RADIUS : NODE_RADIUS;
	}

	function nodeOpacity(node: LayoutNode): number {
		if (!graphData.selectedNodeId) return 1;
		if (node.id === graphData.selectedNodeId) return 1;
		if (selNeighbors.has(node.id)) return 0.9;
		return 0.25;
	}

	function edgeOpacity(fromId: string, toId: string): number {
		if (!graphData.selectedNodeId) return 0.5;
		const key = `${fromId}-${toId}`;
		return selEdges.has(key) ? 1 : 0.08;
	}

	function edgeStrokeColor(fromId: string, toId: string): string {
		if (!graphData.selectedNodeId) return 'var(--color-surface-border)';
		const key = `${fromId}-${toId}`;
		return selEdges.has(key) ? 'var(--color-accent)' : 'var(--color-surface-border)';
	}

	function nodeFill(node: LayoutNode): string {
		if (node.id === graphData.selectedNodeId) return 'var(--color-accent)';
		if (node.id === graphData.hoveredNodeId) return 'var(--color-accent-strong)';
		return 'var(--color-accent-muted)';
	}

	function nodeStrokeColor(node: LayoutNode): string {
		if (node.id === graphData.selectedNodeId) return 'var(--color-accent-strong)';
		return 'var(--color-accent)';
	}

	// -----------------------------------------------------------------------
	// Lifecycle
	// -----------------------------------------------------------------------

	onMount(() => {
		unsubGraphData = graphStore.subscribe((s) => {
			graphData = s;
		});
		unsubEdges = selectedEdges.subscribe((s) => {
			selEdges = s;
		});
		unsubNeighbors = selectedNeighbors.subscribe((s) => {
			selNeighbors = s;
		});

		// Load graph after mount
		const width = containerEl?.clientWidth ?? 400;
		const height = containerEl?.clientHeight ?? 400;
		graphStore.loadGraph(width, height);

		// Global mouse listeners for drag handling
		window.addEventListener('mousemove', handleMouseMove);
		window.addEventListener('mouseup', handleMouseUp);
	});

	onDestroy(() => {
		unsubGraphData?.();
		unsubEdges?.();
		unsubNeighbors?.();
		graphStore.stopSimulation();
		window.removeEventListener('mousemove', handleMouseMove);
		window.removeEventListener('mouseup', handleMouseUp);
	});

	// -----------------------------------------------------------------------
	// Derived helpers
	// -----------------------------------------------------------------------

	function getNodeById(id: string): LayoutNode | undefined {
		return graphData.nodes.find((n) => n.id === id);
	}
</script>

<div class="flex h-full flex-col" role="region" aria-label="Note graph view">
	<!-- Toolbar -->
	<div class="flex shrink-0 items-center gap-1 border-b border-surface-border px-2 py-1">
		<button
			class="rounded px-1.5 py-0.5 text-[10px] text-text-muted hover:bg-surface-overlay hover:text-text-primary"
			onclick={() => graphStore.resetView()}
			title="Reset view"
			type="button"
		>
			Reset
		</button>
		<button
			class="rounded px-1.5 py-0.5 text-[10px] text-text-muted hover:bg-surface-overlay hover:text-text-primary"
			onclick={() => graphStore.zoom(2, 130, 200)}
			title="Zoom in"
			type="button"
		>
			+
		</button>
		<button
			class="rounded px-1.5 py-0.5 text-[10px] text-text-muted hover:bg-surface-overlay hover:text-text-primary"
			onclick={() => graphStore.zoom(-2, 130, 200)}
			title="Zoom out"
			type="button"
		>
			&minus;
		</button>
		{#if graphData.usingMockData}
			<span
				class="ml-auto rounded bg-warning/15 px-1.5 py-0.5 text-[9px] font-medium text-warning"
				title="Backend graph command not available yet — showing demo data (Epic 4)"
			>Demo data</span>
		{/if}
		{#if graphData.isSimulating}
			<span class:ml-auto={!graphData.usingMockData} class="text-[9px] text-text-muted">simulating...</span>
		{/if}
	</div>

	<!-- Graph canvas -->
	<div
		class="relative mx-0 my-0 flex-1 overflow-hidden bg-surface-base"
		bind:this={containerEl}
	>
		{#if graphData.isLoading}
			<div class="flex h-full items-center justify-center">
				<p class="text-xs text-text-muted">Loading graph...</p>
			</div>
		{:else if graphData.error}
			<div class="flex h-full items-center justify-center">
				<p class="text-xs text-error">{graphData.error}</p>
			</div>
		{:else}
			<svg
				bind:this={svgEl}
				class="h-full w-full"
				style="cursor: {dragMode === 'pan' ? 'grabbing' : 'grab'}"
				aria-label="Interactive note graph with {graphData.nodes.length} nodes and {graphData.edges.length} edges"
				role="img"
				onmousedown={handleMouseDown}
				onclick={handleBackgroundClick}
				onkeydown={handleSvgKeydown}
				onwheel={handleWheel}
				tabindex={-1}
			>
				<g transform="translate({graphData.transform.offsetX}, {graphData.transform.offsetY}) scale({graphData.transform.scale})">
					<!-- Edges -->
					{#each graphData.edges as edge (edge.fromId + '-' + edge.toId)}
						{@const fromNode = getNodeById(edge.fromId)}
						{@const toNode = getNodeById(edge.toId)}
						{#if fromNode && toNode}
							<line
								x1={fromNode.x}
								y1={fromNode.y}
								x2={toNode.x}
								y2={toNode.y}
								stroke={edgeStrokeColor(edge.fromId, edge.toId)}
								stroke-width={selEdges.has(`${edge.fromId}-${edge.toId}`) ? 2 : 1}
								opacity={edgeOpacity(edge.fromId, edge.toId)}
							/>
						{/if}
					{/each}

					<!-- Nodes -->
					{#each graphData.nodes as node (node.id)}
						<g
							class="graph-node"
							style="cursor: pointer"
							opacity={nodeOpacity(node)}
							onmousedown={(e) => handleNodeMouseDown(e, node)}
							onclick={(e) => handleNodeClick(e, node)}
							ondblclick={(e) => handleNodeDblClick(e, node)}
							onmouseenter={() => handleNodeEnter(node)}
							onmouseleave={() => handleNodeLeave()}
							onkeydown={(e) => handleNodeKeydown(e, node)}
							role="button"
							tabindex={0}
							aria-label="Note: {node.title}"
						>
							<circle
								cx={node.x}
								cy={node.y}
								r={nodeRadius(node)}
								fill={nodeFill(node)}
								stroke={nodeStrokeColor(node)}
								stroke-width={node.id === graphData.selectedNodeId ? 2.5 : 1.5}
							/>
							<text
								x={node.x}
								y={node.y + nodeRadius(node) + FONT_SIZE + 2}
								text-anchor="middle"
								fill={node.id === graphData.selectedNodeId ? 'var(--color-text-primary)' : 'var(--color-text-muted)'}
								font-size={FONT_SIZE}
								font-family="var(--font-sans)"
								pointer-events="none"
							>
								{node.title}
							</text>
							{#if node.pinned}
								<circle
									cx={node.x + nodeRadius(node) - 2}
									cy={node.y - nodeRadius(node) + 2}
									r="3"
									fill="var(--color-warning)"
									pointer-events="none"
								/>
							{/if}
						</g>
					{/each}
				</g>
			</svg>

			<!-- Tooltip for hovered node -->
			{#if graphData.hoveredNodeId && !dragMode}
				{@const hoveredNode = getNodeById(graphData.hoveredNodeId)}
				{#if hoveredNode}
					<div
						class="pointer-events-none absolute bottom-2 left-2 rounded bg-surface-overlay px-2 py-1 text-[10px] text-text-secondary shadow-md"
					>
						{hoveredNode.path}
						{#if hoveredNode.tags.length > 0}
							<span class="ml-1 text-text-muted">
								({hoveredNode.tags.join(', ')})
							</span>
						{/if}
					</div>
				{/if}
			{/if}
		{/if}
	</div>

	<!-- Footer info -->
	<div class="shrink-0 border-t border-surface-border px-3 py-1.5">
		<p class="text-[11px] leading-relaxed text-text-muted">
			{graphData.nodes.length} nodes &middot; {graphData.edges.length} edges
			{#if graphData.selectedNodeId}
				{@const sel = getNodeById(graphData.selectedNodeId)}
				{#if sel}
					&middot; <span class="text-accent">{sel.title}</span>
				{/if}
			{/if}
		</p>
	</div>
</div>
