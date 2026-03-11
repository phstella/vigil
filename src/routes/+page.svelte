<script lang="ts">
	import { AppShell, Sidebar, TitleBar, WorkspaceGrid } from '$lib/components/layout';
	import { PrimaryRail } from '$lib/components/chrome';
	import type { Section } from '$lib/components/chrome/PrimaryRail.svelte';

	let activeSection: Section | null = $state(null);
	let sidebarOpen = $derived(activeSection !== null);

	function handleSectionChange(section: Section | null) {
		activeSection = section;
	}
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
		{#snippet activityRail()}
			<PrimaryRail {activeSection} onSectionChange={handleSectionChange} />
		{/snippet}

		{#snippet sidebar()}
			<Sidebar isOpen={sidebarOpen} {activeSection} />
		{/snippet}

		{#snippet rightPanel()}
			<div class="flex h-full items-center justify-center bg-surface-raised p-4">
				<div class="text-center">
					<p class="text-sm font-medium text-text-secondary">Code Pane</p>
					<p class="mt-1 text-xs text-text-muted">Right split panel</p>
				</div>
			</div>
		{/snippet}

		<div class="flex h-full items-center justify-center p-4">
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
