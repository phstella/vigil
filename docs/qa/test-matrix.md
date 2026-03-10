# QA Test Matrix

## Platforms
- Linux (Ubuntu LTS, x86_64)
- Windows 11 (x86_64)

## Build Modes
- Development (`npx tauri dev`)
- Release (`npx tauri build`)

## Core Smoke Matrix (MVP)

| Test ID | Flow | Linux | Windows | Pass Criteria |
|---|---|---|---|---|
| QA-001 | Launch app | Required | Required | Main shell interactive within budget |
| QA-002 | Open workspace | Required | Required | Explorer tree loads and is navigable |
| QA-003 | Read/write markdown | Required | Required | Save persists and reload confirms content |
| QA-004 | Open code file | Required | Required | Right pane editor opens and edits correctly |
| QA-005 | Ctrl+P fuzzy file search | Required | Required | Ranked results and open-on-enter works |
| QA-006 | Git gutter markers | Required | Required | Added/modified/deleted markers reflect file diff |
| QA-007 | Backlinks panel | Required | Required | Link updates reflected in backlinks list |
| QA-008 | Status bar live values | Required | Required | Branch/sync/count/version values display correctly |
| QA-009 | Keyboard shortcuts | Required | Required | Ctrl+P/Ctrl+S/Ctrl+B behave consistently |
| QA-010 | Packaging | Required | Required | Installer/binary launches and runs smoke suite |

## Expansion Matrix (Epic 4)

| Test ID | Flow | Linux | Windows | Pass Criteria |
|---|---|---|---|---|
| QA-101 | Content search (phrase) | Required | Required | Snippet-level results under target latency |
| QA-102 | Graph view open/select | Required | Required | Selecting graph node opens note |
| QA-103 | Markdown WYSIWYG toggle | Required | Required | Toggle keeps cursor state and content integrity |
| QA-104 | Vim mode | Required | Required | Normal/insert transitions are reliable |
| QA-105 | Plugin install from omnibar | Required | Required | Install, enable, and disable cycle works |
| QA-106 | Plugin capability prompt | Required | Required | Requested capabilities shown and enforced |

## Regression Checklist per PR
- Lint/format checks pass.
- Rust tests pass.
- At least QA-001 through QA-006 manually verified for UI-affecting PRs.
- Performance spot check for omnibar and typing latency.

## Release Gate
Release candidate is blocked if any of these fail:
- Any required smoke test.
- Any security test for plugin capability isolation.
- Performance budgets exceeded by > 10% at p95.

## Evidence Collection
For each executed test, record:
- build commit hash
- OS/version
- test result (`PASS`/`FAIL`)
- short notes and screenshot/log links
