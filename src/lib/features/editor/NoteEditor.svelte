<script lang="ts">
	/**
	 * NoteEditor -- Markdown editing surface with IPC-backed open/save/autosave
	 * and live preview toggle.
	 *
	 * Displays a file path header/tab area with dirty/saving indicators and a
	 * view-mode toggle button. In 'edit' mode: shows a monospace textarea for
	 * raw markdown editing. In 'preview' mode: shows rendered markdown via
	 * InlinePreview. Raw text is always preserved as the source of truth.
	 * Toggle via Ctrl+. or the header button.
	 */

	import { onDestroy } from 'svelte';
	import { noteStore } from './note-store';
	import InlinePreview from '$lib/features/preview/InlinePreview.svelte';

	let {
		filePath,
		content
	}: {
		filePath: string;
		content: string;
	} = $props();

	// Track which file path we last loaded so we can detect prop changes.
	let loadedPath: string | null = null;

	// When filePath or content props change, load into the note store.
	$effect(() => {
		if (filePath && filePath !== loadedPath) {
			loadedPath = filePath;
			// Open via IPC to get etag for optimistic concurrency.
			// Fall back to direct load if IPC is unavailable (dev/test).
			noteStore.open(filePath).catch(() => {
				noteStore.load(filePath, content);
			});
		}
	});

	function handleInput(e: Event) {
		const target = e.target as HTMLTextAreaElement;
		noteStore.updateContent(target.value);
	}

	function handleToggle() {
		noteStore.toggleViewMode();
	}

	let isPreview = $derived(noteStore.viewMode === 'preview');

	// Clean up autosave timer on destroy.
	onDestroy(() => {
		noteStore.cancelAutosave();
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
			<path d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
		</svg>
		<span class="truncate text-xs font-medium text-text-secondary" title={filePath}>
			{filePath}
		</span>

		<!-- View mode toggle button -->
		<button
			class="ml-auto flex items-center gap-1 rounded px-1.5 py-0.5 text-[10px] font-medium uppercase transition-colors hover:bg-surface-overlay"
			class:text-accent={isPreview}
			class:text-text-muted={!isPreview}
			onclick={handleToggle}
			title={isPreview ? 'Switch to edit mode (Ctrl+.)' : 'Switch to preview mode (Ctrl+.)'}
			aria-label={isPreview ? 'Switch to edit mode' : 'Switch to preview mode'}
		>
			{#if isPreview}
				<!-- Pencil icon for "switch to edit" -->
				<svg class="h-3 w-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7" />
					<path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z" />
				</svg>
				Edit
			{:else}
				<!-- Eye icon for "switch to preview" -->
				<svg class="h-3 w-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
					<circle cx="12" cy="12" r="3" />
				</svg>
				Preview
			{/if}
		</button>

		{#if noteStore.isSaving}
			<span
				class="text-[10px] font-medium text-text-muted"
				title="Saving..."
			>saving</span>
		{:else if noteStore.isDirty}
			<span
				class="h-2 w-2 shrink-0 rounded-full bg-accent"
				title="Unsaved changes"
			></span>
		{/if}
		{#if noteStore.lastError}
			<span
				class="ml-1 truncate text-[10px] text-red-400"
				title={noteStore.lastError}
			>{noteStore.lastError}</span>
		{/if}
	</header>

	<!-- Content area: edit or preview based on viewMode -->
	{#if isPreview}
		<div class="flex-1 overflow-auto">
			<InlinePreview content={noteStore.content} />
		</div>
	{:else}
		<div class="flex-1 overflow-auto p-4">
			<textarea
				class="h-full w-full resize-none border-none bg-transparent font-mono text-sm leading-relaxed text-text-primary outline-none placeholder:text-text-muted"
				placeholder="Start writing..."
				value={noteStore.content}
				oninput={handleInput}
				spellcheck="true"
				aria-label="Markdown editor"
			></textarea>
		</div>
	{/if}
</div>
