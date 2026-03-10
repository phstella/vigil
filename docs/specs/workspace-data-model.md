# Workspace Data Model

## Scope

Define canonical data structures for workspace state, files, notes, tags, links,
search results, git hunks, and cache invalidation. These models are the Rust
source of truth (`src-tauri/src/models/`) and are mirrored to TypeScript
(`src/lib/types/`) for frontend consumption.

All types use `snake_case` field names for consistency with Rust/serde
serialization.

---

## Workspace

### WorkspaceRoot

The top-level container representing an opened workspace directory.

| Field | Type | Description |
|---|---|---|
| `workspace_id` | `string` | Stable hash of `canonical_path` (deterministic across sessions) |
| `canonical_path` | `string` | Absolute canonical path to workspace root |
| `opened_at_ms` | `number` | Unix epoch ms when workspace was opened |

**Invariants:**
- All file operations resolve against `canonical_path` and reject traversal
  outside root.
- `workspace_id` is derived as a hex-encoded hash (e.g., blake3 or sha256) of
  the canonical path string, ensuring stability across restarts.

### Vault

A vault is the logical namespace for a workspace. In MVP, there is a 1:1
mapping between a vault and a workspace root. Post-MVP, multiple vaults
could be supported.

| Field | Type | Description |
|---|---|---|
| `workspace_id` | `string` | References `WorkspaceRoot.workspace_id` |
| `name` | `string` | Display name (derived from directory name) |
| `notes_count` | `number` | Count of `.md` files |
| `files_count` | `number` | Total indexed file count |
| `tags_count` | `number` | Unique tag count across all notes |

**Notes:**
- The vault concept exists to future-proof multi-root workspaces (e.g., a
  "Projects" vault and a "Journal" vault in one window). For MVP, the vault
  is identical to the workspace.

---

## File Index

### FileEntry

Represents a single file or directory in the workspace index.

| Field | Type | Description |
|---|---|---|
| `path` | `string` | Workspace-relative path (forward slashes) |
| `absolute_path` | `string` | Full filesystem path (backend only, never sent to frontend) |
| `name` | `string` | File or directory name |
| `kind` | `"file" \| "dir"` | Entry type |
| `ext` | `string \| null` | File extension without leading dot, or null |
| `size_bytes` | `number` | File size in bytes (0 for directories) |
| `modified_at_ms` | `number` | Last modification unix epoch ms |
| `is_hidden` | `boolean` | Starts with `.` or OS hidden attribute |
| `is_binary` | `boolean` | Detected by null-byte scan of first 8 KB |

**Serialization note:** `absolute_path` is `#[serde(skip)]` in the Rust struct
so it is never exposed over IPC.

### DirEntry

Subset of `FileEntry` sent to the frontend for directory listings. See
`list_dir` in IPC Contracts.

| Field | Type | Description |
|---|---|---|
| `name` | `string` | File or directory name |
| `path` | `string` | Workspace-relative path |
| `kind` | `"file" \| "dir"` | Entry type |
| `ext` | `string \| null` | File extension without dot |
| `size_bytes` | `number \| null` | File size (null for directories) |
| `modified_at_ms` | `number \| null` | Last modification timestamp |
| `is_hidden` | `boolean` | Hidden file flag |

### Ignore Policy

Directories and files matching ignore rules are excluded from the index.

**Default ignored patterns:**
- `.git/`
- `node_modules/`
- `target/`
- `.idea/`
- `.vscode/`
- `.vigil/cache/`

**Custom ignores:** Loaded in order of precedence:
1. `.gitignore` (if present)
2. `.vigilignore` (app-specific, if present)
3. Application settings overrides

---

## Note

A note is a markdown file with optional structured metadata.

### NoteMetadata

Extracted from markdown files during indexing.

| Field | Type | Description |
|---|---|---|
| `path` | `string` | Workspace-relative path |
| `title` | `string` | First `# heading` or filename without extension |
| `tags` | `string[]` | Tags extracted from frontmatter or inline `#tag` syntax |
| `links_out` | `string[]` | Workspace-relative paths this note links to |
| `links_in_count` | `number` | Number of backlinks (computed from link graph) |
| `word_count` | `number` | Approximate word count of body text |
| `has_frontmatter` | `boolean` | Whether YAML frontmatter block is present |
| `modified_at_ms` | `number` | Last modification timestamp |

**Title extraction rules:**
1. If YAML frontmatter contains a `title` field, use that.
2. Otherwise, use the first `# heading` in the document.
3. Otherwise, use the filename without extension.

**Tag extraction rules:**
1. YAML frontmatter `tags` array (e.g., `tags: [project, idea]`).
2. Inline `#tag` syntax in body text (alphanumeric + hyphens, 1-64 chars).
3. Tags are normalized to lowercase, deduplicated.

---

## Tag

### Tag

A first-class entity representing a tag across the workspace.

| Field | Type | Description |
|---|---|---|
| `name` | `string` | Normalized tag name (lowercase) |
| `count` | `number` | Number of notes using this tag |
| `files` | `string[]` | Workspace-relative paths of notes with this tag |

**Invariants:**
- Tags with `count: 0` are pruned from the index.
- Tag names are limited to 64 characters, alphanumeric + hyphens + underscores.

---

## Link Graph

### NoteNode

A node in the bidirectional link graph, representing one note.

| Field | Type | Description |
|---|---|---|
| `id` | `string` | Stable hash of workspace-relative path |
| `path` | `string` | Workspace-relative path |
| `title` | `string` | Note title (see NoteMetadata title extraction) |
| `tags` | `string[]` | Tags on this note |

### LinkEdge

A directed edge in the link graph.

| Field | Type | Description |
|---|---|---|
| `from_node_id` | `string` | Source note node ID |
| `to_node_id` | `string` | Target note node ID |
| `kind` | `"wikilink" \| "markdown"` | Link syntax type |

**Link syntax:**
- `wikilink`: `[[target]]` or `[[target|display text]]`
- `markdown`: `[text](relative/path.md)` or `[text](./path.md)`

### BacklinkRecord

A single backlink entry with context.

| Field | Type | Description |
|---|---|---|
| `source_path` | `string` | Workspace-relative path of the note containing the link |
| `target_path` | `string` | Workspace-relative path of the linked note |
| `context_snippet` | `string` | The line containing the link (truncated to 200 chars) |

---

## Search Models

### FuzzyMatch

Result from filename fuzzy search.

| Field | Type | Description |
|---|---|---|
| `path` | `string` | Workspace-relative file path |
| `display` | `string` | Formatted display string for omnibar |
| `score` | `number` | Match score (higher is better) |
| `kind` | `"file" \| "dir"` | Entry type |
| `matched_indices` | `number[]` | Character positions matching query in `display` |

### ContentMatch (Epic 4)

Result from full-text content search.

| Field | Type | Description |
|---|---|---|
| `path` | `string` | Workspace-relative file path |
| `line_number` | `number` | 1-based line number |
| `line_start_col` | `number` | Start column of match within line |
| `line_end_col` | `number` | End column of match within line |
| `preview` | `string` | Context line(s) around the match |
| `score` | `number` | Relevance score |

---

## Git Models

### GitHunk

A line-level diff hunk representing a change against git HEAD.

| Field | Type | Description |
|---|---|---|
| `change_type` | `"added" \| "modified" \| "deleted"` | Hunk classification |
| `start_line` | `number` | First affected line in working copy (1-based) |
| `end_line` | `number` | Last affected line in working copy (1-based) |
| `base_start_line` | `number \| null` | First line in base version (null for additions) |
| `base_end_line` | `number \| null` | Last line in base version (null for additions) |

---

## Status Model

### WorkspaceStatus

Composite status for the footer status bar.

| Field | Type | Description |
|---|---|---|
| `branch` | `string \| null` | Current git branch name, null if not a git repo |
| `sync_state` | `string` | One of: `"synced"`, `"ahead"`, `"behind"`, `"diverged"`, `"unknown"` |
| `notes_count` | `number` | Number of `.md` files |
| `tags_count` | `number` | Number of unique tags |
| `files_count` | `number` | Total indexed file count |
| `version` | `string` | Application version string (from `Cargo.toml`) |
| `last_index_update_ms` | `number` | Timestamp of last index refresh |

---

## Event Payload Models

### IndexChange

Payload element for `vigil://index-updated` events.

| Field | Type | Description |
|---|---|---|
| `path` | `string` | Workspace-relative path |
| `change_type` | `"created" \| "changed" \| "deleted"` | What happened |
| `kind` | `"file" \| "dir"` | Entry type |

### IndexReady

Payload for `vigil://index-ready` event.

| Field | Type | Description |
|---|---|---|
| `files_count` | `number` | Total files indexed |
| `notes_count` | `number` | Markdown files indexed |
| `duration_ms` | `number` | Time taken for initial scan |

### FsRenamed

Payload for `vigil://fs-renamed` event.

| Field | Type | Description |
|---|---|---|
| `old_path` | `string` | Previous workspace-relative path |
| `new_path` | `string` | New workspace-relative path |

---

## Plugin Models (Epic 4)

### PluginManifest

| Field | Type | Description |
|---|---|---|
| `id` | `string` | Unique plugin identifier |
| `name` | `string` | Display name |
| `version` | `string` | Semver version string |
| `api_version` | `string` | Required host API semver range |
| `entry_wasm` | `string` | Path to WASM artifact relative to plugin root |
| `capabilities` | `string[]` | Requested capability set |
| `description` | `string \| null` | Optional description |
| `author` | `string \| null` | Optional author |

### PluginRuntimeState

| Field | Type | Description |
|---|---|---|
| `plugin_id` | `string` | References `PluginManifest.id` |
| `enabled` | `boolean` | Whether plugin is active |
| `healthy` | `boolean` | Runtime health status |
| `last_error` | `string \| null` | Most recent error message, if any |

---

## Caching Strategy

### In-Memory Index

- The file index, note metadata index, and link graph are held in memory
  behind `parking_lot::RwLock` for concurrent read access.
- The fuzzy search engine (`nucleo-matcher`) operates directly on the in-memory
  file index.

### Persisted Cache

Optional persisted state under `.vigil/cache/` within the workspace:

| Cache Key | Format | Purpose |
|---|---|---|
| `index.json` | JSON | File entry index for fast re-open |
| `notes.json` | JSON | Note metadata cache |
| `links.json` | JSON | Link graph snapshot |

Cache entries are keyed by:
- `workspace_id`
- File path
- File `modified_at_ms` timestamp

If the cached `modified_at_ms` does not match the filesystem, the entry is
re-scanned.

---

## Invalidation Rules

1. **File create/update/delete:** Invalidate the corresponding `FileEntry`,
   `NoteMetadata` (if `.md`), and affected `LinkEdge`/`BacklinkRecord` entries
   immediately.
2. **Workspace switch:** Clear all in-memory state (index, graph, plugin
   runtimes). Load new workspace from scratch or from persisted cache.
3. **Branch change or git refresh:** Invalidate hunk cache for all files.
   Re-compute hunks for currently visible files.
4. **Settings/ignore change:** Schedule incremental re-index. Files newly
   matching ignore rules are removed; files no longer matching are scanned.
5. **Tag change:** Re-extract tags for the affected note and update the
   global tag index.

---

## Concurrency Model

| Component | Thread | Synchronization |
|---|---|---|
| FS watcher (`notify`) | Dedicated Rust thread | Sends events via `crossbeam-channel` |
| File index | Main Tauri thread | `parking_lot::RwLock<FileIndex>` |
| Link graph | Main Tauri thread | `parking_lot::RwLock<LinkGraph>` |
| Search engine | Read from index snapshot | Lock-protected read access |
| File writes | Per-file | `parking_lot::Mutex` keyed by path |
| Event emission | Tauri event loop | Serialized via `app_handle.emit()` |

**Key rules:**
- Reads never block writes for more than the lock acquisition time.
- Writes acquire per-file mutation locks to prevent concurrent writes to the
  same file.
- The UI receives events from a serialized channel to preserve ordering.
- Long-running operations (initial index scan, content search) run on background
  threads and emit completion events.

---

## Cross-Reference

This data model is consumed by the IPC contracts defined in
[`ipc-contracts.md`](ipc-contracts.md). Every response type in the IPC contract
maps to a model defined here. Any model change requires a synchronized update
to both documents.
