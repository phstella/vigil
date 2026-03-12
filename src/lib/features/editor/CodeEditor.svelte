<script lang="ts">
	/**
	 * CodeEditor -- Monaco-powered code editing surface.
	 *
	 * Lazy-loads Monaco on first mount, applies the Vigil dark theme,
	 * and syncs content/language with the code store. The editor container
	 * auto-resizes via Monaco's `automaticLayout` option.
	 *
	 * Performance targets (from editor-performance-budget.md):
	 * - Keystroke-to-paint: <= 16 ms p95
	 * - Scroll FPS: >= 60 FPS
	 * - File switch: <= 120 ms
	 */

	import { onMount, onDestroy } from 'svelte';
	import { codeStore } from './code-store';
	import {
		loadMonaco,
		getDefaultEditorOptions,
		detectMonacoLanguage,
		VIGIL_THEME_NAME
	} from './monaco-config';
	import type * as Monaco from 'monaco-editor';

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

		if (fp !== currentFilePath) {
			// File switched -- update model language and content
			const lang = detectMonacoLanguage(fp);
			const model = editor.getModel();
			if (model) {
				monacoRef.editor.setModelLanguage(model, lang);
				// Avoid triggering our own onDidChangeModelContent handler
				model.setValue(ct);
			}
			currentFilePath = fp;
		}
	});

	onMount(async () => {
		if (!containerEl) return;

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

			// Sync content changes from Monaco back to the code store.
			editor.onDidChangeModelContent(() => {
				const value = editor?.getValue() ?? '';
				codeStore.updateContent(value);
			});

			isLoading = false;
		} catch (err) {
			console.error('[CodeEditor] Failed to load Monaco:', err);
			loadError = err instanceof Error ? err.message : 'Failed to load editor';
			isLoading = false;
		}
	});

	onDestroy(() => {
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
			<span
				class="ml-auto h-2 w-2 shrink-0 rounded-full bg-accent"
				title="Unsaved changes"
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
