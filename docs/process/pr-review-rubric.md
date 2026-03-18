# PR Review Rubric (Sufficiency Focus)

Use this rubric for deep implementation reviews. Findings must be listed by severity first.

## Review Order

1. Critical risks (data loss, security, integrity, hard runtime break)
2. High risks (major behavior regressions, incorrect completion claims)
3. Medium risks (partial integrations, missing gates, doc drift)
4. Low risks (maintainability, minor UX, cleanup)

## What Counts as Strong Evidence

1. Runtime behavior proof (not only static inspection)
2. Command outputs for required validation gates
3. Precise code references tied to each claim
4. Acceptance criteria mapping to observed behavior

## Anti-Patterns (Do Not Approve As Complete)

1. Compile/build pass presented as end-to-end completion proof
2. Mock/demo fallback treated as production-complete without explicit defer
3. "Should work" claims without executable validation
4. Missing negative-path checks for high-risk features

## Minimum Review Output

1. Severity-ordered findings with file references
2. Validation summary (what ran, what did not, and why)
3. Task-by-task sufficiency matrix for scoped tasks
4. Clear final verdict: `PASS`, `PARTIAL`, or `FAIL`
