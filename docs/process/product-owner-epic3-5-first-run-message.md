# Product Owner Agent First-Run Message (Epic 3.5)

Copy/paste this message when starting a PO-style orchestrator run for Epic 3.5:

```md
Run Epic 3.5 orchestration for Vigil using:

- /home/shepard/dev/vigil/docs/process/product-owner-orchestrator-agent-prompt.md

Objective:
- Close Epic 3.5 to a real PASS state with evidence-backed completion.

Scope:
- Epic 3.5 only (tasks 3.5.1-3.5.5 from canonical backlog).
- Canonical source: /home/shepard/dev/vigil/docs/tickets/00-unified-vigil-backlog.md
- Task detail source: /home/shepard/dev/vigil/docs/tickets/epic-3.5-editor-workflow-stability.md

Execution order and dependency rules:
1) 3.5.1 default single-pane behavior first
2) 3.5.2 multi-tab + user-invoked side-by-side after 3.5.1
3) 3.5.3 linux Monaco crash stabilization can run after 3.5.1
4) 3.5.4 explorer metadata cleanup can run in parallel with 3.5.2/3.5.3
5) 3.5.5 Mermaid preview rendering after 3.5.2 is stable

Known Linux baseline for 3.5.3 (must be validated):
- Fedora KDE + Hyprland (Wayland)
- Intel ADL GT2, Mesa 25.3.6
- Historical failure signature: `WebLoaderStrategy::internallyFailedLoadTimerFired`

Execution requirements:
- Use existing prompts in /home/shepard/dev/vigil/agents-prompts/epic-3.5/ wherever possible.
- For missing/split subtasks, instantiate:
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

Runtime evidence rules:
- 3.5.3 requires Linux runtime smoke evidence (no reset/crash, no repeated worker error loop).
- 3.5.5 requires Mermaid runtime smoke evidence for both valid and invalid diagrams.
- If runtime evidence is missing, mark task `PARTIAL` with blocker/impact; do not mark `PASS`.

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
