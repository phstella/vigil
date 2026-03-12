/**
 * Explorer UI state store.
 *
 * Provides mock file-tree data and local UI state (selection, expansion)
 * for the explorer panel. This will be replaced by real backend data
 * once IPC is wired up.
 */

import type { DirEntry } from '$lib/types/store';
import { editorStore } from '$lib/stores/editor';
import { isMarkdownFile } from '$lib/utils/file-routing';
import { detectLanguage } from '$lib/features/editor/code-store';
import { uiStore } from '$lib/stores/ui';

/** A tree node extends DirEntry with optional children for recursive rendering. */
export interface TreeNode extends DirEntry {
	children?: TreeNode[];
}

export interface CollectionSummary {
	name: string;
	path: string;
	filesCount: number;
	notesCount: number;
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
	/** Top-level collection summaries for explorer header display. */
	collections: CollectionSummary[];
	/** Set of expanded directory paths. */
	expandedDirs: Set<string>;
	/** Currently selected file path, or null. */
	selectedFile: string | null;
}

function createExplorerStore() {
	const tree: TreeNode[] = MOCK_TREE;
	const workspaceName: string = 'My Workspace';
	const notesCount = tree.reduce((sum, node) => sum + summarizeNode(node).notesCount, 0);
	const collections: CollectionSummary[] = tree
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
	let expandedDirs = $state<Set<string>>(new Set<string>());
	let selectedFile = $state<string | null>(null);

	return {
		get tree() {
			return tree;
		},
		get workspaceName() {
			return workspaceName;
		},
		get notesCount() {
			return notesCount;
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

		/** Select a file by path and open it in the appropriate editor pane. */
		selectFile(path: string) {
			selectedFile = path;

			// Route to the correct editor pane.
			// Content is a placeholder until real IPC is wired (task 3.5+).
			const language = detectLanguage(path);
			const mockContent = isMarkdownFile(path)
				? `# ${path.split('/').pop()?.replace(/\.md$/, '') ?? 'Untitled'}\n\nStart writing...`
				: `// ${path.split('/').pop() ?? 'file'}\n`;
			editorStore.openFile(path, mockContent, language);

			// Show the right panel when opening a code file
			if (!isMarkdownFile(path)) {
				uiStore.openRightPanel();
			}
		},

		/** Check if a directory is expanded. */
		isExpanded(path: string): boolean {
			return expandedDirs.has(path);
		}
	};
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
 * Mock search results -- realistic content matches across workspace files.
 * Will be replaced by real backend search_content results once IPC is wired.
 */
const MOCK_SEARCH_RESULTS: SearchResult[] = [
	{
		filePath: '/workspace/notes/projects/vigil-roadmap.md',
		fileName: 'vigil-roadmap.md',
		lineNumber: 12,
		lineContent: 'Implement the search panel with debounced input and result highlighting.'
	},
	{
		filePath: '/workspace/notes/projects/design-system.md',
		fileName: 'design-system.md',
		lineNumber: 34,
		lineContent: 'The search component uses a monospace font for snippet display.'
	},
	{
		filePath: '/workspace/notes/daily/2026-03-11.md',
		fileName: '2026-03-11.md',
		lineNumber: 5,
		lineContent: 'Worked on search functionality and sidebar integration today.'
	},
	{
		filePath: '/workspace/notes/index.md',
		fileName: 'index.md',
		lineNumber: 8,
		lineContent: 'Use the search panel to find notes across the workspace quickly.'
	},
	{
		filePath: '/workspace/scripts/build.ts',
		fileName: 'build.ts',
		lineNumber: 22,
		lineContent: 'const searchPaths = glob.sync("**/*.md", { cwd: root });'
	},
	{
		filePath: '/workspace/README.md',
		fileName: 'README.md',
		lineNumber: 15,
		lineContent: 'Full-text search is available via the sidebar search panel or Ctrl+Shift+F.'
	},
	{
		filePath: '/workspace/notes/quick-capture.md',
		fileName: 'quick-capture.md',
		lineNumber: 3,
		lineContent: 'Quick search: remember to review the roadmap before standup.'
	},
	{
		filePath: '/workspace/templates/note-template.md',
		fileName: 'note-template.md',
		lineNumber: 1,
		lineContent: '# {{title}} -- searchable note template'
	}
];

function createSearchStore() {
	let query = $state('');

	const results = $derived.by(() => {
		if (query.length === 0) return [];
		const lower = query.toLowerCase();
		return MOCK_SEARCH_RESULTS.filter(
			(r) =>
				r.lineContent.toLowerCase().includes(lower) ||
				r.fileName.toLowerCase().includes(lower) ||
				r.filePath.toLowerCase().includes(lower)
		);
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
