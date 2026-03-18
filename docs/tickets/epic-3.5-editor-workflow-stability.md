# Epic 3.5: Editor Workflow Stabilization & UX Cleanup

This epic is post-MVP stabilization work. It resolves core editor workflow regressions and UX noise before Epic 4 expansion.

## Task [3.5.1]: Remove Default Split Layout (Adaptive Main Pane Only)
Goal: Remove always-on code split behavior from the default workspace and keep one adaptive main editor surface.
Commands/Code:
Create/modify files:
- `src/routes/+page.svelte`
- `src/lib/components/layout/WorkspaceGrid.svelte`
- `src/lib/stores/ui.ts`
- `src/lib/types/store.ts`
- `docs/specs/keyboard-map.md`
Acceptance Criteria:
- By default, the main pane renders only one editor surface.
- Opening `.md` files shows the note editor in the main pane.
- Opening non-markdown text/code files shows the code editor in the main pane.
- Split mode is not auto-enabled by file type.

## Task [3.5.2]: Add Multi-Tab Editing + Optional Side-by-Side View
Goal: Support multiple open files with tabbed navigation, and allow side-by-side mode as explicit user action.
Commands/Code:
Create/modify files:
- `src/lib/stores/editor.ts`
- `src/lib/features/editor/EditorRouter.svelte`
- `src/lib/features/editor/NoteEditor.svelte`
- `src/lib/features/editor/CodeEditor.svelte`
- `src/routes/+page.svelte`
Acceptance Criteria:
- Users can keep multiple files open as tabs and switch active tab without data loss.
- Tab close and tab activation behavior are deterministic and tested.
- Side-by-side view is available as an explicit toggle, not automatic routing.
- Single-pane adaptive routing remains the default mode.

## Task [3.5.3]: Fix Linux Monaco Crash Path
Goal: Eliminate editor crashes/hangs specific to Linux Monaco runtime paths and prove stability with reproducible evidence.
Commands/Code:
Create/modify files:
- `src/lib/features/editor/CodeEditor.svelte`
- `src/lib/features/editor/monaco-config.ts`
- `src-tauri/src/main.rs`
- `src-tauri/tauri.conf.json`
- `docs/qa/test-matrix.md`
Acceptance Criteria:
- Reproduction scenario is captured with explicit environment details (distro, compositor/window system, GPU/driver).
- Crash no longer occurs under the documented reproduction path.
- Mitigation is validated on at least one Linux CI/runtime smoke job and one local Linux run.
- Any platform-specific tradeoff (performance/features) is documented as an explicit risk/defer note.

### Required Input for Task [3.5.3]
- Linux distro + version
- Desktop/session type (`X11` or `Wayland`)
- GPU + driver version
- Minimal reproduction steps
- Runtime logs/stack trace

### Current Repro Baseline (Captured 2026-03-18)
- Distro/session: Fedora KDE with Hyprland (`Wayland` compositor path)
- GPU/driver: Intel ADL GT2, Mesa `25.3.6` (`OpenGL 4.6`, `direct rendering: Yes`)
- Repro: run `npx tauri dev`, open any code file, app resets while Monaco is loading
- Trace signature:
  - `ERROR: WebKit encountered an internal error. This is a WebKit bug.`
  - `WebLoaderStrategy.cpp(618) : internallyFailedLoadTimerFired()`
  - Repeats during code-file open path, interleaved with backend perf logs

## Task [3.5.4]: Remove Explorer Metadata Blocks
Goal: Simplify Explorer by removing non-essential notes/files metadata blocks.
Commands/Code:
Create/modify files:
- `src/lib/features/explorer/ExplorerPanel.svelte`
- `src/lib/features/explorer/explorer-store.svelte.ts`
Acceptance Criteria:
- Explorer no longer shows the `Notes` count card.
- Explorer no longer shows `Collections` note/file summary rows.
- File tree browsing, selection, and lazy directory expansion remain unchanged.

## Task [3.5.5]: Render Mermaid Charts in Markdown Notes
Goal: Support Mermaid diagram rendering for fenced code blocks in markdown preview mode.
Commands/Code:
Create/modify files:
- `src/lib/features/preview/InlinePreview.svelte`
- `src/lib/utils/markdown.ts`
- `src/lib/features/editor/NoteEditor.svelte`
- `docs/qa/test-matrix.md`
Acceptance Criteria:
- Fenced blocks using ` ```mermaid ` render as diagrams in preview mode.
- Invalid Mermaid definitions fail gracefully (show readable fallback, no crash/reset).
- Raw markdown source remains unchanged and editable.
- Rendering is deterministic across Linux and Windows desktop builds.
