<script lang="ts">
	/**
	 * Sidebar -- Collapsible multi-panel area next to the activity rail.
	 * Slides open/closed with a CSS transition. Renders placeholder content
	 * for each section until real feature panels are wired in.
	 */

	import type { Snippet } from 'svelte';
	import type { Section } from '$lib/components/chrome/PrimaryRail.svelte';
	import { ExplorerPanel, SearchPanel } from '$lib/features/explorer';
	import { GraphView } from '$lib/features/graph';

	let {
		isOpen = false,
		activeSection = null,
		children,
		onMouseEnter,
		onMouseLeave
	}: {
		isOpen?: boolean;
		activeSection?: Section | null;
		children?: Snippet;
		onMouseEnter?: () => void;
		onMouseLeave?: () => void;
	} = $props();

	const sectionTitles: Record<Section, string> = {
		explorer: 'Explorer',
		search: 'Search',
		graph: 'Graph',
		tags: 'Tags'
	};
</script>

<aside
	class="shrink-0 overflow-hidden border-r border-surface-border bg-surface-raised transition-[width] duration-200 ease-in-out"
	style="width: {isOpen ? '260px' : '0px'}"
	aria-label="Sidebar"
	aria-hidden={!isOpen}
	onmouseenter={() => onMouseEnter?.()}
	onmouseleave={() => onMouseLeave?.()}
>
	<div class="flex h-full w-[260px] flex-col">
		{#if activeSection}
			<header class="flex h-9 shrink-0 items-center border-b border-surface-border px-3">
				<span class="text-xs font-semibold uppercase tracking-wider text-text-secondary">
					{sectionTitles[activeSection]}
				</span>
			</header>
		{/if}

		<div class="flex-1 overflow-y-auto">
			{#if children}
				{@render children()}
			{:else if activeSection === 'explorer'}
				<ExplorerPanel />
			{:else if activeSection === 'search'}
				<SearchPanel />
			{:else if activeSection === 'graph'}
				<GraphView />
			{:else if activeSection === 'tags'}
				<div class="p-3">
					<p class="text-sm text-text-muted">Tag browser will appear here.</p>
				</div>
			{/if}
		</div>
	</div>
</aside>
