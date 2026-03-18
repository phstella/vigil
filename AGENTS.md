# AGENTS.md

Repository-level execution and review rules for local coding agents.

## Canonical Scope

1. Epic/task scope source of truth is `docs/tickets/00-unified-vigil-backlog.md`.
2. Execution order source of truth is `docs/tickets/00-mvp-execution-order.md`.
3. If another document conflicts, backlog + execution order win.

## Completion Standard (Required)

A task/epic can be marked `PASS` only when all items below are true:

1. The feature works end-to-end against real runtime paths (not only mocks/fallbacks).
2. Positive-path behavior is validated (manual or automated).
3. Negative/error-path behavior is validated for high-risk flows.
4. Acceptance criteria in the canonical ticket are explicitly mapped to evidence.
5. Quality gates required for that scope have run and results are recorded.
6. Open risks/deferred items are clearly listed and not labeled as `PASS`.

## Evidence Rules

For every completed task, include:

1. Code references (absolute file paths) for key implementation points.
2. Validation command results with pass/fail status.
3. Manual smoke result notes for UI/runtime behavior when applicable.
4. Explicit list of unresolved items and their impact.

Use `docs/process/agent-handoff-template.md` for handoffs and review summaries.

## Required Validation Commands

Frontend baseline:

1. `npm run check`
2. `npm run lint`
3. `npm run build`
4. `npm run format:check`

Rust baseline (when toolchain is available):

1. `cargo fmt --manifest-path src-tauri/Cargo.toml --all --check`
2. `cargo clippy --manifest-path src-tauri/Cargo.toml --workspace --all-targets -- -D warnings`
3. `cargo test --manifest-path src-tauri/Cargo.toml --workspace`

If a required command cannot run locally, record the exact blocker and mark the related claim as `PARTIAL` unless CI evidence is attached.

## Release Claim Rules

Do not claim MVP/epic completion if any release gate is missing:

1. Required smoke coverage from `docs/qa/test-matrix.md`
2. Required CI gates for frontend + rust
3. Packaging/job evidence for platform artifacts when scope includes distribution

## Prohibited Claim Patterns

1. "Build succeeds" used as sole proof of feature completion
2. Mock/fallback behavior labeled as full completion without explicit defer decision
3. Acceptance criteria marked `PASS` without mapped evidence
4. Closing an issue while known regressions remain unlisted

## Required Artifacts for Epic Closeout

1. `docs/implementation-diary/<task>.md` entries updated
2. Handoff/review generated from `docs/process/agent-handoff-template.md`
3. Epic checklist completed using `docs/process/epic-completion-checklist.md`
