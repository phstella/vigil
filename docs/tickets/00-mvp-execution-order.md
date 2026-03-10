# Vigil Execution Order (MVP + Expansion)

Execute in strict order without parallel stream changes.

1. Epic 0 tasks `0.1` -> `0.9`
2. Epic 1 tasks `1.1` -> `1.9`
3. Epic 2 tasks `2.1` -> `2.10`
4. Epic 3 tasks `3.1` -> `3.9`
5. Epic 4 tasks `4.1` -> `4.10` (post-MVP expansion from strategy chat)

Scope note:
- Epics 0-3 define the MVP baseline.
- Epic 4 adds graph view, live WYSIWYG toggle, Vim support, deeper search, typography, and WASM plugin/store capabilities.

Do not start the next task until the current task acceptance criteria pass.
