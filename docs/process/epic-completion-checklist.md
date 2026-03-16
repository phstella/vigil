# Epic Completion Checklist

Use this checklist before marking an Epic or task as complete.

## Metadata

- Epic:
- Branch:
- Commit range:
- Reviewer:
- Date:

## Scope Lock

- [ ] Scope source is `docs/tickets/00-unified-vigil-backlog.md`
- [ ] Task IDs mapped to current implementation
- [ ] Any defer/scope change explicitly documented

## Acceptance Mapping

For each task, map acceptance criteria to evidence.

| Task | Acceptance Criteria | Code References | Validation Evidence | Status (`PASS`/`PARTIAL`/`FAIL`) |
|---|---|---|---|---|
| | | | | |
| | | | | |
| | | | | |

## Validation Gates

Frontend:

- [ ] `npm run check`
- [ ] `npm run lint`
- [ ] `npm run build`
- [ ] `npm run format:check`

Rust:

- [ ] `cargo fmt --manifest-path src-tauri/Cargo.toml --all --check`
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --workspace --all-targets -- -D warnings`
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml --workspace`

If any gate is skipped, record blocker and resulting risk:

- Blocker:
- Impact:

## Runtime Behavior Checks

- [ ] Positive-path smoke checks completed for relevant QA flows (`docs/qa/test-matrix.md`)
- [ ] High-risk negative paths validated (save failure, conflict path, missing workspace/index, etc.)
- [ ] Mock/fallback paths are not counted as full completion unless explicitly accepted as defer

## Release Readiness (when applicable)

- [ ] Packaging/artifact generation validated for required platforms
- [ ] CI jobs exist for required gates and artifacts
- [ ] Known warnings/issues assessed and dispositioned

## Final Risk Summary

- Open critical risks:
- Open high risks:
- Open medium risks:
- Deferred items (explicitly accepted):

## Final Decision

- Decision: `PASS` / `PARTIAL` / `FAIL`
- Rationale:
- Required next actions:
