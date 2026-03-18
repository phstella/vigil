# Epic 3.5 Agent Prompts

Use these prompts in strict order:

1. 3.5.1-single-pane-default-and-optional-split.md
2. 3.5.2-multi-tab-editing-and-user-invoked-side-by-side.md
3. 3.5.3-linux-monaco-crash-stability.md
4. 3.5.4-remove-explorer-metadata-blocks.md
5. 3.5.5-mermaid-rendering-in-markdown-preview.md

Notes:
- Canonical scope comes from `docs/tickets/00-unified-vigil-backlog.md` and `docs/tickets/00-mvp-execution-order.md`.
- Task details come from `docs/tickets/epic-3.5-editor-workflow-stability.md`.
- If any document conflicts, canonical backlog/execution-order win.
- Each task requires an append-only diary at `docs/implementation-diary/{TASK_ID}.md`.
- PASS requires evidence-based completion: frontend + rust gates, acceptance mapping, and runtime smoke for UI/runtime-risk tasks.
- If runtime smoke cannot run locally, task must be reported as `PARTIAL` with blocker and impact.
- Epic closeout must include `docs/process/epic-completion-checklist.md` and a handoff generated from `docs/process/agent-handoff-template.md`.
