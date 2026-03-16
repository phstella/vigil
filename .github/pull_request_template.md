## Summary

- Scope:
- Canonical task IDs:
- Why this change:

## Validation

Frontend:

- [ ] `npm run check`
- [ ] `npm run lint`
- [ ] `npm run build`
- [ ] `npm run format:check`

Rust:

- [ ] `cargo fmt --manifest-path src-tauri/Cargo.toml --all --check`
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --workspace --all-targets -- -D warnings`
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml --workspace`

If any item is unchecked, explain:

- Command:
- Blocker:
- Risk:

## Acceptance Criteria Mapping

Reference canonical source: `docs/tickets/00-unified-vigil-backlog.md`

| Task | Acceptance Criteria | Evidence (code/tests/manual) | Status (`PASS`/`PARTIAL`/`FAIL`) |
| ---- | ------------------- | ---------------------------- | -------------------------------- |
|      |                     |                              |                                  |
|      |                     |                              |                                  |

## Runtime Checks

- [ ] Positive-path runtime behavior validated
- [ ] Negative/error paths validated for high-risk flows
- [ ] No critical/high unresolved regressions

## Risks and Defers

- Open risks:
- Deferred items:
- Follow-up tasks:

## Process Compliance

- [ ] Reviewed `AGENTS.md`
- [ ] Used `docs/process/epic-completion-checklist.md`
- [ ] Handoff/review doc follows `docs/process/agent-handoff-template.md`
