<script lang="ts">
	/**
	 * BacklinksPanel -- Collapsible panel showing notes that link to the active file.
	 *
	 * Displays backlink records with source path and context snippet.
	 * Click on a backlink to navigate to the source note.
	 */

	import { linksStore } from './links-store.svelte';

	let {
		onNavigate
	}: {
		/** Called when the user clicks a backlink to navigate to the source note. */
		onNavigate?: (path: string) => void;
	} = $props();

	let count = $derived(linksStore.backlinks.length);

	function handleToggle() {
		linksStore.toggleCollapsed();
	}

	function handleClick(path: string) {
		onNavigate?.(path);
	}
</script>

<div class="border-t border-surface-border bg-surface-base">
	<!-- Header: toggle collapse -->
	<button
		class="flex w-full items-center gap-2 px-4 py-2 text-left text-xs font-medium uppercase tracking-wide text-text-muted transition-colors hover:bg-surface-overlay hover:text-text-secondary"
		onclick={handleToggle}
		aria-expanded={!linksStore.isCollapsed}
		aria-controls="backlinks-list"
	>
		<svg
			class="h-3 w-3 shrink-0 transition-transform"
			class:rotate-90={!linksStore.isCollapsed}
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
		>
			<polyline points="9 18 15 12 9 6" />
		</svg>
		<span>Backlinks</span>
		<span class="ml-auto tabular-nums text-text-muted">
			{#if linksStore.isLoading}
				...
			{:else}
				{count}
			{/if}
		</span>
	</button>

	<!-- Backlink list -->
	{#if !linksStore.isCollapsed}
		<div
			id="backlinks-list"
			class="max-h-48 overflow-y-auto px-2 pb-2"
			role="list"
			aria-label="Backlinks to this note"
		>
			{#if linksStore.isLoading}
				<div class="px-2 py-3 text-center text-xs text-text-muted">
					Loading backlinks...
				</div>
			{:else if linksStore.lastError}
				<div class="px-2 py-3 text-center text-xs text-red-400">
					{linksStore.lastError}
				</div>
			{:else if count === 0}
				<div class="px-2 py-3 text-center text-xs text-text-muted">
					No notes link to this file
				</div>
			{:else}
				{#each linksStore.backlinks as backlink (backlink.source_path + backlink.context_snippet)}
					<button
						class="group w-full cursor-pointer rounded px-2 py-1.5 text-left transition-colors hover:bg-surface-overlay"
						onclick={() => handleClick(backlink.source_path)}
					>
						<div class="flex items-center gap-1.5">
							<svg
								class="h-3 w-3 shrink-0 text-accent-muted"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
							>
								<path
									d="M10 13a5 5 0 007.54.54l3-3a5 5 0 00-7.07-7.07l-1.72 1.71"
								/>
								<path
									d="M14 11a5 5 0 00-7.54-.54l-3 3a5 5 0 007.07 7.07l1.71-1.71"
								/>
							</svg>
							<span
								class="truncate text-xs font-medium text-text-secondary group-hover:text-text-primary"
							>
								{backlink.source_path}
							</span>
						</div>
						{#if backlink.context_snippet}
							<p
								class="mt-0.5 truncate pl-[18px] text-[11px] leading-relaxed text-text-muted"
							>
								{backlink.context_snippet}
							</p>
						{/if}
					</button>
				{/each}
			{/if}
		</div>
	{/if}
</div>
