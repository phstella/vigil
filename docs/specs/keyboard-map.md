# Keyboard Map

## Goal
Define a deterministic, keyboard-centric interaction model.

## Global Shortcuts (All Views)
- `Ctrl+P`: open omnibar
- `Ctrl+N`: create new note
- `Ctrl+S`: save active buffer
- `Ctrl+B`: toggle explorer sidebar
- `Ctrl+\\`: toggle split orientation (future)
- `Ctrl+Shift+P`: command-only omnibar mode

## Navigation
- `Ctrl+1`: focus explorer
- `Ctrl+2`: focus center note pane
- `Ctrl+3`: focus right code pane
- `Ctrl+Tab`: next open tab/pane item
- `Ctrl+Shift+Tab`: previous open tab/pane item

## Omnibar Controls
- `Enter`: open selected item/execute command
- `Esc`: close omnibar
- `ArrowUp/ArrowDown`: move selection
- `Ctrl+J` / `Ctrl+K`: optional Vim-like list navigation

## Editor Controls
- `Tab`: indent or snippet expansion
- `Shift+Tab`: outdent
- `Ctrl+/`: toggle line comment (code files)
- `Alt+Up/Alt+Down`: move line/selection

## Markdown-Specific
- `Ctrl+.`: toggle markdown live-render mode (Epic 4)
- `[[`: trigger link suggestions

## Vim Mode (Epic 4)
- `Ctrl+Alt+V`: toggle Vim mode
- Normal mode: movement/operators from Vim mapping layer.
- Insert mode: standard editor input behavior.
- Status bar must indicate `VIM: NORMAL` or `VIM: INSERT`.

## Conflict Rules
1. Active overlay shortcuts have highest priority.
2. Editor-level shortcuts override global shortcuts only when explicitly mapped.
3. If Vim mode is enabled, Vim bindings supersede editor defaults in normal mode.
4. Any shortcut conflict must be declared in settings UI before shipping.

## Accessibility Notes
- Provide remapping file in settings for non-US keyboards.
- Reserve `Ctrl+Q` as optional quit binding only if platform standards permit.
- Ensure all core actions remain reachable without mouse.

## Acceptance Checklist
- All global shortcuts work in Linux and Windows.
- Shortcut handling is consistent across note/code panes.
- No double-trigger behavior when focus changes quickly.
