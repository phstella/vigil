/**
 * Explorer UI state store.
 *
 * Provides IPC-backed file-tree data and local UI state (selection, expansion)
 * for the explorer panel. Tree data is loaded lazily per directory via `listDir`.
 * File content is loaded via `readFile` when a file is selected.
 *
 * All paths stored and emitted are workspace-relative, per the IPC contract.
 */

import type { DirEntry } from '$lib/types/store';
import { listDir, readFile } from '$lib/ipc/files';
import { editorStore } from '$lib/stores/editor';
import { isMarkdownFile } from '$lib/utils/file-routing';
import { detectLanguage } from '$lib/features/editor/code-store';
import { uiStore } from '$lib/stores/ui';

/** A tree node extends DirEntry with optional children for recursive rendering. */
export interface TreeNode extends DirEntry {
	children?: TreeNode[];
	/** Whether this directory's children have been loaded from the backend. */
	childrenLoaded?: boolean;
}

export interface CollectionSummary {
	name: string;
	path: string;
	filesCount: number;
	notesCount: number;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function summarizeNode(node: TreeNode): { filesCount: number; notesCount: number } {
	if (node.kind === 'file') {
		return {
			filesCount: 1,
			notesCount: node.ext === 'md' ? 1 : 0
		};
	}

	let filesCount = 0;
	let notesCount = 0;
	for (const child of node.children ?? []) {
		const childSummary = summarizeNode(child);
		filesCount += childSummary.filesCount;
		notesCount += childSummary.notesCount;
	}

	return { filesCount, notesCount };
}

/** Convert a DirEntry from listDir into a TreeNode. */
function entryToTreeNode(entry: DirEntry): TreeNode {
	return {
		...entry,
		children: entry.kind === 'dir' ? [] : undefined,
		childrenLoaded: false
	};
}

/**
 * Deep-clone a tree, replacing children of a target directory with new entries.
 * Returns a new array (immutable update).
 */
function updateTreeChildren(
	tree: TreeNode[],
	dirPath: string,
	newChildren: TreeNode[]
): TreeNode[] {
	return tree.map((node) => {
		if (node.path === dirPath && node.kind === 'dir') {
			return { ...node, children: newChildren, childrenLoaded: true };
		}
		if (node.kind === 'dir' && node.children && node.children.length > 0) {
			const updatedChildren = updateTreeChildren(node.children, dirPath, newChildren);
			if (updatedChildren !== node.children) {
				return { ...node, children: updatedChildren };
			}
		}
		return node;
	});
}

/**
 * Mark a directory as needing refresh by clearing its childrenLoaded flag.
 */
function markDirNeedsRefresh(tree: TreeNode[], dirPath: string): TreeNode[] {
	return tree.map((node) => {
		if (node.path === dirPath && node.kind === 'dir') {
			return { ...node, childrenLoaded: false };
		}
		if (node.kind === 'dir' && node.children && node.children.length > 0) {
			const updatedChildren = markDirNeedsRefresh(node.children, dirPath);
			if (updatedChildren !== node.children) {
				return { ...node, children: updatedChildren };
			}
		}
		return node;
	});
}

/**
 * Remove a specific path from the tree (for deletions).
 */
function removeFromTree(tree: TreeNode[], targetPath: string): TreeNode[] {
	return tree
		.filter((node) => node.path !== targetPath)
		.map((node) => {
			if (node.kind === 'dir' && node.children) {
				const filtered = removeFromTree(node.children, targetPath);
				if (filtered !== node.children) {
					return { ...node, children: filtered };
				}
			}
			return node;
		});
}

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

export interface ExplorerState {
	/** Root tree nodes. */
	tree: TreeNode[];
	/** Workspace display name. */
	workspaceName: string;
	/** Total markdown note count for the workspace. */
	notesCount: number;
	/** Total file count for the workspace. */
	filesCount: number;
	/** Top-level collection summaries for explorer header display. */
	collections: CollectionSummary[];
	/** Set of expanded directory paths. */
	expandedDirs: Set<string>;
	/** Currently selected file path, or null. */
	selectedFile: string | null;
	/** Whether the workspace is currently being loaded. */
	loading: boolean;
	/** Error message if workspace loading failed. */
	error: string | null;
}

function createExplorerStore() {
	let tree = $state<TreeNode[]>([]);
	let workspaceName = $state<string>('');
	let notesCount = $state<number>(0);
	let filesCount = $state<number>(0);
	let expandedDirs = $state<Set<string>>(new Set<string>());
	let selectedFile = $state<string | null>(null);
	let loading = $state<boolean>(false);
	let error = $state<string | null>(null);

	const collections = $derived.by(() => {
		return tree
			.filter((node): node is TreeNode & { kind: 'dir' } => node.kind === 'dir')
			.map((node) => {
				const summary = summarizeNode(node);
				return {
					name: node.name,
					path: node.path,
					filesCount: summary.filesCount,
					notesCount: summary.notesCount
				};
			});
	});

	const derivedNotesCount = $derived.by(() => {
		return tree.reduce((sum, node) => sum + summarizeNode(node).notesCount, 0);
	});

	return {
		get tree() {
			return tree;
		},
		get workspaceName() {
			return workspaceName;
		},
		get notesCount() {
			return notesCount || derivedNotesCount;
		},
		get filesCount() {
			return filesCount;
		},
		get collections() {
			return collections;
		},
		get expandedDirs() {
			return expandedDirs;
		},
		get selectedFile() {
			return selectedFile;
		},
		get loading() {
			return loading;
		},
		get error() {
			return error;
		},

		/**
		 * Initialize the explorer with workspace root data.
		 * Called after openWorkspace succeeds.
		 */
		async loadWorkspaceRoot(name: string, rootNotesCount: number, rootFilesCount: number) {
			loading = true;
			error = null;
			workspaceName = name;
			notesCount = rootNotesCount;
			filesCount = rootFilesCount;

			try {
				const response = await listDir('');
				tree = response.entries.map(entryToTreeNode);
				loading = false;
			} catch (err: unknown) {
				const msg = err && typeof err === 'object' && 'message' in err
					? (err as { message: string }).message
					: 'Failed to load workspace root';
				error = msg;
				loading = false;
				console.error('[explorer] failed to load workspace root:', err);
			}
		},

		/**
		 * Toggle a directory's expanded/collapsed state.
		 * When expanding, lazily loads children via listDir if not yet loaded.
		 */
		async toggleExpand(path: string) {
			const next = new Set(expandedDirs);
			if (next.has(path)) {
				next.delete(path);
				expandedDirs = next;
				return;
			}

			// Expanding: check if children need loading
			next.add(path);
			expandedDirs = next;

			// Find the node to check if children are loaded
			const node = findNode(tree, path);
			if (node && node.kind === 'dir' && !node.childrenLoaded) {
				try {
					const response = await listDir(path);
					const children = response.entries.map(entryToTreeNode);
					tree = updateTreeChildren(tree, path, children);
				} catch (err) {
					console.error('[explorer] failed to load directory:', path, err);
				}
			}
		},

		/** Select a file by path and open it in the appropriate editor pane via IPC readFile. */
		async selectFile(path: string) {
			selectedFile = path;

			try {
				const response = await readFile(path);
				const language = isMarkdownFile(path) ? 'markdown' : detectLanguage(path);
				editorStore.openFile(path, response.content, language);

				// Show the right panel when opening a code file
				if (!isMarkdownFile(path)) {
					uiStore.openRightPanel();
				}
			} catch (err) {
				console.error('[explorer] failed to read file:', path, err);
				// Still open the file with empty content so the tab exists
				const language = isMarkdownFile(path) ? 'markdown' : detectLanguage(path);
				editorStore.openFile(path, '', language);
			}
		},

		/** Check if a directory is expanded. */
		isExpanded(path: string): boolean {
			return expandedDirs.has(path);
		},

		/**
		 * Handle a file-watcher index-updated event.
		 * Refreshes affected parent directories so watcher changes flow to the visible tree.
		 */
		async handleIndexChange(changePath: string, changeType: 'created' | 'changed' | 'deleted') {
			const parentDir = changePath.includes('/')
				? changePath.substring(0, changePath.lastIndexOf('/'))
				: '';

			if (changeType === 'deleted') {
				tree = removeFromTree(tree, changePath);
			}

			// Mark parent as needing refresh and re-fetch if expanded
			tree = markDirNeedsRefresh(tree, parentDir);

			if (expandedDirs.has(parentDir) || parentDir === '') {
				try {
					const response = await listDir(parentDir);
					const children = response.entries.map(entryToTreeNode);
					if (parentDir === '') {
						tree = children;
					} else {
						tree = updateTreeChildren(tree, parentDir, children);
					}
				} catch (err) {
					console.error('[explorer] failed to refresh directory:', parentDir, err);
				}
			}
		},

		/**
		 * Handle a file rename: update the tree entry path.
		 */
		handleRename(oldPath: string, newPath: string) {
			// Remove old, refresh parent dirs of both old and new
			void this.handleIndexChange(oldPath, 'deleted');
			void this.handleIndexChange(newPath, 'created');
		},

		/**
		 * Reset the explorer to empty state (e.g. when switching workspaces).
		 */
		reset() {
			tree = [];
			workspaceName = '';
			notesCount = 0;
			filesCount = 0;
			expandedDirs = new Set<string>();
			selectedFile = null;
			loading = false;
			error = null;
		}
	};
}

/** Find a node in the tree by path. */
function findNode(nodes: TreeNode[], path: string): TreeNode | undefined {
	for (const node of nodes) {
		if (node.path === path) return node;
		if (node.kind === 'dir' && node.children) {
			const found = findNode(node.children, path);
			if (found) return found;
		}
	}
	return undefined;
}

export const explorerStore = createExplorerStore();

// ---------------------------------------------------------------------------
// Search store
// ---------------------------------------------------------------------------

/** A single search result entry. */
export interface SearchResult {
	/** File path relative to workspace root. */
	filePath: string;
	/** Display file name. */
	fileName: string;
	/** 1-based line number of the match. */
	lineNumber: number;
	/** Full text of the matched line. */
	lineContent: string;
}

/**
 * Search store -- currently a placeholder that will be wired to
 * backend search_content IPC once available (Epic 4).
 * Returns empty results until then.
 */
function createSearchStore() {
	let query = $state('');

	const results = $derived.by((): SearchResult[] => {
		// Search will be backed by IPC search_content in Epic 4.
		// For now, return empty results (no mock data).
		if (query.length === 0) return [];
		return [];
	});

	return {
		get query() {
			return query;
		},
		get results(): SearchResult[] {
			return results;
		},

		/** Update the active search query. */
		setQuery(value: string) {
			query = value;
		}
	};
}

export const searchStore = createSearchStore();
