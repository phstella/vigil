# IPC Contracts

## Purpose
Define stable frontend-backend contracts between Svelte and Tauri/Rust.

## Versioning
- Contract version: `v1`.
- Backward-compatible additions: new optional fields only.
- Breaking changes: require `v2` namespace and migration notes.

## Common Envelope
All command responses should serialize to one of:

```json
{ "ok": true, "data": {} }
```

```json
{ "ok": false, "error": { "code": "STRING", "message": "STRING", "details": {} } }
```

Error codes (initial set):
- `WORKSPACE_NOT_OPEN`
- `PATH_OUTSIDE_WORKSPACE`
- `FILE_NOT_FOUND`
- `PERMISSION_DENIED`
- `INVALID_ARGUMENT`
- `INDEX_UNAVAILABLE`
- `GIT_UNAVAILABLE`
- `PLUGIN_ERROR`
- `INTERNAL_ERROR`

## Commands (MVP)

### `open_workspace(root_path: string)`
- Returns: workspace metadata, initial file counts, canonical root path.
- Notes: root path must be canonicalized and validated.

### `list_dir(path: string)`
- Returns: sorted directory entries with `name`, `path`, `kind`, `size?`, `modified_at?`.
- Notes: paths are workspace-relative when sent to frontend.

### `read_file(path: string)`
- Returns: UTF-8 text content + metadata (`encoding`, `modified_at`, `size`).
- Notes: binary files return `INVALID_ARGUMENT` unless explicitly supported later.

### `write_file(path: string, content: string)`
- Returns: new `modified_at`, `size`, and `etag`.
- Notes: should fail on stale `etag` when optimistic concurrency is enabled.

### `create_note(path: string)`
- Returns: created file metadata.
- Notes: auto-append `.md` if no extension policy is enabled by settings.

### `fuzzy_find(query: string, limit: number)`
- Returns: ranked list of matches (`path`, `display`, `score`, `kind`).
- Notes: used by `Ctrl+P` omnibar.

### `get_git_hunks(path: string)`
- Returns: line-range hunks with `type` = `added | modified | deleted`.

### `get_backlinks(path: string)`
- Returns: list of notes linking to target file.

### `workspace_status()`
- Returns: `branch`, `sync_state`, `notes_count`, `tags_count`, `version`.

## Commands (Epic 4)

### `search_content(query: string, limit: number)`
- Returns: snippet-level matches (`path`, `line`, `preview`, `score`).

### `get_note_graph()`
- Returns: graph payload `{ nodes: [], edges: [] }`.

### `plugin_list()`
- Returns: installed plugin manifests + enabled state.

### `plugin_install(plugin_id: string, version?: string)`
- Returns: install result + capability summary.

### `plugin_enable(plugin_id: string, enabled: boolean)`
- Returns: active state and runtime status.

## Event Channels
- `index://updated`
  - Payload: changed paths + update type (`created`, `changed`, `deleted`).
- `git://hunks`
  - Payload: file path + latest hunk list.
- `status://updated`
  - Payload: workspace status snapshot.
- `plugins://updated`
  - Payload: plugin install/enable/runtime transition.

## Type Ownership
- Rust source of truth: `src-tauri/src/models/*.rs`.
- Frontend mirrors: `src/lib/types/*.ts`.
- Rule: frontend types must be generated or manually synced from Rust models each contract change.

## Validation Rules
- Every command validates canonical path confinement to workspace root.
- Every command enforces payload size limits to prevent UI freezes.
- All event payloads include `timestamp_ms` and `contract_version`.

## Change Control
- Any contract change requires updates in:
  - Rust models
  - frontend IPC wrappers
  - this file
  - integration tests
