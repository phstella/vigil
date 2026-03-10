# Epic 1: Backend Foundation (Rust/Tauri)

> Build all Rust-side capabilities: filesystem operations, git integration,
> file watching, fuzzy search, full-text indexing, and bidirectional link
> graph parsing.

## Why This Epic Exists

The Rust backend is Vigil's performance advantage. Every heavy operation —
file I/O, git diffing, search indexing, link graph resolution — runs natively
in Rust on background threads, keeping the UI thread free for zero-lag editing.

## Tasks

| Task | Name | Depends On |
|---|---|---|
| [1.1](1.1-fs-read-write.md) | File System Read/Write Commands | Epic 0 |
| [1.2](1.2-workspace-management.md) | Workspace Open & Recent Files | 1.1 |
| [1.3](1.3-file-watcher.md) | Recursive File Watcher | 1.2 |
| [1.4](1.4-git-diff.md) | Git Status & Diff on Background Thread | Epic 0 |
| [1.5](1.5-fuzzy-finder.md) | Fuzzy File Finder | 1.2 |
| [1.6](1.6-fulltext-search.md) | Full-Text Search Index | 1.2 |
| [1.7](1.7-link-graph.md) | Bidirectional Link Graph | 1.2 |
| [1.8](1.8-tag-index.md) | Tag Extraction & Index | 1.2 |

## Execution Order

```
1.1 ──▶ 1.2 ──▶ 1.3
              ├──▶ 1.5  (parallel)
              ├──▶ 1.6  (parallel)
              ├──▶ 1.7  (parallel)
              └──▶ 1.8  (parallel)

1.4 (independent, can run in parallel with 1.1–1.3)
```

## Key Design Decisions

- **Atomic writes**: All file writes go to a `.tmp` file first, then rename.
  This prevents data loss on crash.
- **Background threads**: Git diffing and search indexing use
  `tokio::task::spawn_blocking` to avoid blocking the Tauri main thread.
- **Incremental indexing**: The Tantivy search index and link graph update
  incrementally on file-watcher events, not full rebuilds.
- **Trash, not delete**: File deletion moves to OS trash via the `trash` crate.

## Exit Criteria

- All 8 Tauri command groups are callable from the frontend via `invoke()`
- File watcher emits events visible in the browser console
- Fuzzy find returns results in < 5ms for 50k-file workspaces
- Git diff returns correct hunk data matching `git diff` output
- Full-text search returns results with snippets in < 100ms
- Backlink resolution is correct for `[[wikilink]]` references
- Tag extraction works for both YAML frontmatter and inline `#tags`
