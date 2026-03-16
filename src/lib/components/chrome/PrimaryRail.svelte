<script lang="ts">
	/**
	 * PrimaryRail -- Vertical icon rail on the far-left edge.
	 * Provides icon buttons for Explorer, Search, Graph, and Tags.
	 * Clicking an already-active section deselects it (closes the sidebar).
	 */

	export type Section = 'explorer' | 'search' | 'graph' | 'tags';

	let {
		activeSection = null,
		onSectionChange
	}: {
		activeSection?: Section | null;
		onSectionChange: (section: Section | null) => void;
	} = $props();

	function handleClick(section: Section) {
		if (activeSection === section) {
			onSectionChange(null);
		} else {
			onSectionChange(section);
		}
	}

	const sections: { id: Section; label: string }[] = [
		{ id: 'explorer', label: 'Explorer' },
		{ id: 'search', label: 'Search' },
		{ id: 'graph', label: 'Graph' },
		{ id: 'tags', label: 'Tags' }
	];
</script>

<nav
	class="flex w-12 shrink-0 flex-col items-center gap-1 border-r border-surface-border bg-surface-base pt-2"
	aria-label="Activity rail"
>
	{#each sections as section (section.id)}
		<button
			onclick={() => handleClick(section.id)}
			class="relative flex h-10 w-10 items-center justify-center rounded-md transition-colors
				{activeSection === section.id
				? 'bg-accent/15 text-accent'
				: 'text-text-muted hover:bg-surface-overlay hover:text-text-primary'}"
			aria-label={section.label}
			aria-pressed={activeSection === section.id}
		>
			{#if activeSection === section.id}
				<span
					class="absolute left-0 top-1/2 h-5 w-0.5 -translate-y-1/2 rounded-r-full bg-accent"
				></span>
			{/if}

			{#if section.id === 'explorer'}
				<svg
					class="h-5 w-5"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
				>
					<path
						d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
					/>
				</svg>
			{:else if section.id === 'search'}
				<svg
					class="h-5 w-5"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
				>
					<circle cx="11" cy="11" r="7" />
					<path d="M16 16l4.5 4.5" />
				</svg>
			{:else if section.id === 'graph'}
				<svg
					class="h-5 w-5"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
				>
					<circle cx="6" cy="6" r="2.5" />
					<circle cx="18" cy="6" r="2.5" />
					<circle cx="12" cy="18" r="2.5" />
					<path d="M8 7.5l4 8M16 7.5l-4 8M8.5 6h7" />
				</svg>
			{:else if section.id === 'tags'}
				<svg
					class="h-5 w-5"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
				>
					<path
						d="M7 7h.01M3 12l8.5-8.5a1 1 0 01.7-.3H19a1 1 0 011 1v6.8a1 1 0 01-.3.7L12 20l-9-8z"
					/>
				</svg>
			{/if}
		</button>
	{/each}
</nav>
