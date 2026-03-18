/**
 * File-type routing utilities.
 *
 * Classifies files by extension to determine which editor pane they belong in:
 * - `.md` files open in the center note pane (NoteEditor / Tiptap).
 * - All other text files open in the right code pane (CodeEditor / Monaco).
 *
 * This is the single source of truth for file-type routing decisions.
 */

/** The two editor pane targets. */
export type EditorPane = 'note' | 'code';

/** Markdown extensions treated as note files. */
const MARKDOWN_EXTENSIONS = new Set(['md', 'markdown', 'mdx']);

/**
 * Extract the file extension (without dot, lowercased) from a path.
 * Returns empty string if no extension is found.
 */
export function getFileExtension(filePath: string): string {
	const dot = filePath.lastIndexOf('.');
	if (dot === -1 || dot === filePath.length - 1) return '';
	// Handle paths like ".vigilrc" (hidden files with no real extension)
	const base = filePath.lastIndexOf('/');
	if (dot === base + 1) return '';
	return filePath.slice(dot + 1).toLowerCase();
}

/** Returns true if the file should open in the center note pane. */
export function isMarkdownFile(filePath: string): boolean {
	return MARKDOWN_EXTENSIONS.has(getFileExtension(filePath));
}

/** Returns true if the file should open in the right code pane. */
export function isCodeFile(filePath: string): boolean {
	return !isMarkdownFile(filePath);
}

/**
 * Determine which editor pane a file should be routed to.
 * `.md` / `.markdown` / `.mdx` -> 'note' (center pane).
 * Everything else -> 'code' (right pane).
 */
export function getEditorPane(filePath: string): EditorPane {
	return isMarkdownFile(filePath) ? 'note' : 'code';
}
