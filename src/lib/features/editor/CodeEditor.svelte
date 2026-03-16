<script lang="ts">
	/**
	 * CodeEditor -- Monaco-powered code editing surface.
	 *
	 * Lazy-loads Monaco on first mount, applies the Vigil dark theme,
	 * and syncs content/language with the code store. The editor container
	 * auto-resizes via Monaco's `automaticLayout` option.
	 *
	 * Git gutter markers (Task 3.8): Shows added/modified/deleted line
	 * indicators in the glyph margin. Refreshes on file open, content
	 * edit (debounced), save, and backend `vigil://git-hunks` events.
	 *
	 * Performance targets (from editor-performance-budget.md):
	 * - Keystroke-to-paint: <= 16 ms p95
	 * - Scroll FPS: >= 60 FPS
	 * - File switch: <= 120 ms
	 * - Git hunk refresh: <= 200 ms for files under 5k lines
	 */

	import { onMount, onDestroy } from 'svelte';
	import { codeStore } from './code-store.svelte';
	import {
		loadMonaco,
		getDefaultEditorOptions,
		detectMonacoLanguage,
		VIGIL_THEME_NAME,
		isMonacoLoaded
	} from './monaco-config';
	import { GutterController } from '$lib/features/git/gutter';
	import { onGitHunks } from '$lib/ipc/events';
	import { perfTimer } from '$lib/utils/perf';
	import type * as Monaco from 'monaco-editor';
	import type { UnlistenFn } from '@tauri-apps/api/event';

	let {
		filePath,
		content
	}: {
		filePath: string;
		content: string;
	} = $props();

	let containerEl: HTMLDivElement | undefined = $state();
	let editor: Monaco.editor.IStandaloneCodeEditor | null = null;
	let monacoRef: typeof Monaco | null = null;
	let isLoading = $state(true);
	let loadError: string | null = $state(null);

	// Track the last filePath we set up so we can detect file switches
	let currentFilePath: string | null = null;
	let lastAppliedPropContent: string | null = null;

	// Git gutter controller
	let gutterController: GutterController | null = null;
	let unlistenGitHunks: UnlistenFn | null = null;

	// Sync incoming props into the local code store.
	$effect(() => {
		codeStore.load(filePath, content);
	});

	/**
	 * When filePath or content props change after initial mount,
	 * update the Monaco model accordingly.
	 */
	$effect(() => {
		if (!editor || !monacoRef) return;

		// Read these to track them as dependencies
		const fp = filePath;
		const ct = content;
		const model = editor.getModel();
		if (!model) return;
		const didSwitchFile = fp !== currentFilePath;

		if (didSwitchFile) {
			// File switched -- update model language.
			const lang = detectMonacoLanguage(fp);
			monacoRef.editor.setModelLanguage(model, lang);
			currentFilePath = fp;

			// Update git gutter for the new file
			gutterController?.setFilePath(fp);
		}

		// Content can arrive asynchronously after the file path changes.
		// Keep Monaco in sync whenever upstream props publish new content.
		if (didSwitchFile || ct !== lastAppliedPropContent) {
			if (model.getValue() !== ct) {
				// Avoid triggering our own onDidChangeModelContent handler with redundant writes.
				model.setValue(ct);
			}
			lastAppliedPropContent = ct;
		}
	});

	onMount(async () => {
		if (!containerEl) return;

		// First Monaco boot includes dynamic import + worker spin-up, so use a wider cold-start budget.
		const mountTimer = perfTimer('editor-mount', isMonacoLoaded() ? 120 : 500);

		try {
			monacoRef = await loadMonaco();

			const language = detectMonacoLanguage(filePath);
			const options = getDefaultEditorOptions();

			editor = monacoRef.editor.create(containerEl, {
				...options,
				value: content,
				language
			});

			// Ensure theme is applied
			monacoRef.editor.setTheme(VIGIL_THEME_NAME);

			currentFilePath = filePath;
			lastAppliedPropContent = content;

			// Initialize git gutter controller
			gutterController = new GutterController(editor);
			gutterController.setFilePath(filePath);

			// Subscribe to backend git hunk push events
			unlistenGitHunks = await onGitHunks((payload) => {
				if (payload.path === currentFilePath && gutterController) {
					gutterController.applyHunks(payload.hunks);
				}
			});

			// Sync content changes from Monaco back to the code store,
			// and schedule a debounced git gutter refresh on edits.
			editor.onDidChangeModelContent(() => {
				const value = editor?.getValue() ?? '';
				codeStore.updateContent(value);
				gutterController?.scheduleRefresh();
			});

			isLoading = false;
			mountTimer.stop();
		} catch (err) {
			console.error('[CodeEditor] Failed to load Monaco:', err);
			loadError = err instanceof Error ? err.message : 'Failed to load editor';
			isLoading = false;
			mountTimer.stop();
		}
	});

	onDestroy(() => {
		// Clean up git gutter controller
		if (gutterController) {
			gutterController.dispose();
			gutterController = null;
		}

		// Unsubscribe from git hunk events
		if (unlistenGitHunks) {
			unlistenGitHunks();
			unlistenGitHunks = null;
		}

		if (editor) {
			editor.dispose();
			editor = null;
		}
	});
</script>

<div class="flex h-full flex-col bg-surface-base">
	<!-- Tab / header bar -->
	<header
		class="flex h-9 shrink-0 items-center gap-2 border-b border-surface-border bg-surface-raised px-3"
	>
		<svg
			class="h-3.5 w-3.5 shrink-0 text-text-muted"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="1.5"
		>
			<path d="M10 20l4-16m4.5 4l4.5 4-4.5 4M5.5 8L1 12l4.5 4" />
		</svg>
		<span class="truncate text-xs font-medium text-text-secondary" title={filePath}>
			{filePath}
		</span>
		<span
			class="rounded bg-surface-overlay px-1.5 py-0.5 text-[10px] font-medium uppercase text-text-muted"
		>
			{codeStore.language}
		</span>
		{#if codeStore.isDirty}
			<span class="ml-auto h-2 w-2 shrink-0 rounded-full bg-accent" title="Unsaved changes"
			></span>
		{/if}
	</header>

	<!-- Monaco editor container -->
	<div class="relative flex-1 overflow-hidden">
		{#if isLoading}
			<div class="flex h-full items-center justify-center">
				<span class="text-sm text-text-muted">Loading editor...</span>
			</div>
		{:else if loadError}
			<div class="flex h-full items-center justify-center">
				<div class="text-center">
					<span class="text-sm text-error">Editor failed to load</span>
					<p class="mt-1 text-xs text-text-muted">{loadError}</p>
				</div>
			</div>
		{/if}
		<div
			bind:this={containerEl}
			class="absolute inset-0"
			class:invisible={isLoading || !!loadError}
			role="code"
			aria-label="Code editor"
		></div>
	</div>
</div>
