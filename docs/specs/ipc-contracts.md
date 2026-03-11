# IPC Contracts

## Purpose

Define stable frontend-backend contracts between Svelte and Tauri/Rust.
All IPC crosses the Tauri v2 `#[tauri::command]` boundary via `invoke()` on the
frontend and event channels via `listen()`. This document is the single source of
truth for command signatures, response shapes, error codes, and event payloads.

## Versioning

- Contract version: `v1`.
- Backward-compatible additions: new optional fields only.
- Breaking changes: require `v2` namespace and migration notes.
- Event payloads carry `contract_version: "v1"` so consumers can detect
  mismatches during development.
- Command responses use typed payloads and do not embed `contract_version`.

## Command Response Contract

Every `#[tauri::command]` returns `Result<T, VigilError>`.

- Success path: `invoke()` resolves with the serialized command payload `T`
  directly (no backend-provided `ok/data` wrapper).
- Error path: `invoke()` rejects with the serialized `ErrorEnvelope` produced by
  `VigilError -> tauri::ipc::InvokeError`.

Success example (`workspace_status`-style typed payload):
```json
{
  "branch": "main",
  "sync_state": "synced",
  "notes_count": 42,
  "tags_count": 15,
  "files_count": 100,
  "version": "0.0.1",
  "last_index_update_ms": 1700000000000
}
```

Error example:
```json
{
  "code": "WORKSPACE_NOT_OPEN",
  "message": "No workspace is currently open"
}
```

`details` is optional and only present when additional structured context is
provided.

### Error Codes

| Code | Meaning |
|---|---|
| `WORKSPACE_NOT_OPEN` | Command requires an open workspace but none is active |
| `PATH_OUTSIDE_WORKSPACE` | Resolved path escapes workspace root (traversal attempt) |
| `FILE_NOT_FOUND` | Target file or directory does not exist |
| `FILE_ALREADY_EXISTS` | Create/rename target already exists |
| `PERMISSION_DENIED` | OS-level permission failure |
| `INVALID_ARGUMENT` | Malformed or out-of-range parameter |
| `BINARY_FILE` | Operation not supported on binary files |
| `STALE_ETAG` | Optimistic concurrency conflict on write |
| `INDEX_UNAVAILABLE` | File index not yet ready (workspace still scanning) |
| `GIT_UNAVAILABLE` | Workspace is not a git repository or git2 failed |
| `PLUGIN_ERROR` | Plugin runtime fault (Epic 4) |
| `INTERNAL_ERROR` | Unexpected backend failure |

### Payload Size Limits

To prevent UI thread freezes, the backend enforces these limits:

| Limit | Value | Behaviour on exceed |
|---|---|---|
| `read_file` content | 10 MB | Return `INVALID_ARGUMENT` with size in details |
| `list_dir` entries | 50,000 | Truncate and set `truncated: true` in response |
| `fuzzy_find` results | Caller-specified `limit`, max 200 | Clamp silently |
| Event payload | 1 MB | Drop event and log warning |

---

## Commands (MVP)

All path parameters are **workspace-relative** unless stated otherwise. The
backend resolves them against `WorkspaceRoot.canonical_path` and rejects any
path that escapes the root.

### `open_workspace`

Open (or switch to) a workspace directory.

```
open_workspace(root_path: string) -> OpenWorkspaceResponse
```

| Field | Type | Description |
|---|---|---|
| `root_path` | `string` | Absolute filesystem path to workspace root |

**Response: `OpenWorkspaceResponse`**

| Field | Type | Description |
|---|---|---|
| `workspace_id` | `string` | Stable hash of canonical root path |
| `canonical_path` | `string` | Canonicalized absolute path |
| `notes_count` | `number` | Number of `.md` files found |
| `files_count` | `number` | Total indexed file count |
| `opened_at_ms` | `number` | Unix epoch ms when workspace was opened |

**Errors:** `INVALID_ARGUMENT` (path does not exist or is a file), `PERMISSION_DENIED`.

**Notes:**
- Root path is canonicalized (symlinks resolved, `..` collapsed) before use.
- Triggers background index scan; `index://ready` event fires when complete.
- Previous workspace state is discarded on switch.

**Performance:** <= 1500 ms to first usable tree for 10k files (see performance budget).

---

### `list_dir`

List entries in a workspace directory.

```
list_dir(path: string) -> ListDirResponse
```

| Field | Type | Description |
|---|---|---|
| `path` | `string` | Workspace-relative directory path; `""` for root |

**Response: `ListDirResponse`**

| Field | Type | Description |
|---|---|---|
| `entries` | `DirEntry[]` | Sorted directory entries |
| `truncated` | `boolean` | `true` if entry count exceeded limit |

**`DirEntry`**

| Field | Type | Description |
|---|---|---|
| `name` | `string` | File or directory name |
| `path` | `string` | Workspace-relative path |
| `kind` | `"file" \| "dir"` | Entry type |
| `ext` | `string \| null` | File extension without dot, or `null` |
| `size_bytes` | `number \| null` | File size (null for directories) |
| `modified_at_ms` | `number \| null` | Last modification unix epoch ms |
| `is_hidden` | `boolean` | Starts with `.` or OS hidden attribute |

**Errors:** `WORKSPACE_NOT_OPEN`, `FILE_NOT_FOUND`, `PATH_OUTSIDE_WORKSPACE`.

**Notes:**
- Entries are sorted: directories first, then alphabetical (case-insensitive).
- Honors ignore policy (`.git/`, `node_modules/`, etc. filtered out).
- Frontend uses this for explorer tree expansion (lazy per-directory).

---

### `read_file`

Read a text file's content and metadata.

```
read_file(path: string) -> ReadFileResponse
```

| Field | Type | Description |
|---|---|---|
| `path` | `string` | Workspace-relative file path |

**Response: `ReadFileResponse`**

| Field | Type | Description |
|---|---|---|
| `content` | `string` | UTF-8 text content |
| `encoding` | `string` | Detected encoding (always `"utf-8"` for MVP) |
| `size_bytes` | `number` | File size in bytes |
| `modified_at_ms` | `number` | Last modification unix epoch ms |
| `etag` | `string` | Content hash for optimistic concurrency |

**Errors:** `WORKSPACE_NOT_OPEN`, `FILE_NOT_FOUND`, `PATH_OUTSIDE_WORKSPACE`, `BINARY_FILE`, `INVALID_ARGUMENT` (file too large).

**Notes:**
- Binary files are detected by null-byte scan of the first 8 KB. Binary files
  return `BINARY_FILE` error.
- `etag` is a hex-encoded hash of content bytes. Used by `write_file` for
  conflict detection.

**Performance:** <= 120 ms for indexed files (file switch budget).

---

### `write_file`

Write content to a file, optionally with optimistic concurrency check.

```
write_file(path: string, content: string, etag?: string) -> WriteFileResponse
```

| Field | Type | Description |
|---|---|---|
| `path` | `string` | Workspace-relative file path |
| `content` | `string` | New UTF-8 file content |
| `etag` | `string \| null` | Expected etag; if set, write fails on mismatch |

**Response: `WriteFileResponse`**

| Field | Type | Description |
|---|---|---|
| `size_bytes` | `number` | New file size |
| `modified_at_ms` | `number` | New modification timestamp |
| `etag` | `string` | Updated content hash |

**Errors:** `WORKSPACE_NOT_OPEN`, `PATH_OUTSIDE_WORKSPACE`, `PERMISSION_DENIED`, `STALE_ETAG`.

**Notes:**
- Creates the file if it does not exist (parent directories are not auto-created).
- When `etag` is provided and does not match the current file hash, returns
  `STALE_ETAG` to signal a concurrent edit. The frontend should prompt the user
  to reload or force-save.
- Emits `index://updated` event with `change_type: "changed"` or `"created"`.

---

### `create_note`

Create a new markdown note file.

```
create_note(path: string) -> CreateNoteResponse
```

| Field | Type | Description |
|---|---|---|
| `path` | `string` | Workspace-relative path for the new note |

**Response: `CreateNoteResponse`**

| Field | Type | Description |
|---|---|---|
| `path` | `string` | Actual workspace-relative path (may have `.md` appended) |
| `size_bytes` | `number` | File size (0 for empty) |
| `modified_at_ms` | `number` | Creation timestamp |
| `etag` | `string` | Content hash |

**Errors:** `WORKSPACE_NOT_OPEN`, `PATH_OUTSIDE_WORKSPACE`, `FILE_ALREADY_EXISTS`, `PERMISSION_DENIED`.

**Notes:**
- If `path` has no extension, `.md` is appended automatically.
- Creates parent directories as needed.
- File is created with an empty frontmatter template if configured in settings.
- Emits `index://updated` event with `change_type: "created"`.

---

### `rename_file`

Rename or move a file within the workspace.

```
rename_file(old_path: string, new_path: string) -> RenameFileResponse
```

| Field | Type | Description |
|---|---|---|
| `old_path` | `string` | Current workspace-relative path |
| `new_path` | `string` | Target workspace-relative path |

**Response: `RenameFileResponse`**

| Field | Type | Description |
|---|---|---|
| `path` | `string` | New workspace-relative path |
| `modified_at_ms` | `number` | Timestamp after rename |

**Errors:** `WORKSPACE_NOT_OPEN`, `FILE_NOT_FOUND`, `FILE_ALREADY_EXISTS`, `PATH_OUTSIDE_WORKSPACE`, `PERMISSION_DENIED`.

**Notes:**
- Emits `index://updated` events for both the old path (`"deleted"`) and the new
  path (`"created"`).
- Link graph and backlink index are invalidated for affected paths.

---

### `delete_file`

Delete a file or empty directory within the workspace.

```
delete_file(path: string) -> DeleteFileResponse
```

| Field | Type | Description |
|---|---|---|
| `path` | `string` | Workspace-relative path to delete |

**Response: `DeleteFileResponse`**

| Field | Type | Description |
|---|---|---|
| `path` | `string` | Deleted workspace-relative path |

**Errors:** `WORKSPACE_NOT_OPEN`, `FILE_NOT_FOUND`, `PATH_OUTSIDE_WORKSPACE`, `PERMISSION_DENIED`, `INVALID_ARGUMENT` (non-empty directory).

**Notes:**
- Non-empty directories are rejected to prevent accidental data loss.
- Emits `index://updated` with `change_type: "deleted"`.

---

### `fuzzy_find`

Search workspace file index by fuzzy filename matching.

```
fuzzy_find(query: string, limit: number) -> FuzzyFindResponse
```

| Field | Type | Description |
|---|---|---|
| `query` | `string` | User input string |
| `limit` | `number` | Maximum results (clamped to 200) |

**Response: `FuzzyFindResponse`**

| Field | Type | Description |
|---|---|---|
| `matches` | `FuzzyMatch[]` | Ranked results, best first |

**`FuzzyMatch`**

| Field | Type | Description |
|---|---|---|
| `path` | `string` | Workspace-relative file path |
| `display` | `string` | Formatted display string for omnibar |
| `score` | `number` | Match score (higher is better) |
| `kind` | `"file" \| "dir"` | Entry type |
| `matched_indices` | `number[]` | Character positions that matched in `display` |

**Errors:** `WORKSPACE_NOT_OPEN`, `INDEX_UNAVAILABLE`.

**Notes:**
- Backed by `nucleo-matcher` for fast fuzzy scoring.
- Used by `Ctrl+P` omnibar in filename mode.
- Empty query returns recently opened files (up to `limit`).

**Performance:** <= 80 ms first result render.

---

### `get_git_hunks`

Get line-level diff hunks for a file against the git HEAD.

```
get_git_hunks(path: string) -> GitHunksResponse
```

| Field | Type | Description |
|---|---|---|
| `path` | `string` | Workspace-relative file path |

**Response: `GitHunksResponse`**

| Field | Type | Description |
|---|---|---|
| `hunks` | `GitHunk[]` | List of diff hunks |

**`GitHunk`**

| Field | Type | Description |
|---|---|---|
| `change_type` | `"added" \| "modified" \| "deleted"` | Hunk classification |
| `start_line` | `number` | First line of hunk in working copy (1-based) |
| `end_line` | `number` | Last line of hunk in working copy (1-based) |
| `base_start_line` | `number \| null` | First line in base (null for additions) |
| `base_end_line` | `number \| null` | Last line in base (null for additions) |

**Errors:** `WORKSPACE_NOT_OPEN`, `FILE_NOT_FOUND`, `GIT_UNAVAILABLE`.

**Notes:**
- Returns empty list for untracked files.
- Used for editor gutter decorations.

**Performance:** <= 200 ms for files under 5k lines.

---

### `get_backlinks`

Get notes that link to the specified file.

```
get_backlinks(path: string) -> BacklinksResponse
```

| Field | Type | Description |
|---|---|---|
| `path` | `string` | Workspace-relative target file path |

**Response: `BacklinksResponse`**

| Field | Type | Description |
|---|---|---|
| `backlinks` | `BacklinkRecord[]` | Notes linking to this file |

**`BacklinkRecord`**

| Field | Type | Description |
|---|---|---|
| `source_path` | `string` | Workspace-relative path of the linking note |
| `target_path` | `string` | Workspace-relative path of the linked file |
| `context_snippet` | `string` | Line containing the link (truncated to 200 chars) |

**Errors:** `WORKSPACE_NOT_OPEN`, `INDEX_UNAVAILABLE`.

**Notes:**
- Scans `[[wikilink]]` and standard markdown `[text](path)` link syntax.
- Results update when index processes file changes.

---

### `workspace_status`

Get current workspace status for the footer status bar.

```
workspace_status() -> WorkspaceStatusResponse
```

No parameters.

**Response: `WorkspaceStatusResponse`**

| Field | Type | Description |
|---|---|---|
| `branch` | `string \| null` | Current git branch name, or null if not a git repo |
| `sync_state` | `string` | One of `"synced"`, `"ahead"`, `"behind"`, `"diverged"`, `"unknown"` |
| `notes_count` | `number` | Number of `.md` files in workspace |
| `tags_count` | `number` | Number of unique tags across all notes |
| `files_count` | `number` | Total indexed file count |
| `version` | `string` | Application version string |
| `last_index_update_ms` | `number` | Timestamp of last index refresh |

**Errors:** `WORKSPACE_NOT_OPEN`.

**Notes:**
- `sync_state` compares local branch to its upstream tracking branch via git2.
  Falls back to `"unknown"` if no remote is configured.
- Used by the footer status bar (Task 2.7/3.7).

---

## Commands (Epic 4 -- Post-MVP)

These commands are defined here for forward compatibility but are not part of the
MVP contract. Implementations should return `INTERNAL_ERROR` with message
"Not implemented" until the corresponding epic task is complete.

### `search_content`

Full-text content search with snippet extraction.

```
search_content(query: string, limit: number) -> SearchContentResponse
```

**Response: `SearchContentResponse`**

| Field | Type | Description |
|---|---|---|
| `matches` | `ContentMatch[]` | Ranked snippet matches |

**`ContentMatch`**

| Field | Type | Description |
|---|---|---|
| `path` | `string` | Workspace-relative file path |
| `line_number` | `number` | 1-based line number |
| `line_start_col` | `number` | Start column of match |
| `line_end_col` | `number` | End column of match |
| `preview` | `string` | Context line(s) around match |
| `score` | `number` | Relevance score |

---

### `get_note_graph`

Get the full link graph for visualization.

```
get_note_graph() -> NoteGraphResponse
```

**Response: `NoteGraphResponse`**

| Field | Type | Description |
|---|---|---|
| `nodes` | `NoteNode[]` | All notes as graph nodes |
| `edges` | `LinkEdge[]` | All links as graph edges |

See Workspace Data Model for `NoteNode` and `LinkEdge` schemas.

---

### `plugin_list`

List installed plugins with their runtime state.

```
plugin_list() -> PluginListResponse
```

**Response:** Array of `PluginManifest` merged with `PluginRuntimeState`.

---

### `plugin_install`

Install a plugin from a local source.

```
plugin_install(source_path: string) -> PluginInstallResponse
```

**Response:** Installed manifest + capability summary.

---

### `plugin_enable`

Enable or disable an installed plugin.

```
plugin_enable(plugin_id: string, enabled: boolean) -> PluginEnableResponse
```

**Response:** Updated `PluginRuntimeState`.

---

## Event Channels

Events are emitted from the Rust backend via `app_handle.emit()` and consumed
on the frontend via `listen()`. All payloads include common metadata fields.

### Common Event Metadata

Every event payload includes:

| Field | Type | Description |
|---|---|---|
| `timestamp_ms` | `number` | Unix epoch ms when event was emitted |
| `contract_version` | `string` | Always `"v1"` |

---

### `vigil://index-updated`

Emitted when the file index detects filesystem changes.

| Field | Type | Description |
|---|---|---|
| `changes` | `IndexChange[]` | List of changed entries |

**`IndexChange`**

| Field | Type | Description |
|---|---|---|
| `path` | `string` | Workspace-relative path |
| `change_type` | `"created" \| "changed" \| "deleted"` | What happened |
| `kind` | `"file" \| "dir"` | Entry type |

**Frontend action:** Refresh explorer tree, invalidate file caches, update note/file counts.

---

### `vigil://index-ready`

Emitted once when the initial workspace index scan completes.

| Field | Type | Description |
|---|---|---|
| `files_count` | `number` | Total files indexed |
| `notes_count` | `number` | Markdown files indexed |
| `duration_ms` | `number` | Time taken for initial scan |

**Frontend action:** Enable search, show file counts, hide loading indicators.

---

### `vigil://git-hunks`

Emitted when git diff state changes for a file (after save or external change).

| Field | Type | Description |
|---|---|---|
| `path` | `string` | Workspace-relative file path |
| `hunks` | `GitHunk[]` | Updated hunk list |

**Frontend action:** Update editor gutter decorations.

---

### `vigil://status-updated`

Emitted when workspace status changes (branch switch, index update, etc.).

| Field | Type | Description |
|---|---|---|
| `status` | `WorkspaceStatusResponse` | Full status snapshot |

**Frontend action:** Refresh footer status bar segments.

---

### `vigil://fs-renamed`

Emitted when a file is renamed or moved within the workspace.

| Field | Type | Description |
|---|---|---|
| `old_path` | `string` | Previous workspace-relative path |
| `new_path` | `string` | New workspace-relative path |

**Frontend action:** Update open editor tabs, explorer tree, and breadcrumbs.

---

### `vigil://plugins-updated` (Epic 4)

Emitted on plugin install, enable/disable, or runtime state change.

| Field | Type | Description |
|---|---|---|
| `plugin_id` | `string` | Affected plugin identifier |
| `event_type` | `"installed" \| "removed" \| "enabled" \| "disabled" \| "error"` | What changed |
| `state` | `PluginRuntimeState \| null` | Current state, null if removed |

**Frontend action:** Refresh plugin management panel.

---

## Event Channel Naming Convention

All Vigil events use the `vigil://` prefix to avoid collisions with Tauri
system events. Format: `vigil://<domain>-<action>`.

---

## Frontend IPC Wrapper Convention

Frontend code must never call `invoke()` or `listen()` directly. All IPC goes
through typed wrapper modules:

| Module | Commands Covered |
|---|---|
| `src/lib/ipc/files.ts` | `open_workspace`, `list_dir`, `read_file`, `write_file`, `create_note`, `rename_file`, `delete_file` |
| `src/lib/ipc/search.ts` | `fuzzy_find`, `search_content` (Epic 4) |
| `src/lib/ipc/git.ts` | `get_git_hunks` |
| `src/lib/ipc/links.ts` | `get_backlinks`, `get_note_graph` (Epic 4) |
| `src/lib/ipc/status.ts` | `workspace_status` |
| `src/lib/ipc/events.ts` | All `listen()` subscriptions |
| `src/lib/ipc/plugins.ts` | `plugin_list`, `plugin_install`, `plugin_enable` (Epic 4) |

---

## Type Ownership

- **Rust source of truth:** `src-tauri/src/models/*.rs`
- **Frontend mirrors:** `src/lib/types/*.ts`
- **Rule:** Frontend types must be manually synced from Rust models on each
  contract change. A future task may generate TS types from Rust via `ts-rs`.

---

## Validation Rules

1. Every command validates canonical path confinement to workspace root before
   any filesystem operation.
2. Every command enforces payload size limits (see table above).
3. All event payloads include `timestamp_ms` and `contract_version`.
4. Path parameters are normalized (forward slashes, no trailing slash, no `..`).
5. String parameters are trimmed; empty strings where not meaningful return
   `INVALID_ARGUMENT`.

---

## Change Control

Any contract change requires synchronized updates in:

1. This document (`docs/specs/ipc-contracts.md`)
2. Rust models (`src-tauri/src/models/*.rs`)
3. Rust command handlers (`src-tauri/src/commands/*.rs`)
4. Frontend IPC wrappers (`src/lib/ipc/*.ts`)
5. Frontend type definitions (`src/lib/types/*.ts`)
6. Integration tests (`src-tauri/tests/*.rs`)

A contract change PR must reference the updated section of this document in its
description.
