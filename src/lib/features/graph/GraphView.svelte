<script lang="ts">
	/**
	 * GraphView -- Placeholder graph visualization panel for the sidebar.
	 * Renders a static SVG with mock nodes and edges representing note
	 * connections. Real force-directed layout and backend integration
	 * will be wired in Epic 3.
	 */

	interface GraphNode {
		id: string;
		label: string;
		cx: number;
		cy: number;
	}

	interface GraphEdge {
		from: string;
		to: string;
	}

	const nodes: GraphNode[] = [
		{ id: 'inbox', label: 'Inbox', cx: 120, cy: 60 },
		{ id: 'projects', label: 'Projects', cx: 50, cy: 140 },
		{ id: 'daily', label: 'Daily Log', cx: 190, cy: 130 },
		{ id: 'ideas', label: 'Ideas', cx: 80, cy: 230 },
		{ id: 'ref', label: 'References', cx: 170, cy: 220 },
		{ id: 'archive', label: 'Archive', cx: 130, cy: 310 }
	];

	const edges: GraphEdge[] = [
		{ from: 'inbox', to: 'projects' },
		{ from: 'inbox', to: 'daily' },
		{ from: 'projects', to: 'ideas' },
		{ from: 'daily', to: 'ref' },
		{ from: 'ideas', to: 'ref' },
		{ from: 'ideas', to: 'archive' },
		{ from: 'ref', to: 'archive' }
	];

	function getNode(id: string): GraphNode | undefined {
		return nodes.find((n) => n.id === id);
	}
</script>

<div class="flex h-full flex-col">
	<!-- Graph canvas -->
	<div class="flex-1 overflow-hidden rounded-md bg-surface-base mx-2 my-2">
		<svg
			viewBox="0 0 240 360"
			class="h-full w-full"
			aria-label="Graph view placeholder showing mock note connections"
		>
			<!-- Edges -->
			{#each edges as edge (edge.from + '-' + edge.to)}
				{@const fromNode = getNode(edge.from)}
				{@const toNode = getNode(edge.to)}
				{#if fromNode && toNode}
					<line
						x1={fromNode.cx}
						y1={fromNode.cy}
						x2={toNode.cx}
						y2={toNode.cy}
						stroke="var(--color-surface-border)"
						stroke-width="1.5"
					/>
				{/if}
			{/each}

			<!-- Nodes -->
			{#each nodes as node (node.id)}
				<circle
					cx={node.cx}
					cy={node.cy}
					r="8"
					fill="var(--color-accent-muted)"
					stroke="var(--color-accent)"
					stroke-width="1.5"
				/>
				<text
					x={node.cx}
					y={node.cy + 20}
					text-anchor="middle"
					fill="var(--color-text-muted)"
					font-size="10"
					font-family="var(--font-sans)"
				>
					{node.label}
				</text>
			{/each}
		</svg>
	</div>

	<!-- Footer info -->
	<div class="shrink-0 border-t border-surface-border px-3 py-2">
		<p class="text-[11px] leading-relaxed text-text-muted">
			{nodes.length} nodes &middot; {edges.length} edges
		</p>
		<p class="mt-1 text-[10px] text-text-muted/60">
			Graph view &mdash; connect to backend in Epic 3
		</p>
	</div>
</div>
