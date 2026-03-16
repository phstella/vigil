/**
 * Code editor UI state store.
 *
 * Provides local state for the CodeEditor skeleton component.
 * Tracks the active code file path, content buffer, detected language,
 * and dirty state. Will be extended with Monaco integration in a future epic.
 */

/** Map of common file extensions to language identifiers. */
const EXT_LANGUAGE_MAP: Record<string, string> = {
	ts: 'typescript',
	tsx: 'typescript',
	js: 'javascript',
	jsx: 'javascript',
	json: 'json',
	html: 'html',
	css: 'css',
	scss: 'scss',
	svelte: 'svelte',
	rs: 'rust',
	toml: 'toml',
	yaml: 'yaml',
	yml: 'yaml',
	sh: 'shell',
	bash: 'shell',
	py: 'python',
	go: 'go',
	sql: 'sql',
	xml: 'xml',
	svg: 'xml'
};

/** Detect a language identifier from a file path's extension. */
export function detectLanguage(filePath: string): string {
	const dot = filePath.lastIndexOf('.');
	if (dot === -1) return 'plaintext';
	const ext = filePath.slice(dot + 1).toLowerCase();
	return EXT_LANGUAGE_MAP[ext] ?? 'plaintext';
}

export interface CodeEditorState {
	/** Workspace-relative path of the active code file, or null. */
	filePath: string | null;
	/** Text content of the active code file. */
	content: string;
	/** Detected language identifier for syntax highlighting. */
	language: string;
	/** Whether the file has unsaved modifications. */
	isDirty: boolean;
}

function createCodeStore() {
	let filePath = $state<string | null>(null);
	let content = $state('');
	let language = $state('plaintext');
	let isDirty = $state(false);

	return {
		get filePath() {
			return filePath;
		},
		get content() {
			return content;
		},
		get language() {
			return language;
		},
		get isDirty() {
			return isDirty;
		},

		/** Load a code file into the editor. */
		load(path: string, text: string) {
			filePath = path;
			content = text;
			language = detectLanguage(path);
			isDirty = false;
		},

		/** Update the content buffer and mark as dirty. */
		updateContent(text: string) {
			content = text;
			isDirty = true;
		},

		/** Clear dirty state (e.g. after save). */
		markClean() {
			isDirty = false;
		},

		/** Reset the store to its initial empty state. */
		reset() {
			filePath = null;
			content = '';
			language = 'plaintext';
			isDirty = false;
		}
	};
}

export const codeStore = createCodeStore();
