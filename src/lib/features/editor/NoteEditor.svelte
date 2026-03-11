<script lang="ts">
	/**
	 * NoteEditor -- Skeleton markdown editing surface.
	 *
	 * Displays a file path header/tab area and a monospace textarea for
	 * markdown content. This is a placeholder for future Tiptap WYSIWYG
	 * integration. Tracks dirty state via the note store.
	 */

	import { noteStore } from './note-store';

	let {
		filePath,
		content
	}: {
		filePath: string;
		content: string;
	} = $props();

	// Sync incoming props into the local note store on load.
	$effect(() => {
		noteStore.load(filePath, content);
	});

	function handleInput(e: Event) {
		const target = e.target as HTMLTextAreaElement;
		noteStore.updateContent(target.value);
	}
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
		{#if noteStore.isDirty}
			<span
				class="ml-auto h-2 w-2 shrink-0 rounded-full bg-accent"
				title="Unsaved changes"
			></span>
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
