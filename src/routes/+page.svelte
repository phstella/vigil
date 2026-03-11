<script lang="ts">
	import { AppShell, Sidebar, TitleBar, WorkspaceGrid } from '$lib/components/layout';
	import { PrimaryRail } from '$lib/components/chrome';
	import { EditorRouter } from '$lib/features/editor';
	import { editorStore } from '$lib/stores/editor';
	import type { Section } from '$lib/components/chrome/PrimaryRail.svelte';
	import type { EditorState } from '$lib/types/store';

	let activeSection: Section | null = $state(null);
	let sidebarOpen = $derived(activeSection !== null);

	// Subscribe to the global editor store to feed the EditorRouter.
	let editorState: EditorState = $state({
		activeFile: null,
		openFiles: [],
		isDirty: false,
		content: '',
		language: 'plaintext'
	});

	editorStore.subscribe((s) => {
		editorState = s;
	});

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
			<EditorRouter
				filePath={editorState.activeFile && !editorState.activeFile.endsWith('.md')
					? editorState.activeFile
					: null}
				content={editorState.activeFile && !editorState.activeFile.endsWith('.md')
					? editorState.content
					: ''}
			/>
		{/snippet}

		<EditorRouter
			filePath={editorState.activeFile?.endsWith('.md') ? editorState.activeFile : null}
			content={editorState.activeFile?.endsWith('.md') ? editorState.content : ''}
		/>
	</WorkspaceGrid>

	{#snippet statusbar()}
		<footer
			class="flex h-6 shrink-0 items-center border-t border-surface-border bg-surface-base px-3"
		>
			<span class="text-xs text-text-muted">Ready</span>
		</footer>
	{/snippet}
</AppShell>
