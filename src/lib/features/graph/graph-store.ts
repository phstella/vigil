/**
 * Graph store -- manages note graph data, force-directed layout, pan/zoom,
 * and selection state for the interactive graph view.
 */

import { writable, derived, get } from 'svelte/store';
import type { NoteNode, LinkEdge } from '$lib/types/ipc';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/** A positioned node in the force-directed layout. */
export interface LayoutNode {
	id: string;
	path: string;
	title: string;
	tags: string[];
	x: number;
	y: number;
	vx: number;
	vy: number;
	/** Whether this node is pinned (not affected by simulation). */
	pinned: boolean;
}

export interface GraphEdge {
	fromId: string;
	toId: string;
	kind: 'wikilink' | 'markdown';
}

export interface ViewTransform {
	/** Pan offset X. */
	offsetX: number;
	/** Pan offset Y. */
	offsetY: number;
	/** Zoom scale factor. */
	scale: number;
}

export interface GraphState {
	nodes: LayoutNode[];
	edges: GraphEdge[];
	selectedNodeId: string | null;
	hoveredNodeId: string | null;
	transform: ViewTransform;
	isSimulating: boolean;
	isLoading: boolean;
	error: string | null;
	/** True when graph is displaying mock data because the backend is not yet available. */
	usingMockData: boolean;
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const REPULSION_STRENGTH = 800;
const SPRING_LENGTH = 80;
const SPRING_STIFFNESS = 0.04;
const DAMPING = 0.85;
const CENTER_GRAVITY = 0.01;
const MIN_VELOCITY = 0.1;
const MIN_ZOOM = 0.2;
const MAX_ZOOM = 4.0;
const ZOOM_STEP = 0.1;

// ---------------------------------------------------------------------------
// Mock data (used when backend is unavailable)
// ---------------------------------------------------------------------------

const MOCK_NODES: NoteNode[] = [
	{ id: 'n1', path: 'inbox.md', title: 'Inbox', tags: [] },
	{ id: 'n2', path: 'projects.md', title: 'Projects', tags: ['project'] },
	{ id: 'n3', path: 'daily-log.md', title: 'Daily Log', tags: ['journal'] },
	{ id: 'n4', path: 'ideas.md', title: 'Ideas', tags: ['idea'] },
	{ id: 'n5', path: 'references.md', title: 'References', tags: ['ref'] },
	{ id: 'n6', path: 'archive.md', title: 'Archive', tags: [] }
];

const MOCK_EDGES: LinkEdge[] = [
	{ from_node_id: 'n1', to_node_id: 'n2', kind: 'wikilink' },
	{ from_node_id: 'n1', to_node_id: 'n3', kind: 'wikilink' },
	{ from_node_id: 'n2', to_node_id: 'n4', kind: 'wikilink' },
	{ from_node_id: 'n3', to_node_id: 'n5', kind: 'markdown' },
	{ from_node_id: 'n4', to_node_id: 'n5', kind: 'wikilink' },
	{ from_node_id: 'n4', to_node_id: 'n6', kind: 'wikilink' },
	{ from_node_id: 'n5', to_node_id: 'n6', kind: 'markdown' }
];

// ---------------------------------------------------------------------------
// Helper: create positioned nodes from raw data
// ---------------------------------------------------------------------------

function createLayoutNodes(nodes: NoteNode[], width: number, height: number): LayoutNode[] {
	const cx = width / 2;
	const cy = height / 2;
	const radius = Math.min(width, height) * 0.3;

	return nodes.map((n, i) => {
		const angle = (2 * Math.PI * i) / nodes.length;
		return {
			id: n.id,
			path: n.path,
			title: n.title,
			tags: n.tags,
			x: cx + radius * Math.cos(angle) + (Math.random() - 0.5) * 20,
			y: cy + radius * Math.sin(angle) + (Math.random() - 0.5) * 20,
			vx: 0,
			vy: 0,
			pinned: false
		};
	});
}

function createGraphEdges(edges: LinkEdge[]): GraphEdge[] {
	return edges.map((e) => ({
		fromId: e.from_node_id,
		toId: e.to_node_id,
		kind: e.kind
	}));
}

// ---------------------------------------------------------------------------
// Force simulation tick
// ---------------------------------------------------------------------------

function simulationTick(
	nodes: LayoutNode[],
	edges: GraphEdge[]
): { nodes: LayoutNode[]; settled: boolean } {
	const n = nodes.length;
	if (n === 0) return { nodes, settled: true };

	// Compute center of mass
	let cmx = 0;
	let cmy = 0;
	for (const node of nodes) {
		cmx += node.x;
		cmy += node.y;
	}
	cmx /= n;
	cmy /= n;

	// Build adjacency map for quick lookup
	const nodeMap = new Map<string, number>();
	for (let i = 0; i < n; i++) {
		nodeMap.set(nodes[i].id, i);
	}

	// Force accumulators
	const fx = new Float64Array(n);
	const fy = new Float64Array(n);

	// Repulsion (all pairs)
	for (let i = 0; i < n; i++) {
		for (let j = i + 1; j < n; j++) {
			const dx = nodes[j].x - nodes[i].x;
			const dy = nodes[j].y - nodes[i].y;
			let dist = Math.sqrt(dx * dx + dy * dy);
			if (dist < 1) dist = 1;

			const force = REPULSION_STRENGTH / (dist * dist);
			const forceX = (dx / dist) * force;
			const forceY = (dy / dist) * force;

			fx[i] -= forceX;
			fy[i] -= forceY;
			fx[j] += forceX;
			fy[j] += forceY;
		}
	}

	// Spring attraction (edges)
	for (const edge of edges) {
		const si = nodeMap.get(edge.fromId);
		const ti = nodeMap.get(edge.toId);
		if (si === undefined || ti === undefined) continue;

		const dx = nodes[ti].x - nodes[si].x;
		const dy = nodes[ti].y - nodes[si].y;
		const dist = Math.sqrt(dx * dx + dy * dy);
		if (dist < 0.1) continue;

		const displacement = dist - SPRING_LENGTH;
		const force = SPRING_STIFFNESS * displacement;
		const forceX = (dx / dist) * force;
		const forceY = (dy / dist) * force;

		fx[si] += forceX;
		fy[si] += forceY;
		fx[ti] -= forceX;
		fy[ti] -= forceY;
	}

	// Center gravity
	for (let i = 0; i < n; i++) {
		fx[i] += (cmx - nodes[i].x) * CENTER_GRAVITY;
		fy[i] += (cmy - nodes[i].y) * CENTER_GRAVITY;
	}

	// Apply forces + damping
	let settled = true;
	const updated = nodes.map((node, i) => {
		if (node.pinned) return node;

		const nvx = (node.vx + fx[i]) * DAMPING;
		const nvy = (node.vy + fy[i]) * DAMPING;
		const speed = Math.sqrt(nvx * nvx + nvy * nvy);

		if (speed > MIN_VELOCITY) settled = false;

		return {
			...node,
			x: node.x + nvx,
			y: node.y + nvy,
			vx: nvx,
			vy: nvy
		};
	});

	return { nodes: updated, settled };
}

// ---------------------------------------------------------------------------
// Store factory
// ---------------------------------------------------------------------------

function createGraphStore() {
	const store = writable<GraphState>({
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

	let animFrameId: number | null = null;

	function stopSimulation() {
		if (animFrameId !== null) {
			cancelAnimationFrame(animFrameId);
			animFrameId = null;
		}
		store.update((s) => ({ ...s, isSimulating: false }));
	}

	function runSimulation() {
		store.update((s) => ({ ...s, isSimulating: true }));

		function tick() {
			const currentState = get(store);
			if (!currentState.isSimulating) return;

			const result = simulationTick(currentState.nodes, currentState.edges);
			store.update((s) => ({
				...s,
				nodes: result.nodes,
				isSimulating: !result.settled
			}));

			if (!result.settled) {
				animFrameId = requestAnimationFrame(tick);
			} else {
				animFrameId = null;
			}
		}

		animFrameId = requestAnimationFrame(tick);
	}

	return {
		subscribe: store.subscribe,

		/** Load graph data from backend or fallback to mock. */
		async loadGraph(width: number = 400, height: number = 400) {
			store.update((s) => ({ ...s, isLoading: true, error: null, usingMockData: false }));
			stopSimulation();

			let noteNodes: NoteNode[];
			let linkEdges: LinkEdge[];
			let isMock = false;

			try {
				const { getNoteGraph } = await import('$lib/ipc/links');
				const response = await getNoteGraph();
				noteNodes = response.nodes;
				linkEdges = response.edges;
			} catch (err) {
				// Backend command not available — falling back to demo data.
				console.warn('[graph-store] get_note_graph IPC failed — using demo data.', err);
				noteNodes = MOCK_NODES;
				linkEdges = MOCK_EDGES;
				isMock = true;
			}

			const nodes = createLayoutNodes(noteNodes, width, height);
			const edges = createGraphEdges(linkEdges);

			store.update((s) => ({
				...s,
				nodes,
				edges,
				selectedNodeId: null,
				hoveredNodeId: null,
				transform: { offsetX: 0, offsetY: 0, scale: 1 },
				isLoading: false,
				usingMockData: isMock
			}));

			runSimulation();
		},

		/** Select a node by ID (null to deselect). */
		selectNode(nodeId: string | null) {
			store.update((s) => ({ ...s, selectedNodeId: nodeId }));
		},

		/** Set hovered node ID. */
		hoverNode(nodeId: string | null) {
			store.update((s) => ({ ...s, hoveredNodeId: nodeId }));
		},

		/** Pan the view by a delta. */
		pan(dx: number, dy: number) {
			store.update((s) => ({
				...s,
				transform: {
					...s.transform,
					offsetX: s.transform.offsetX + dx,
					offsetY: s.transform.offsetY + dy
				}
			}));
		},

		/** Zoom in/out at a focal point. */
		zoom(delta: number, focalX: number, focalY: number) {
			store.update((s) => {
				const oldScale = s.transform.scale;
				const newScale = Math.max(
					MIN_ZOOM,
					Math.min(MAX_ZOOM, oldScale + delta * ZOOM_STEP)
				);
				const ratio = newScale / oldScale;

				// Adjust offset so zoom centers on focal point
				const newOffsetX = focalX - (focalX - s.transform.offsetX) * ratio;
				const newOffsetY = focalY - (focalY - s.transform.offsetY) * ratio;

				return {
					...s,
					transform: {
						offsetX: newOffsetX,
						offsetY: newOffsetY,
						scale: newScale
					}
				};
			});
		},

		/** Reset view transform to default. */
		resetView() {
			store.update((s) => ({
				...s,
				transform: { offsetX: 0, offsetY: 0, scale: 1 }
			}));
		},

		/** Pin/unpin a node (stops it from moving in simulation). */
		pinNode(nodeId: string, pinned: boolean) {
			store.update((s) => ({
				...s,
				nodes: s.nodes.map((nd) => (nd.id === nodeId ? { ...nd, pinned } : nd))
			}));
		},

		/** Update a node's position (for drag). */
		moveNode(nodeId: string, x: number, y: number) {
			store.update((s) => ({
				...s,
				nodes: s.nodes.map((nd) => (nd.id === nodeId ? { ...nd, x, y, vx: 0, vy: 0 } : nd))
			}));
		},

		/** Restart the force simulation. */
		restartSimulation() {
			stopSimulation();
			runSimulation();
		},

		/** Stop the force simulation. */
		stopSimulation,

		/** Destroy the store and cleanup. */
		destroy() {
			stopSimulation();
		}
	};
}

export const graphStore = createGraphStore();

/** Derived store: edges connected to the selected node. */
export const selectedEdges = derived({ subscribe: graphStore.subscribe }, ($graphState) => {
	if (!$graphState.selectedNodeId) return new Set<string>();
	const set = new Set<string>();
	for (const edge of $graphState.edges) {
		if (
			edge.fromId === $graphState.selectedNodeId ||
			edge.toId === $graphState.selectedNodeId
		) {
			set.add(`${edge.fromId}-${edge.toId}`);
		}
	}
	return set;
});

/** Derived store: neighbor node IDs of the selected node. */
export const selectedNeighbors = derived({ subscribe: graphStore.subscribe }, ($graphState) => {
	if (!$graphState.selectedNodeId) return new Set<string>();
	const set = new Set<string>();
	for (const edge of $graphState.edges) {
		if (edge.fromId === $graphState.selectedNodeId) set.add(edge.toId);
		if (edge.toId === $graphState.selectedNodeId) set.add(edge.fromId);
	}
	return set;
});
