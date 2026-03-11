<script lang="ts">
	/**
	 * EditorRouter -- Routes to NoteEditor or CodeEditor based on file extension.
	 *
	 * - `.md` files are routed to NoteEditor (markdown/Tiptap placeholder).
	 * - All other files are routed to CodeEditor (Monaco placeholder).
	 * - When no file is selected, displays an empty-state message.
	 */

	import NoteEditor from './NoteEditor.svelte';
	import CodeEditor from './CodeEditor.svelte';

	let {
		filePath = null,
		content = ''
	}: {
		filePath?: string | null;
		content?: string;
	} = $props();

	let isMarkdown = $derived(filePath?.endsWith('.md') ?? false);
</script>

{#if filePath}
	{#if isMarkdown}
		<NoteEditor {filePath} {content} />
	{:else}
		<CodeEditor {filePath} {content} />
	{/if}
{:else}
	<div class="flex h-full items-center justify-center bg-surface-base p-4">
		<div class="text-center">
			<svg
				class="mx-auto h-10 w-10 text-text-muted opacity-40"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1"
			>
				<path d="M13 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V9z" />
				<polyline points="13 2 13 9 20 9" />
			</svg>
			<p class="mt-3 text-sm text-text-muted">Open a file to start editing</p>
		</div>
	</div>
{/if}
