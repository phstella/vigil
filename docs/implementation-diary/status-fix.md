# Status Bar Re-fetch Fix

## 2026-03-12T23:21 — Status bar stale-mock fix

**Problem:** `statusStore.initialize()` ran before `openAndLoadWorkspace('.')` in `onMount`. If the backend was not yet ready, the status store fell back to mock data and never re-fetched after the workspace opened.

**Fix:** Chained `statusStore.initialize()` inside the `.then()` of `openAndLoadWorkspace('.')` so the status bar re-fetches once the workspace is confirmed open.

**File changed:** `src/routes/+page.svelte` (lines 193-203)

**Validation:**
- `npm run check` — PASS (0 errors, 2 pre-existing warnings)
- `npm run lint` — PASS (1 pre-existing error in NoteEditor.svelte, unrelated)
- `npm run build` — PASS (built successfully)
