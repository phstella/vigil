<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { AppShell, Sidebar, TitleBar, WorkspaceGrid } from '$lib/components/layout';
	import { PrimaryRail } from '$lib/components/chrome';
	import { EditorRouter } from '$lib/features/editor';
	import { Omnibar } from '$lib/features/omnibar';
	import { StatusBar } from '$lib/features/status';
	import { editorStore } from '$lib/stores/editor';
	import { uiStore } from '$lib/stores/ui';
	import { shortcutRegistry } from '$lib/utils';
	import { noteStore } from '$lib/features/editor/note-store';
	import { codeStore } from '$lib/features/editor/code-store';
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

	/** Placeholder save action -- logs and clears dirty state. */
	function saveCurrentFile() {
		if (noteStore.isDirty) {
			console.log('[shortcut] save note:', noteStore.filePath);
			noteStore.markClean();
			return;
		}
		if (codeStore.isDirty) {
			console.log('[shortcut] save code:', codeStore.filePath);
			codeStore.markClean();
			return;
		}
		console.log('[shortcut] save: nothing to save');
	}

	/** Placeholder new note action. */
	function createNewNote() {
		// TODO: Wire to create_note IPC command once available.
		console.log('[shortcut] create new note');
	}

	onMount(() => {
		shortcutRegistry.register('ctrl+p', () => uiStore.toggleOmnibar(), { global: true });
		shortcutRegistry.register('ctrl+s', saveCurrentFile);
		shortcutRegistry.register('ctrl+b', () => uiStore.toggleSidebar());
		shortcutRegistry.register('ctrl+n', createNewNote);
	});

	onDestroy(() => {
		shortcutRegistry.unregister('ctrl+p');
		shortcutRegistry.unregister('ctrl+s');
		shortcutRegistry.unregister('ctrl+b');
		shortcutRegistry.unregister('ctrl+n');
	});
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
		<StatusBar />
	{/snippet}
</AppShell>

<!-- Omnibar overlay, rendered outside the AppShell so it floats above everything -->
{#if uiState.omnibarOpen}
	<Omnibar onclose={handleOmnibarClose} onselect={handleOmnibarSelect} />
{/if}
