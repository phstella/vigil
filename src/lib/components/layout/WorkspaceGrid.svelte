<script lang="ts">
	// Horizontal flex layout arranging: ActivityRail (left), Sidebar (left panel),
	// EditorArea (center), and optionally a graph/right panel.

	import type { Snippet } from 'svelte';

	let {
		activityRail,
		sidebar,
		children,
		rightPanel
	}: {
		activityRail?: Snippet;
		sidebar?: Snippet;
		children: Snippet;
		rightPanel?: Snippet;
	} = $props();
</script>

<div class="flex min-h-0 flex-1">
	{#if activityRail}
		{@render activityRail()}
	{/if}

	{#if sidebar}
		{@render sidebar()}
	{/if}

	<div class="flex min-w-0 flex-1 flex-col">
		{@render children()}
	</div>

	{#if rightPanel}
		{@render rightPanel()}
	{/if}
</div>
