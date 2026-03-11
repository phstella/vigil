<script lang="ts">
	import { AppShell, TitleBar, WorkspaceGrid } from '$lib/components/layout';
</script>

<svelte:head>
	<title>Vigil</title>
	<meta name="description" content="Lightning-fast editing. Interconnected thinking." />
</svelte:head>

<AppShell>
	{#snippet titlebar()}
		<TitleBar />
	{/snippet}

	<WorkspaceGrid>
		<div class="flex flex-1 items-center justify-center">
			<div class="text-center">
				<h1 class="text-2xl font-semibold text-text-primary">Vigil</h1>
				<p class="mt-1 text-sm text-text-muted">Open a workspace to get started</p>
			</div>
		</div>
	</WorkspaceGrid>

	{#snippet statusbar()}
		<footer
			class="flex h-6 shrink-0 items-center border-t border-surface-border bg-surface-base px-3"
		>
			<span class="text-xs text-text-muted">Ready</span>
		</footer>
	{/snippet}
</AppShell>
