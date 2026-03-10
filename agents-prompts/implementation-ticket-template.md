# Vigil Coding Agent Prompt Template

Use this template to run one scoped implementation task at a time.

## Prompt

```md
You are implementing Vigil ticket {TASK_ID}.

<context>
- Repo root: /home/shepard/dev/vigil
- Canonical plan: /home/shepard/dev/vigil/docs/tickets/00-unified-vigil-backlog.md
- Execution order: /home/shepard/dev/vigil/docs/tickets/00-mvp-execution-order.md
- Architecture boundaries: /home/shepard/dev/vigil/docs/architecture/01-repository-architecture.md
- IPC contracts: /home/shepard/dev/vigil/docs/specs/ipc-contracts.md
- Performance budgets: /home/shepard/dev/vigil/docs/specs/editor-performance-budget.md
- QA gates: /home/shepard/dev/vigil/docs/qa/test-matrix.md
- Out of scope: Vim mode and plugin store marketplace flow
</context>

<task>
- Implement exactly: {TASK_TITLE}
- Ticket source: {TICKET_FILE_AND_SECTION}
- Optional detailed guidance: {RELATED_VIGIL_PLAN_FILES}
</task>

<scope>
- Allowed files/modules only:
  {ALLOWED_PATHS}
- Forbidden files/modules:
  {FORBIDDEN_PATHS}
- Do not modify unrelated files.
</scope>

<non_goals>
- Do not implement:
  {NON_GOALS}
</non_goals>

<execution_requirements>
1. Restate implementation plan in 3-6 steps.
2. Implement code/docs required by this ticket.
3. Run required checks/tests:
   {CHECK_COMMANDS}
4. If a check fails, fix and re-run until passing or clearly blocked.
5. Report results using the exact output contract below.
</execution_requirements>

<completion_contract>
Task is done only if:
1. All acceptance criteria are addressed.
2. Required checks/tests were run and outcomes reported.
3. Output format exactly matches the output contract.
</completion_contract>

<verification_loop>
Before finalizing, validate:
- Correctness: every required behavior is implemented.
- Grounding: each claim is supported by changed code/tests/command output.
- Scope: no unrelated files were changed.
- Performance/safety: no avoidable regressions introduced.
</verification_loop>

<missing_context_policy>
If required context is missing:
1. Inspect local files first.
2. Make the minimal reasonable assumption when low risk.
3. If still blocked or high risk, ask one concise question.
</missing_context_policy>

<output_contract>
Return exactly these sections in order:

1) Changed Files
- List each file and what changed.

2) Commands Run
- List commands actually executed and concise outcomes.

3) Acceptance Criteria
- Criterion-by-criterion status: pass/fail/partial.

4) Risks / Follow-ups
- Remaining risks, TODOs, or blockers.

5) JSON Summary
```json
{
  "task_id": "{TASK_ID}",
  "status": "pass|partial|blocked",
  "changed_files": ["path"],
  "checks": [
    {"name": "command or check", "result": "pass|fail|not_run"}
  ],
  "open_risks": ["text"]
}
```
</output_contract>
```

## Usage Notes

- Keep one prompt per ticket.
- Prefer narrow write scopes to reduce merge conflicts.
- Keep the static template text stable; only change placeholders.
- If a ticket is too large, split it into sequential subtasks before implementation.
