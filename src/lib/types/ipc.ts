// TypeScript interfaces mirroring Rust IPC models.
// Source of truth: src-tauri/src/models/*.rs + docs/specs/ipc-contracts.md

// ---------------------------------------------------------------------------
// Error envelope
// ---------------------------------------------------------------------------

/** Error codes matching the IPC contract error table. */
export type ErrorCode =
	| 'WORKSPACE_NOT_OPEN'
	| 'PATH_OUTSIDE_WORKSPACE'
	| 'FILE_NOT_FOUND'
	| 'FILE_ALREADY_EXISTS'
	| 'PERMISSION_DENIED'
	| 'INVALID_ARGUMENT'
	| 'BINARY_FILE'
	| 'STALE_ETAG'
	| 'INDEX_UNAVAILABLE'
	| 'GIT_UNAVAILABLE'
	| 'PLUGIN_ERROR'
	| 'INTERNAL_ERROR';

/** Structured error envelope returned by failed invoke() calls. */
export interface VigilError {
	code: ErrorCode;
	message: string;
	details?: unknown;
}

// ---------------------------------------------------------------------------
// Workspace
// ---------------------------------------------------------------------------

export interface WorkspaceRoot {
	workspace_id: string;
	canonical_path: string;
	opened_at_ms: number;
}

export interface OpenWorkspaceResponse {
	workspace_id: string;
	canonical_path: string;
	notes_count: number;
	files_count: number;
	opened_at_ms: number;
}

// ---------------------------------------------------------------------------
// Files
// ---------------------------------------------------------------------------

export type EntryKind = 'file' | 'dir';

export interface FileEntry {
	path: string;
	name: string;
	kind: EntryKind;
	ext: string | null;
	size_bytes: number;
	modified_at_ms: number;
	is_hidden: boolean;
	is_binary: boolean;
}

export interface DirEntry {
	name: string;
	path: string;
	kind: EntryKind;
	ext: string | null;
	size_bytes: number | null;
	modified_at_ms: number | null;
	is_hidden: boolean;
}

export interface ListDirResponse {
	entries: DirEntry[];
	truncated: boolean;
}

export interface ReadFileResponse {
	content: string;
	encoding: string;
	size_bytes: number;
	modified_at_ms: number;
	etag: string;
}

export interface WriteFileRequest {
	path: string;
	content: string;
	etag?: string | null;
}

export interface WriteFileResponse {
	size_bytes: number;
	modified_at_ms: number;
	etag: string;
}

export interface CreateNoteResponse {
	path: string;
	size_bytes: number;
	modified_at_ms: number;
	etag: string;
}

export interface RenameFileResponse {
	path: string;
	modified_at_ms: number;
}

export interface DeleteFileResponse {
	path: string;
}

// ---------------------------------------------------------------------------
// Search
// ---------------------------------------------------------------------------

export interface FuzzyMatch {
	path: string;
	display: string;
	score: number;
	kind: EntryKind;
	matched_indices: number[];
}

export interface FuzzyFindResponse {
	matches: FuzzyMatch[];
}

export interface ContentMatch {
	path: string;
	line_number: number;
	line_start_col: number;
	line_end_col: number;
	preview: string;
	score: number;
}

export interface SearchContentResponse {
	matches: ContentMatch[];
}

export interface Tag {
	name: string;
	count: number;
	files: string[];
}

// ---------------------------------------------------------------------------
// Git
// ---------------------------------------------------------------------------

export type HunkChangeType = 'added' | 'modified' | 'deleted';

export interface GitHunk {
	change_type: HunkChangeType;
	start_line: number;
	end_line: number;
	base_start_line: number | null;
	base_end_line: number | null;
}

export interface GitHunksResponse {
	hunks: GitHunk[];
}

export type GitFileStatus =
	| 'clean'
	| 'modified'
	| 'new'
	| 'deleted'
	| 'renamed'
	| 'conflicted'
	| 'unknown';

export interface GitStatusEntry {
	path: string;
	status: GitFileStatus;
}

// ---------------------------------------------------------------------------
// Links
// ---------------------------------------------------------------------------

export interface BacklinkRecord {
	source_path: string;
	target_path: string;
	context_snippet: string;
}

export interface BacklinksResponse {
	backlinks: BacklinkRecord[];
}

// ---------------------------------------------------------------------------
// Graph
// ---------------------------------------------------------------------------

/** A node in the note link graph. */
export interface NoteNode {
	id: string;
	path: string;
	title: string;
	tags: string[];
}

/** A directed edge in the note link graph. */
export interface LinkEdge {
	from_node_id: string;
	to_node_id: string;
	kind: 'wikilink' | 'markdown';
}

/** Response from get_note_graph. */
export interface NoteGraphResponse {
	nodes: NoteNode[];
	edges: LinkEdge[];
}

// ---------------------------------------------------------------------------
// Status
// ---------------------------------------------------------------------------

export type SyncState = 'synced' | 'ahead' | 'behind' | 'diverged' | 'unknown';

export interface WorkspaceStatus {
	branch: string | null;
	sync_state: SyncState;
	notes_count: number;
	tags_count: number;
	files_count: number;
	version: string;
	last_index_update_ms: number;
}

// ---------------------------------------------------------------------------
// Event payloads
// ---------------------------------------------------------------------------

/** Common metadata included in every event payload. */
export interface EventMeta {
	timestamp_ms: number;
	contract_version: string;
}

export type IndexChangeType = 'created' | 'changed' | 'deleted';

export interface IndexChange {
	path: string;
	change_type: IndexChangeType;
	kind: string;
}

export interface IndexUpdatedPayload extends EventMeta {
	changes: IndexChange[];
}

export interface IndexReadyPayload extends EventMeta {
	files_count: number;
	notes_count: number;
	duration_ms: number;
}

export interface GitHunksPayload extends EventMeta {
	path: string;
	hunks: GitHunk[];
}

export interface StatusUpdatedPayload extends EventMeta {
	status: WorkspaceStatus;
}

export interface FsRenamedPayload extends EventMeta {
	old_path: string;
	new_path: string;
}
