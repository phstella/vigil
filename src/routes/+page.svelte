<script lang="ts">
	import { AppShell, Sidebar, TitleBar, WorkspaceGrid } from '$lib/components/layout';
	import { PrimaryRail } from '$lib/components/chrome';
	import { EditorRouter } from '$lib/features/editor';
	import { Omnibar } from '$lib/features/omnibar';
	import { StatusBar } from '$lib/features/status';
	import { editorStore } from '$lib/stores/editor';
	import { uiStore } from '$lib/stores/ui';
	import type { Section } from '$lib/components/chrome/PrimaryRail.svelte';
	import type { EditorState, UiState } from '$lib/types/store';

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

	// Subscribe to the UI store to track omnibar visibility.
	let uiState: UiState = $state({
		sidebarOpen: true,
		sidebarSection: 'explorer',
		omnibarOpen: false,
		rightPanelOpen: false
	});

	uiStore.subscribe((s) => {
		uiState = s;
	});

	function handleSectionChange(section: Section | null) {
		activeSection = section;
	}

	function handleOmnibarClose() {
		if (uiState.omnibarOpen) {
			uiStore.toggleOmnibar();
		}
	}

	function handleOmnibarSelect(path: string) {
		// TODO: Wire to editor open flow once IPC is available.
		console.log('[omnibar] selected:', path);
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
		<div class="relative">
			<StatusBar />
			<!-- Temporary toggle: will be replaced by Ctrl+P shortcut in ticket 2.12 -->
			<button
				class="absolute right-20 top-0 z-raised flex h-6 items-center px-2 text-xs text-text-muted transition-colors hover:text-accent"
				onclick={() => uiStore.toggleOmnibar()}
				type="button"
				aria-label="Toggle omnibar"
			>
				Ctrl+P
			</button>
		</div>
	{/snippet}
</AppShell>

<!-- Omnibar overlay, rendered outside the AppShell so it floats above everything -->
{#if uiState.omnibarOpen}
	<Omnibar onclose={handleOmnibarClose} onselect={handleOmnibarSelect} />
{/if}
