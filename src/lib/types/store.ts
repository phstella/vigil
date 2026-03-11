/**
 * TypeScript interfaces for frontend store state shapes.
 *
 * These mirror the workspace data model (docs/specs/workspace-data-model.md)
 * where applicable, but represent *UI-side* state rather than backend models.
 */

// ---------------------------------------------------------------------------
// Shared / Domain types used across stores
// ---------------------------------------------------------------------------

/** Subset of FileEntry sent from the backend for directory listings. */
export interface DirEntry {
	name: string;
	path: string;
	kind: 'file' | 'dir';
	ext: string | null;
	size_bytes: number | null;
	modified_at_ms: number | null;
	is_hidden: boolean;
}

/** A line-level diff hunk from git. */
export interface GitHunk {
	change_type: 'added' | 'modified' | 'deleted';
	start_line: number;
	end_line: number;
	base_start_line: number | null;
	base_end_line: number | null;
}

/** Git sync state relative to remote. */
export type SyncState = 'synced' | 'ahead' | 'behind' | 'diverged' | 'unknown';

/** A git status entry for a single file. */
export interface GitStatusEntry {
	path: string;
	status: 'added' | 'modified' | 'deleted' | 'renamed' | 'untracked';
}

// ---------------------------------------------------------------------------
// Editor store
// ---------------------------------------------------------------------------

/** Represents a single open file tab. */
export interface OpenFile {
	path: string;
	isDirty: boolean;
	language: string;
}

export interface EditorState {
	/** Workspace-relative path of the currently active file, or null. */
	activeFile: string | null;
	/** Ordered list of open file tabs. */
	openFiles: OpenFile[];
	/** Whether the active file has unsaved changes. */
	isDirty: boolean;
	/** Text content of the active file. */
	content: string;
	/** Detected language / syntax mode of the active file. */
	language: string;
}

// ---------------------------------------------------------------------------
// Files store
// ---------------------------------------------------------------------------

export interface FilesState {
	/** Absolute path to the workspace root, or null if none opened. */
	workspaceRoot: string | null;
	/** Top-level directory entries (lazy-loaded per folder). */
	fileTree: DirEntry[];
	/** Set of expanded directory paths in the explorer. */
	expandedDirs: Set<string>;
}

// ---------------------------------------------------------------------------
// Git store
// ---------------------------------------------------------------------------

export interface GitState {
	/** Current branch name, or null if not a git repo. */
	branch: string | null;
	/** Sync state relative to remote. */
	syncState: SyncState;
	/** Map of workspace-relative file path to its diff hunks. */
	hunks: Map<string, GitHunk[]>;
	/** List of files with uncommitted changes. */
	statusEntries: GitStatusEntry[];
}

// ---------------------------------------------------------------------------
// Settings store
// ---------------------------------------------------------------------------

export interface SettingsState {
	/** Active colour theme identifier. */
	theme: string;
	/** Editor font size in pixels. */
	fontSize: number;
	/** Editor font family CSS value. */
	fontFamily: string;
	/** Sidebar width in pixels. */
	sidebarWidth: number;
	/** Whether the editor wraps long lines. */
	editorWordWrap: boolean;
}

// ---------------------------------------------------------------------------
// UI store
// ---------------------------------------------------------------------------

/** Sidebar panel identifiers. */
export type SidebarSection = 'explorer' | 'search' | 'graph' | 'tags';

export interface UiState {
	/** Whether the sidebar is visible. */
	sidebarOpen: boolean;
	/** Which sidebar panel is active. */
	sidebarSection: SidebarSection;
	/** Whether the omnibar overlay is open. */
	omnibarOpen: boolean;
	/** Whether the right (code) panel is visible. */
	rightPanelOpen: boolean;
}
