# Product Owner Agent First-Run Message (Epic 3 Recovery)

Copy/paste this message when starting a PO-style orchestrator run for Epic 3:

```md
Run Epic 3 recovery orchestration for Vigil using:

- /home/shepard/dev/vigil/docs/process/product-owner-orchestrator-agent-prompt.md

Objective:
- Close Epic 3 to a real PASS state (not partial) with evidence-backed completion.

Scope:
- Epic 3 only (tasks 3.1-3.12 from canonical backlog).
- Canonical source: /home/shepard/dev/vigil/docs/tickets/00-unified-vigil-backlog.md

Current baseline to use:
- /home/shepard/dev/vigil/docs/epic-3-re-review-2026-03-12.md
- /home/shepard/dev/vigil/docs/process/epic-3-completion-checklist-2026-03-12.md

Priority blockers to assign first:
1) Fix critical note-switch data-loss path:
   - /home/shepard/dev/vigil/src/lib/features/editor/NoteEditor.svelte
   - /home/shepard/dev/vigil/src/lib/features/editor/note-store.ts
2) Ensure status is re-fetched after successful workspace open:
   - /home/shepard/dev/vigil/src/routes/+page.svelte
   - /home/shepard/dev/vigil/src/lib/features/status/status-store.ts
3) Resolve graph scope mismatch:
   - Implement/register backend get_note_graph, or formally defer and downgrade claim.
4) Add CI artifact packaging jobs (Linux/Windows tauri build).
5) Resolve formatting gate failures.

Execution requirements:
- Use existing prompts in /home/shepard/dev/vigil/agents-prompts/epic-3/ wherever possible.
- For missing recovery-specific work, instantiate:
  /home/shepard/dev/vigil/agents-prompts/implementation-ticket-template.md
- Enforce strict file scopes to avoid overlap.
- Require append-only diary updates in /home/shepard/dev/vigil/docs/implementation-diary/{TASK_ID}.md

Quality gates required per completed task:
- npm run check
- npm run lint
- npm run build
- npm run format:check
- cargo fmt --manifest-path src-tauri/Cargo.toml --all --check
- cargo clippy --manifest-path src-tauri/Cargo.toml --workspace --all-targets -- -D warnings
- cargo test --manifest-path src-tauri/Cargo.toml --workspace

Closeout requirements:
- Update epic checklist with real statuses.
- Produce a final handoff using:
  /home/shepard/dev/vigil/docs/process/agent-handoff-template.md
- Do not claim PASS while any critical/high issue remains.

Start now by returning:
1) Scope Decision
2) Task Board
3) Developer Prompt Packets for the first wave
4) JSON Summary
```
