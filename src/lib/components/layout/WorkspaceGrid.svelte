<script lang="ts">
	// Horizontal flex layout arranging: ActivityRail (left), Sidebar (left panel),
	// EditorArea (center), and optionally a graph/right panel.
	// When a rightPanel snippet is provided, center and right are joined by a
	// resizable SplitPane divider.

	import type { Snippet } from 'svelte';
	import SplitPane from './SplitPane.svelte';

	let {
		activityRail,
		sidebar,
		children,
		rightPanel,
		splitEnabled = false,
		splitDirection = 'horizontal',
		initialSplit = 50
	}: {
		activityRail?: Snippet;
		sidebar?: Snippet;
		children: Snippet;
		rightPanel?: Snippet;
		splitEnabled?: boolean;
		splitDirection?: 'horizontal' | 'vertical';
		initialSplit?: number;
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
		{#if rightPanel && splitEnabled}
			<SplitPane direction={splitDirection} {initialSplit}>
				{#snippet first()}
					{@render children()}
				{/snippet}
				{#snippet second()}
					{@render rightPanel()}
				{/snippet}
			</SplitPane>
		{:else}
			{@render children()}
		{/if}
	</div>
</div>
