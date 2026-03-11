/**
 * Explorer UI state store.
 *
 * Provides mock file-tree data and local UI state (selection, expansion)
 * for the explorer panel. This will be replaced by real backend data
 * once IPC is wired up.
 */

import type { DirEntry } from '$lib/types/store';

/** A tree node extends DirEntry with optional children for recursive rendering. */
export interface TreeNode extends DirEntry {
	children?: TreeNode[];
}

// ---------------------------------------------------------------------------
// Mock data -- realistic workspace tree
// ---------------------------------------------------------------------------

function file(
	name: string,
	path: string,
	ext: string | null = null,
	size: number = 1024
): TreeNode {
	return {
		name,
		path,
		kind: 'file',
		ext,
		size_bytes: size,
		modified_at_ms: Date.now(),
		is_hidden: false
	};
}

function dir(name: string, path: string, children: TreeNode[] = []): TreeNode {
	return {
		name,
		path,
		kind: 'dir',
		ext: null,
		size_bytes: null,
		modified_at_ms: Date.now(),
		is_hidden: false,
		children
	};
}

const MOCK_TREE: TreeNode[] = [
	dir('notes', '/workspace/notes', [
		dir('daily', '/workspace/notes/daily', [
			file('2026-03-11.md', '/workspace/notes/daily/2026-03-11.md', 'md', 2048),
			file('2026-03-10.md', '/workspace/notes/daily/2026-03-10.md', 'md', 1536),
			file('2026-03-09.md', '/workspace/notes/daily/2026-03-09.md', 'md', 980)
		]),
		dir('projects', '/workspace/notes/projects', [
			file('vigil-roadmap.md', '/workspace/notes/projects/vigil-roadmap.md', 'md', 4096),
			file('design-system.md', '/workspace/notes/projects/design-system.md', 'md', 3200)
		]),
		file('index.md', '/workspace/notes/index.md', 'md', 512),
		file('quick-capture.md', '/workspace/notes/quick-capture.md', 'md', 256)
	]),
	dir('scripts', '/workspace/scripts', [
		file('build.ts', '/workspace/scripts/build.ts', 'ts', 1200),
		file('deploy.sh', '/workspace/scripts/deploy.sh', 'sh', 800)
	]),
	dir('templates', '/workspace/templates', [
		file('note-template.md', '/workspace/templates/note-template.md', 'md', 340),
		file('meeting-template.md', '/workspace/templates/meeting-template.md', 'md', 520)
	]),
	file('README.md', '/workspace/README.md', 'md', 2400),
	file('.vigilrc', '/workspace/.vigilrc', null, 128)
];

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

export interface ExplorerState {
	/** Root tree nodes. */
	tree: TreeNode[];
	/** Workspace display name. */
	workspaceName: string;
	/** Set of expanded directory paths. */
	expandedDirs: Set<string>;
	/** Currently selected file path, or null. */
	selectedFile: string | null;
}

function createExplorerStore() {
	const tree: TreeNode[] = MOCK_TREE;
	const workspaceName: string = 'My Workspace';
	let expandedDirs = $state<Set<string>>(new Set<string>());
	let selectedFile = $state<string | null>(null);

	return {
		get tree() {
			return tree;
		},
		get workspaceName() {
			return workspaceName;
		},
		get expandedDirs() {
			return expandedDirs;
		},
		get selectedFile() {
			return selectedFile;
		},

		/** Toggle a directory's expanded/collapsed state. */
		toggleExpand(path: string) {
			const next = new Set(expandedDirs);
			if (next.has(path)) {
				next.delete(path);
			} else {
				next.add(path);
			}
			expandedDirs = next;
		},

		/** Select a file by path. */
		selectFile(path: string) {
			selectedFile = path;
		},

		/** Check if a directory is expanded. */
		isExpanded(path: string): boolean {
			return expandedDirs.has(path);
		}
	};
}

export const explorerStore = createExplorerStore();
