// Barrel file for editor feature.
export { default as EditorRouter } from './EditorRouter.svelte';
export { default as NoteEditor } from './NoteEditor.svelte';
export { default as CodeEditor } from './CodeEditor.svelte';
export { noteStore } from './note-store';
export type { NoteEditorState, NoteViewMode } from './note-store';
export { codeStore, detectLanguage } from './code-store';
export type { CodeEditorState } from './code-store';
