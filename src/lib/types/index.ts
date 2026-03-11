// Barrel file for shared TypeScript types.

// Store (UI-side) types
export type {
	DirEntry,
	GitHunk,
	SyncState,
	GitStatusEntry,
	OpenFile,
	EditorState,
	FilesState,
	GitState,
	SettingsState,
	SidebarSection,
	UiState
} from './store';

// IPC contract types (backend-mirror, prefixed where they'd collide with store types)
export type {
	ErrorCode,
	VigilError,
	WorkspaceRoot,
	OpenWorkspaceResponse,
	EntryKind,
	FileEntry,
	DirEntry as IpcDirEntry,
	ListDirResponse,
	ReadFileResponse,
	WriteFileRequest,
	WriteFileResponse,
	CreateNoteResponse,
	RenameFileResponse,
	DeleteFileResponse,
	FuzzyMatch,
	FuzzyFindResponse,
	ContentMatch,
	SearchContentResponse,
	Tag,
	HunkChangeType,
	GitHunk as IpcGitHunk,
	GitHunksResponse,
	GitFileStatus,
	GitStatusEntry as IpcGitStatusEntry,
	BacklinkRecord,
	BacklinksResponse,
	SyncState as IpcSyncState,
	WorkspaceStatus,
	EventMeta,
	IndexChangeType,
	IndexChange,
	IndexUpdatedPayload,
	IndexReadyPayload,
	GitHunksPayload,
	StatusUpdatedPayload
} from './ipc';
