/**
 * Monaco editor configuration for Vigil.
 *
 * Provides a performance-focused default configuration, a custom dark theme
 * aligned with Vigil design tokens, and helpers for language detection.
 *
 * Performance notes (per docs/specs/editor-performance-budget.md):
 * - Keystroke-to-paint latency: <= 16 ms p95
 * - Scroll FPS: >= 60 FPS target, >= 45 FPS floor
 * - File switch: <= 120 ms
 * - Monaco is lazy-loaded to keep the initial bundle lean.
 */

import type * as Monaco from 'monaco-editor';

// ---------------------------------------------------------------------------
// Vigil dark theme definition
// ---------------------------------------------------------------------------

/** Custom Monaco theme matching Vigil design tokens from app.css. */
export const VIGIL_THEME_NAME = 'vigil-dark';

export const vigilDarkTheme: Monaco.editor.IStandaloneThemeData = {
	base: 'vs-dark',
	inherit: true,
	rules: [
		{ token: '', foreground: 'e4e4e7' }, // --color-text-primary
		{ token: 'comment', foreground: '71717a', fontStyle: 'italic' }, // --color-text-muted
		{ token: 'keyword', foreground: '6d9eff' }, // --color-accent
		{ token: 'string', foreground: '4ade80' }, // --color-success
		{ token: 'number', foreground: 'facc15' }, // --color-warning
		{ token: 'type', foreground: '8bb4ff' }, // --color-accent-strong
		{ token: 'function', foreground: '6d9eff' }, // --color-accent
		{ token: 'variable', foreground: 'e4e4e7' }, // --color-text-primary
		{ token: 'operator', foreground: 'a1a1aa' }, // --color-text-secondary
		{ token: 'delimiter', foreground: 'a1a1aa' } // --color-text-secondary
	],
	colors: {
		'editor.background': '#0e0e10', // --color-surface-base
		'editor.foreground': '#e4e4e7', // --color-text-primary
		'editor.lineHighlightBackground': '#18181b', // --color-surface-raised
		'editor.selectionBackground': '#4a6fa540', // --color-accent-muted + alpha
		'editor.inactiveSelectionBackground': '#4a6fa520',
		'editorLineNumber.foreground': '#71717a', // --color-text-muted
		'editorLineNumber.activeForeground': '#a1a1aa', // --color-text-secondary
		'editorCursor.foreground': '#6d9eff', // --color-accent
		'editorIndentGuide.background': '#2e2e33', // --color-surface-border
		'editorIndentGuide.activeBackground': '#71717a',
		'editor.findMatchBackground': '#6d9eff30',
		'editor.findMatchHighlightBackground': '#6d9eff15',
		'editorBracketMatch.background': '#6d9eff20',
		'editorBracketMatch.border': '#6d9eff40',
		'editorGutter.background': '#0e0e10', // match editor bg
		'editorWidget.background': '#18181b', // --color-surface-raised
		'editorWidget.border': '#2e2e33', // --color-surface-border
		'input.background': '#222225', // --color-surface-overlay
		'input.border': '#2e2e33',
		'input.foreground': '#e4e4e7',
		'list.activeSelectionBackground': '#6d9eff20',
		'list.hoverBackground': '#18181b',
		'scrollbarSlider.background': '#71717a30',
		'scrollbarSlider.hoverBackground': '#71717a50',
		'scrollbarSlider.activeBackground': '#71717a70'
	}
};

// ---------------------------------------------------------------------------
// Performance-focused default editor options
// ---------------------------------------------------------------------------

/**
 * Default Monaco editor options optimized for performance.
 *
 * Key choices:
 * - minimap disabled: reduces GPU/CPU overhead for large files
 * - smooth scrolling off: avoids animation overhead
 * - render whitespace minimal: less decoration overhead
 * - fixed font size/family matching design tokens
 */
export function getDefaultEditorOptions(
	overrides?: Partial<Monaco.editor.IStandaloneEditorConstructionOptions>
): Monaco.editor.IStandaloneEditorConstructionOptions {
	return {
		// Theme
		theme: VIGIL_THEME_NAME,

		// Typography (matching design tokens)
		fontSize: 14,
		fontFamily: "'JetBrains Mono', 'Fira Code', ui-monospace, monospace",
		fontLigatures: true,
		lineHeight: 20,

		// Performance: disable heavyweight features
		minimap: { enabled: false },
		smoothScrolling: false,
		cursorSmoothCaretAnimation: 'off',
		renderWhitespace: 'none',
		renderLineHighlight: 'line',
		occurrencesHighlight: 'off',
		selectionHighlight: false,
		matchBrackets: 'near',

		// Scrolling performance
		scrollBeyondLastLine: false,
		fastScrollSensitivity: 5,

		// Line numbers and gutter
		lineNumbers: 'on',
		glyphMargin: true, // needed for git gutter decorations (Task 3.6)
		folding: true,
		foldingStrategy: 'indentation',

		// Editor behavior
		automaticLayout: true, // auto-resize on container resize
		wordWrap: 'off',
		tabSize: 2,
		insertSpaces: false,
		detectIndentation: true,
		bracketPairColorization: { enabled: true },

		// Suggestions and intellisense (lightweight for code files)
		quickSuggestions: false,
		suggestOnTriggerCharacters: false,
		parameterHints: { enabled: false },
		hover: { enabled: true, delay: 300 },

		// Accessibility
		accessibilitySupport: 'off', // improves performance, users can enable

		// Padding
		padding: { top: 8, bottom: 8 },

		// Read-only management (will be toggled per file)
		readOnly: false,

		// Apply caller overrides
		...overrides
	};
}

// ---------------------------------------------------------------------------
// Language detection
// ---------------------------------------------------------------------------

/** Map of file extensions to Monaco language identifiers. */
const EXT_TO_MONACO_LANGUAGE: Record<string, string> = {
	ts: 'typescript',
	tsx: 'typescript',
	js: 'javascript',
	jsx: 'javascript',
	mjs: 'javascript',
	cjs: 'javascript',
	json: 'json',
	jsonc: 'json',
	html: 'html',
	htm: 'html',
	css: 'css',
	scss: 'scss',
	less: 'less',
	xml: 'xml',
	svg: 'xml',
	yaml: 'yaml',
	yml: 'yaml',
	toml: 'ini', // Monaco doesn't have a toml mode, ini is closest
	rs: 'rust',
	py: 'python',
	go: 'go',
	java: 'java',
	kt: 'kotlin',
	c: 'c',
	cpp: 'cpp',
	h: 'cpp',
	hpp: 'cpp',
	cs: 'csharp',
	rb: 'ruby',
	php: 'php',
	sh: 'shell',
	bash: 'shell',
	zsh: 'shell',
	fish: 'shell',
	sql: 'sql',
	graphql: 'graphql',
	gql: 'graphql',
	dockerfile: 'dockerfile',
	lua: 'lua',
	r: 'r',
	swift: 'swift',
	md: 'markdown',
	mdx: 'markdown'
};

/**
 * Detect the Monaco language identifier from a file path.
 * Falls back to 'plaintext' for unknown extensions.
 */
export function detectMonacoLanguage(filePath: string): string {
	// Handle files without extensions but with known names
	const basename = filePath.split('/').pop() ?? '';
	const lowerBasename = basename.toLowerCase();

	if (lowerBasename === 'dockerfile') return 'dockerfile';
	if (lowerBasename === 'makefile' || lowerBasename === 'gnumakefile') return 'shell';

	const dot = basename.lastIndexOf('.');
	if (dot === -1) return 'plaintext';
	const ext = basename.slice(dot + 1).toLowerCase();
	return EXT_TO_MONACO_LANGUAGE[ext] ?? 'plaintext';
}

// ---------------------------------------------------------------------------
// Monaco loader
// ---------------------------------------------------------------------------

/** Cached reference to avoid re-importing. */
let monacoInstance: typeof Monaco | null = null;
let monacoEnvironmentConfigured = false;
let monacoWorkerConstructorsPromise: Promise<MonacoWorkerConstructors> | null = null;

type WorkerCtor = new () => Worker;
interface MonacoWorkerConstructors {
	EditorWorker: WorkerCtor;
	JsonWorker: WorkerCtor;
	CssWorker: WorkerCtor;
	HtmlWorker: WorkerCtor;
	TsWorker: WorkerCtor;
}

async function loadMonacoWorkerConstructors(): Promise<MonacoWorkerConstructors> {
	if (monacoWorkerConstructorsPromise) {
		return monacoWorkerConstructorsPromise;
	}

	monacoWorkerConstructorsPromise = Promise.all([
		import('monaco-editor/esm/vs/editor/editor.worker?worker'),
		import('monaco-editor/esm/vs/language/json/json.worker?worker'),
		import('monaco-editor/esm/vs/language/css/css.worker?worker'),
		import('monaco-editor/esm/vs/language/html/html.worker?worker'),
		import('monaco-editor/esm/vs/language/typescript/ts.worker?worker')
	]).then(([editor, json, css, html, ts]) => ({
		EditorWorker: editor.default,
		JsonWorker: json.default,
		CssWorker: css.default,
		HtmlWorker: html.default,
		TsWorker: ts.default
	}));

	return monacoWorkerConstructorsPromise;
}

async function configureMonacoEnvironment() {
	if (monacoEnvironmentConfigured || typeof window === 'undefined') return;
	monacoEnvironmentConfigured = true;
	const workers = await loadMonacoWorkerConstructors();

	const target = globalThis as typeof globalThis & {
		MonacoEnvironment?: {
			getWorker: (_moduleId: string, label: string) => Worker;
		};
	};

	target.MonacoEnvironment = {
		getWorker(_moduleId: string, label: string): Worker {
			switch (label) {
				case 'json':
					return new workers.JsonWorker();
				case 'css':
				case 'scss':
				case 'less':
					return new workers.CssWorker();
				case 'html':
				case 'handlebars':
				case 'razor':
					return new workers.HtmlWorker();
				case 'typescript':
				case 'javascript':
					return new workers.TsWorker();
				default:
					return new workers.EditorWorker();
			}
		}
	};
}

export function isMonacoLoaded(): boolean {
	return monacoInstance !== null;
}

/**
 * Lazily load the Monaco editor module.
 *
 * The dynamic import ensures Monaco's ~2 MB of JS is not included in the
 * initial bundle, meeting the performance budget requirement of lazy-loading
 * heavy editor modules.
 */
export async function loadMonaco(): Promise<typeof Monaco> {
	if (monacoInstance) return monacoInstance;
	await configureMonacoEnvironment();

	const monaco = await import('monaco-editor');
	monacoInstance = monaco;

	// Register Vigil theme once on first load
	monaco.editor.defineTheme(VIGIL_THEME_NAME, vigilDarkTheme);

	return monaco;
}
