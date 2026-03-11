<script lang="ts">
	/**
	 * SearchPanel -- Sidebar search panel with debounced input and mock results.
	 * Displays a search input, mock search results with file path, line number,
	 * and matched line snippet. Will be wired to backend search_content and
	 * fuzzy_find commands in a later epic.
	 */

	import { searchStore } from './explorer-store';

	let inputValue = $state('');
	let debounceTimer = $state<ReturnType<typeof setTimeout> | null>(null);

	function handleInput(event: Event) {
		const target = event.target as HTMLInputElement;
		inputValue = target.value;

		if (debounceTimer) {
			clearTimeout(debounceTimer);
		}

		debounceTimer = setTimeout(() => {
			searchStore.setQuery(inputValue.trim());
			debounceTimer = null;
		}, 200);
	}

	function handleClear() {
		inputValue = '';
		searchStore.setQuery('');
		if (debounceTimer) {
			clearTimeout(debounceTimer);
			debounceTimer = null;
		}
	}

	function highlightMatch(text: string, query: string): { before: string; match: string; after: string } | null {
		if (!query) return null;
		const lower = text.toLowerCase();
		const idx = lower.indexOf(query.toLowerCase());
		if (idx === -1) return null;
		return {
			before: text.slice(0, idx),
			match: text.slice(idx, idx + query.length),
			after: text.slice(idx + query.length)
		};
	}

	const query = $derived(searchStore.query);
	const results = $derived(searchStore.results);
	const hasQuery = $derived(query.length > 0);
</script>

<div class="flex h-full flex-col">
	<!-- Search input -->
	<div class="relative px-3 py-2">
		<div class="relative">
			<svg
				class="pointer-events-none absolute left-2 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-text-muted"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
			>
				<circle cx="11" cy="11" r="7" />
				<path d="M16 16l4.5 4.5" />
			</svg>
			<input
				type="text"
				value={inputValue}
				oninput={handleInput}
				placeholder="Search in files..."
				class="w-full rounded-md border border-surface-border bg-surface-base py-1.5 pl-7 pr-7
					text-xs text-text-primary placeholder:text-text-muted
					focus:border-accent focus:outline-none"
			/>
			{#if inputValue.length > 0}
				<button
					onclick={handleClear}
					class="absolute right-1.5 top-1/2 flex h-4 w-4 -translate-y-1/2 items-center justify-center
						rounded text-text-muted hover:text-text-primary"
					aria-label="Clear search"
				>
					<svg class="h-3 w-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
						<path d="M18 6L6 18M6 6l12 12" />
					</svg>
				</button>
			{/if}
		</div>
	</div>

	<!-- Results area -->
	<div class="flex-1 overflow-y-auto">
		{#if !hasQuery}
			<!-- Empty state: no query -->
			<div class="flex flex-col items-center gap-2 px-3 py-8 text-center">
				<svg
					class="h-8 w-8 text-text-muted/50"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
				>
					<circle cx="11" cy="11" r="7" />
					<path d="M16 16l4.5 4.5" />
				</svg>
				<p class="text-xs text-text-muted">
					Type to search across all files in the workspace.
				</p>
			</div>
		{:else if results.length === 0}
			<!-- No results state -->
			<div class="flex flex-col items-center gap-2 px-3 py-8 text-center">
				<svg
					class="h-8 w-8 text-text-muted/50"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
				>
					<circle cx="11" cy="11" r="7" />
					<path d="M16 16l4.5 4.5" />
					<path d="M8 11h6" />
				</svg>
				<p class="text-xs text-text-muted">
					No results for "<span class="text-text-secondary">{query}</span>"
				</p>
			</div>
		{:else}
			<!-- Results list -->
			<div class="px-1 pb-2">
				<p class="px-2 pb-1.5 text-[10px] font-medium uppercase tracking-wider text-text-muted">
					{results.length} result{results.length === 1 ? '' : 's'}
				</p>
				<ul class="list-none p-0">
					{#each results as result (result.filePath + ':' + result.lineNumber)}
						{@const parts = highlightMatch(result.lineContent, query)}
						<li>
							<button
								class="group flex w-full flex-col gap-0.5 rounded-md px-2 py-1.5 text-left
									hover:bg-surface-overlay"
								aria-label="Result in {result.filePath} at line {result.lineNumber}"
							>
								<div class="flex items-baseline gap-1.5">
									<span class="truncate text-xs font-medium text-text-secondary">
										{result.fileName}
									</span>
									<span class="shrink-0 text-[10px] text-text-muted">
										:{result.lineNumber}
									</span>
								</div>
								<div class="truncate font-mono text-[11px] leading-relaxed text-text-muted">
									{#if parts}
										{parts.before}<span class="rounded-sm bg-accent/20 text-accent">{parts.match}</span>{parts.after}
									{:else}
										{result.lineContent}
									{/if}
								</div>
								<div class="truncate text-[10px] text-text-muted/70">
									{result.filePath}
								</div>
							</button>
						</li>
					{/each}
				</ul>
			</div>
		{/if}
	</div>
</div>
