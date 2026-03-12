<script lang="ts">
	/**
	 * NoteEditor -- Markdown editing surface with IPC-backed open/save/autosave.
	 *
	 * Displays a file path header/tab area with dirty indicator and a monospace
	 * textarea for markdown content. Loads file content via read_file IPC on
	 * open, saves via write_file IPC, and autosaves after a 2-second debounce.
	 * Tracks dirty state and etag for optimistic concurrency.
	 *
	 * This is a placeholder textarea surface; Tiptap WYSIWYG integration is
	 * deferred to task 3.3.
	 */

	import { onDestroy } from 'svelte';
	import { noteStore } from './note-store';

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
		{#if noteStore.isSaving}
			<span
				class="ml-auto text-[10px] font-medium text-text-muted"
				title="Saving..."
			>saving</span>
		{:else if noteStore.isDirty}
			<span
				class="ml-auto h-2 w-2 shrink-0 rounded-full bg-accent"
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

	<!-- Markdown editing area (placeholder for Tiptap) -->
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
</div>
