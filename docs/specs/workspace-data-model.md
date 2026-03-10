# Workspace Data Model

## Scope
Define canonical structures for workspace files, search/index data, link graph, git hunks, and cache invalidation.

## Root Workspace

### WorkspaceRoot
- `canonical_path: string`
- `opened_at_ms: number`
- `workspace_id: string` (stable hash of canonical path)

Rule: all file operations must resolve against `canonical_path` and reject traversal outside root.

## File Index

### FileEntry
- `path: string` (workspace-relative)
- `absolute_path: string` (backend only)
- `kind: file | dir`
- `ext: string | null`
- `size_bytes: number`
- `modified_at_ms: number`
- `is_hidden: boolean`
- `is_binary: boolean`

### Ignore Policy
Default ignored directories:
- `.git/`
- `node_modules/`
- `target/`
- `.idea/`
- `.vscode/`

Optional custom ignores loaded from `.gitignore` plus app settings.

## Search Models

### FuzzyMatch
- `path`
- `display`
- `score`
- `kind` (`file`, `command`, `plugin`)

### ContentMatch (Epic 4)
- `path`
- `line_number`
- `line_start_col`
- `line_end_col`
- `preview`
- `score`

## Link Graph

### NoteNode
- `id` (stable hash of path)
- `path`
- `title`
- `tags[]`

### LinkEdge
- `from_node_id`
- `to_node_id`
- `kind` (`wikilink`, `markdown`)

### BacklinkRecord
- `source_path`
- `target_path`
- `context_snippet`

## Git Hunk Model

### GitHunk
- `path`
- `change_type` (`added`, `modified`, `deleted`)
- `start_line`
- `end_line`
- `base_start_line?`
- `base_end_line?`

## Status Model

### WorkspaceStatus
- `branch`
- `sync_state`
- `notes_count`
- `tags_count`
- `version`
- `last_index_update_ms`

## Plugin Models (Epic 4)

### PluginManifest
- `id`
- `name`
- `version`
- `api_version`
- `entry_wasm`
- `capabilities[]`

### PluginRuntimeState
- `plugin_id`
- `enabled`
- `healthy`
- `last_error?`

## Caching Strategy
- In-memory index for explorer and fuzzy search.
- Optional persisted content index under `.vigil/cache/`.
- Persisted cache keys:
  - `workspace_id`
  - file path
  - file modified timestamp

## Invalidation Rules
1. On file create/update/delete event, invalidate corresponding index entries immediately.
2. On workspace change, clear all in-memory state and plugin runtime state.
3. On branch change or git refresh event, invalidate hunk cache for affected files.
4. On settings/ignore change, schedule incremental re-index.

## Concurrency Model
- FS watcher runs on dedicated Rust thread.
- Search requests read from lock-protected snapshot state.
- Writes acquire per-file mutation lock.
- UI receives events from serialized channel to preserve ordering.
