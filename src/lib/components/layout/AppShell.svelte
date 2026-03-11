<script lang="ts">
	// Top-level app shell arranging TitleBar (top), main content (middle), StatusBar slot (bottom).

	import type { Snippet } from 'svelte';

	let { titlebar, children, statusbar }: { titlebar: Snippet; children: Snippet; statusbar?: Snippet } =
		$props();
</script>

<div class="flex h-screen flex-col overflow-hidden bg-surface-base text-text-primary">
	{@render titlebar()}

	<main class="flex min-h-0 flex-1">
		{@render children()}
	</main>

	{#if statusbar}
		{@render statusbar()}
	{/if}
</div>
