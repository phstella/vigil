// Barrel file for shared utilities.
export { shortcutRegistry } from './shortcuts';
export type { ShortcutEntry, ShortcutRegistry } from './shortcuts';
export { markdownToHtml } from './markdown';
export {
	isMarkdownFile,
	isCodeFile,
	getEditorPane,
	getFileExtension
} from './file-routing';
export type { EditorPane } from './file-routing';
