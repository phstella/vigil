<script lang="ts">
	/**
	 * CodeEditor -- Skeleton code editing surface with line numbers.
	 *
	 * Displays file path and detected language in a header, a simple
	 * line-number gutter alongside a monospace textarea. This is a
	 * placeholder for future Monaco integration.
	 */

	import { codeStore } from './code-store';

	let {
		filePath,
		content
	}: {
		filePath: string;
		content: string;
	} = $props();

	// Sync incoming props into the local code store on load.
	$effect(() => {
		codeStore.load(filePath, content);
	});

	let lineCount = $derived(Math.max(1, codeStore.content.split('\n').length));

	function handleInput(e: Event) {
		const target = e.target as HTMLTextAreaElement;
		codeStore.updateContent(target.value);
	}

	/** Keep the gutter scroll in sync with the textarea. */
	let gutterEl: HTMLDivElement | undefined = $state();
	function handleScroll(e: Event) {
		if (gutterEl) {
			gutterEl.scrollTop = (e.target as HTMLTextAreaElement).scrollTop;
		}
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

	<!-- Code editing area with line-number gutter (placeholder for Monaco) -->
	<div class="flex flex-1 overflow-hidden">
		<!-- Line number gutter -->
		<div
			bind:this={gutterEl}
			class="shrink-0 select-none overflow-hidden border-r border-surface-border bg-surface-raised py-2 pr-2 text-right"
			aria-hidden="true"
		>
			{#each Array.from({ length: lineCount }, (__, n) => n + 1) as lineNum (lineNum)}
				<div class="px-2 font-mono text-xs leading-5 text-text-muted">
					{lineNum}
				</div>
			{/each}
		</div>

		<!-- Code textarea -->
		<textarea
			class="flex-1 resize-none border-none bg-transparent py-2 pl-3 font-mono text-sm leading-5 text-text-primary outline-none placeholder:text-text-muted"
			placeholder="// Code goes here..."
			value={codeStore.content}
			oninput={handleInput}
			onscroll={handleScroll}
			spellcheck="false"
			aria-label="Code editor"
			wrap="off"
		></textarea>
	</div>
</div>
