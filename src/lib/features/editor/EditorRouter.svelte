<script lang="ts">
	/**
	 * EditorRouter -- Routes to NoteEditor or CodeEditor based on file extension.
	 *
	 * Supports two modes via the `pane` prop:
	 * - `"note"` (center pane): only renders NoteEditor for `.md` files.
	 * - `"code"` (right pane): only renders CodeEditor for non-`.md` files.
	 * - `"auto"` (default): routes `.md` to NoteEditor, everything else to CodeEditor.
	 *
	 * When no file is selected, displays an empty-state message.
	 */

	import NoteEditor from './NoteEditor.svelte';
	import CodeEditor from './CodeEditor.svelte';
	import { isMarkdownFile } from '$lib/utils/file-routing';

	let {
		filePath = null,
		content = '',
		pane = 'auto'
	}: {
		filePath?: string | null;
		content?: string;
		pane?: 'note' | 'code' | 'auto';
	} = $props();

	let shouldRenderNote = $derived(
		filePath != null &&
			((pane === 'note' && isMarkdownFile(filePath)) ||
				(pane === 'auto' && isMarkdownFile(filePath)))
	);

	let shouldRenderCode = $derived(
		filePath != null &&
			((pane === 'code' && !isMarkdownFile(filePath)) ||
				(pane === 'auto' && !isMarkdownFile(filePath)))
	);
</script>

{#if shouldRenderNote}
	<NoteEditor filePath={filePath!} {content} />
{:else if shouldRenderCode}
	<CodeEditor filePath={filePath!} {content} />
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
			<p class="mt-3 text-sm text-text-muted">
				{#if pane === 'note'}
					Open a markdown file to start writing
				{:else if pane === 'code'}
					Open a code file to start editing
				{:else}
					Open a file to start editing
				{/if}
			</p>
		</div>
	</div>
{/if}
