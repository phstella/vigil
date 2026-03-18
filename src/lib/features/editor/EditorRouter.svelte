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
	 *
	 * Renders a tab bar showing all open files when tabs are available.
	 *
	 * Performance (Task 3.11):
	 * - CodeEditor is dynamically imported so Monaco's ~2 MB bundle is not
	 *   included in the initial route chunk, improving cold-start time.
	 * - Pane switch render is instrumented with a perf timer.
	 */

	import NoteEditor from './NoteEditor.svelte';
	import { isMarkdownFile } from '$lib/utils/file-routing';
	import { perfTimer } from '$lib/utils/perf';
	import type { OpenFile } from '$lib/types/store';

	let {
		filePath = null,
		content = '',
		pane = 'auto',
		tabs = [],
		onactivatetab,
		onclosetab
	}: {
		filePath?: string | null;
		content?: string;
		pane?: 'note' | 'code' | 'auto';
		tabs?: OpenFile[];
		onactivatetab?: (path: string) => void;
		onclosetab?: (path: string) => void;
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

	let showTabs = $derived(tabs.length > 0);

	/** Extract the filename from a path for display in the tab. */
	function displayName(path: string): string {
		const sep = path.lastIndexOf('/');
		return sep >= 0 ? path.slice(sep + 1) : path;
	}

	function handleTabClick(path: string) {
		onactivatetab?.(path);
	}

	function handleTabClose(e: MouseEvent, path: string) {
		e.stopPropagation();
		onclosetab?.(path);
	}

	function handleTabKeydown(e: KeyboardEvent, path: string) {
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			onactivatetab?.(path);
		}
	}

	function handleCloseKeydown(e: KeyboardEvent, path: string) {
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			e.stopPropagation();
			onclosetab?.(path);
		}
	}

	// Lazy-load CodeEditor only when the code pane is actually needed.
	let CodeEditorModule = $state<Promise<typeof import('./CodeEditor.svelte')> | null>(null);
	$effect(() => {
		if (shouldRenderCode && CodeEditorModule === null) {
			CodeEditorModule = import('./CodeEditor.svelte');
		}
	});

	// Instrument pane switch rendering
	$effect(() => {
		if (filePath != null) {
			const timer = perfTimer('pane-switch-render', 120);
			// Use microtask to measure after Svelte renders the new pane
			queueMicrotask(() => {
				timer.stop();
			});
		}
	});
</script>

<div class="flex h-full flex-col">
	{#if showTabs}
		<!-- Tab bar -->
		<div
			class="flex h-8 shrink-0 items-stretch overflow-x-auto border-b border-surface-border bg-surface-raised"
			role="tablist"
			aria-label="Open files"
		>
			{#each tabs as tab (tab.path)}
				{@const isActive = tab.path === filePath}
				<div
					class="group flex max-w-[180px] shrink-0 cursor-pointer items-center gap-1 border-r border-surface-border px-2 text-xs transition-colors"
					class:bg-surface-base={isActive}
					class:text-text-primary={isActive}
					class:bg-surface-raised={!isActive}
					class:text-text-muted={!isActive}
					class:hover:bg-surface-overlay={!isActive}
					role="tab"
					aria-selected={isActive}
					tabindex={isActive ? 0 : -1}
					title={tab.path}
					onclick={() => handleTabClick(tab.path)}
					onkeydown={(e) => handleTabKeydown(e, tab.path)}
				>
					<!-- File type indicator -->
					{#if isMarkdownFile(tab.path)}
						<span class="shrink-0 text-[10px] font-semibold text-accent opacity-60">MD</span>
					{:else}
						<span class="shrink-0 text-[10px] font-semibold text-text-muted opacity-40"
							>&lt;/&gt;</span
						>
					{/if}

					<span class="truncate">{displayName(tab.path)}</span>

					<!-- Dirty indicator or close button -->
					{#if tab.isDirty}
						<span
							class="ml-auto h-2 w-2 shrink-0 rounded-full bg-accent"
							title="Unsaved changes"
						></span>
					{/if}
					<button
						class="ml-auto flex h-4 w-4 shrink-0 items-center justify-center rounded opacity-0 transition-opacity hover:bg-surface-border group-hover:opacity-100"
						class:opacity-100={isActive}
						onclick={(e) => handleTabClose(e, tab.path)}
						onkeydown={(e) => handleCloseKeydown(e, tab.path)}
						title="Close tab"
						aria-label={`Close ${displayName(tab.path)}`}
						tabindex={-1}
					>
						<svg
							class="h-3 w-3"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
						>
							<line x1="18" y1="6" x2="6" y2="18" />
							<line x1="6" y1="6" x2="18" y2="18" />
						</svg>
					</button>
				</div>
			{/each}
		</div>
	{/if}

	<!-- Editor content area -->
	<div class="min-h-0 flex-1">
		{#if shouldRenderNote}
			<NoteEditor filePath={filePath!} {content} />
		{:else if shouldRenderCode}
			{#if CodeEditorModule}
				{#await CodeEditorModule}
					<div class="flex h-full items-center justify-center bg-surface-base">
						<span class="text-sm text-text-muted">Loading editor...</span>
					</div>
				{:then mod}
					{#key filePath}
						<mod.default filePath={filePath!} {content} />
					{/key}
				{:catch}
					<div class="flex h-full items-center justify-center bg-surface-base">
						<span class="text-sm text-error">Failed to load code editor</span>
					</div>
				{/await}
			{:else}
				<div class="flex h-full items-center justify-center bg-surface-base">
					<span class="text-sm text-text-muted">Loading editor...</span>
				</div>
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
	</div>
</div>
