# Editor Performance Budget

## Objective
Guarantee a desktop-native feel with no visible input lag.

## Hard Budgets (Development Builds)
- Cold app launch to interactive shell: <= 1200 ms
- Warm launch to interactive shell: <= 500 ms
- Workspace open (10k files): <= 1500 ms to first usable tree
- `Ctrl+P` first result render: <= 80 ms for filename search
- Content search result render (Epic 4): <= 150 ms median
- Keystroke-to-paint latency in active editor: <= 16 ms p95
- Scroll FPS in large text files: >= 60 FPS target, >= 45 FPS floor
- File switch (already indexed): <= 120 ms
- Markdown live-toggle switch (Epic 4): <= 100 ms

## Memory Budget (Desktop)
- Idle baseline (no workspace): <= 220 MB RSS
- Medium workspace (5k files): <= 450 MB RSS
- Large workspace (10k files + search index): <= 700 MB RSS

## Throughput Targets
- Indexing throughput on SSD: >= 20k files/min for text metadata pass.
- Git hunk refresh after save: <= 200 ms for file under 5k lines.

## Measurement Protocol
- Test environments:
  - Linux: latest Ubuntu LTS, Wayland and X11 spot-check.
  - Windows: latest stable Windows 11.
- Build mode:
  - Performance gates run on release-profile binaries.
- Repeatability:
  - Run each benchmark 10 times, report median and p95.

## Instrumentation Requirements
- Rust timers around:
  - workspace scan
  - fuzzy search
  - content search
  - git hunk computation
- Frontend timers around:
  - omnibar open and first paint
  - editor mount and first paint
  - pane switch render
- Emit telemetry to local debug log file in development mode.

## Optimization Priorities
1. Avoid blocking UI thread on IPC calls.
2. Incremental index updates only (no full re-index unless requested).
3. Debounce high-frequency commands and events.
4. Lazy-load heavy editor modules (Monaco + optional plugins).
5. Virtualize long lists (explorer and omnibar result sets).

## Regression Policy
- Any p95 budget regression > 10% blocks merge unless waived.
- Waiver requires documented root cause and follow-up task.
