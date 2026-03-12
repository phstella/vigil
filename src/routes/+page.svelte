<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { AppShell, Sidebar, TitleBar, WorkspaceGrid } from '$lib/components/layout';
	import { PrimaryRail } from '$lib/components/chrome';
	import { EditorRouter } from '$lib/features/editor';
	import { Omnibar } from '$lib/features/omnibar';
	import { StatusBar } from '$lib/features/status';
	import { editorStore } from '$lib/stores/editor';
	import { settingsStore } from '$lib/stores/settings';
	import { uiStore } from '$lib/stores/ui';
	import { shortcutRegistry } from '$lib/utils';
	import { noteStore } from '$lib/features/editor/note-store';
	import { codeStore } from '$lib/features/editor/code-store';
	import type { Section } from '$lib/components/chrome/PrimaryRail.svelte';
	import type { EditorState, SettingsState, UiState } from '$lib/types/store';

	const SIDEBAR_AUTO_HIDE_MS = 3000;

	// Subscribe to the global editor store to feed the EditorRouter.
	let editorState: EditorState = $state({
		activeFile: null,
		openFiles: [],
		isDirty: false,
		content: '',
		language: 'plaintext'
	});

	// Subscribe to the UI store to track omnibar visibility.
	let uiState: UiState = $state({
		sidebarOpen: true,
		sidebarSection: 'explorer',
		omnibarOpen: false,
		rightPanelOpen: false
	});

	let settingsState: SettingsState = $state({
		theme: 'dark',
		fontSize: 14,
		fontFamily: 'JetBrains Mono, Fira Code, monospace',
		sidebarWidth: 260,
		autoHideSidebar: true,
		editorWordWrap: true
	});

	let sidebarAutoHideTimer: ReturnType<typeof window.setTimeout> | null = null;
	let sidebarHovered = false;
	let cleanupEditorSubscription: (() => void) | null = null;
	let cleanupUiSubscription: (() => void) | null = null;
	let cleanupSettingsSubscription: (() => void) | null = null;

	function clearSidebarAutoHideTimer() {
		if (sidebarAutoHideTimer !== null) {
			window.clearTimeout(sidebarAutoHideTimer);
			sidebarAutoHideTimer = null;
		}
	}

	function scheduleSidebarAutoHide() {
		clearSidebarAutoHideTimer();
		if (!settingsState.autoHideSidebar || !uiState.sidebarOpen || sidebarHovered) return;

		sidebarAutoHideTimer = window.setTimeout(() => {
			uiStore.closeSidebar();
		}, SIDEBAR_AUTO_HIDE_MS);
	}

	function handleActivity() {
		scheduleSidebarAutoHide();
	}

	function handleLeftEdgeHover() {
		if (!settingsState.autoHideSidebar || uiState.sidebarOpen) return;
		uiStore.openSidebar();
	}

	function handleSidebarMouseEnter() {
		sidebarHovered = true;
		clearSidebarAutoHideTimer();
	}

	function handleSidebarMouseLeave() {
		sidebarHovered = false;
		scheduleSidebarAutoHide();
	}

	function handleSectionChange(section: Section | null) {
		if (section === null) {
			uiStore.closeSidebar();
			return;
		}
		uiStore.setSidebarSection(section);
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
		cleanupEditorSubscription = editorStore.subscribe((s) => {
			editorState = s;
		});

		cleanupUiSubscription = uiStore.subscribe((s) => {
			uiState = s;
			scheduleSidebarAutoHide();
		});

		cleanupSettingsSubscription = settingsStore.subscribe((s) => {
			settingsState = s;
			scheduleSidebarAutoHide();
		});

		shortcutRegistry.register('ctrl+p', () => uiStore.toggleOmnibar(), { global: true });
		shortcutRegistry.register('ctrl+s', saveCurrentFile, { global: true });
		shortcutRegistry.register('ctrl+b', () => uiStore.toggleSidebar(), { global: true });
		shortcutRegistry.register('ctrl+n', createNewNote, { global: true });
		shortcutRegistry.register('ctrl+.', () => noteStore.toggleViewMode(), { global: true });

		window.addEventListener('mousemove', handleActivity, { capture: true, passive: true });
		window.addEventListener('mousedown', handleActivity, { capture: true, passive: true });
		window.addEventListener('keydown', handleActivity, { capture: true });
		scheduleSidebarAutoHide();
	});

	onDestroy(() => {
		shortcutRegistry.unregister('ctrl+p');
		shortcutRegistry.unregister('ctrl+s');
		shortcutRegistry.unregister('ctrl+b');
		shortcutRegistry.unregister('ctrl+n');
		shortcutRegistry.unregister('ctrl+.');

		window.removeEventListener('mousemove', handleActivity, { capture: true });
		window.removeEventListener('mousedown', handleActivity, { capture: true });
		window.removeEventListener('keydown', handleActivity, { capture: true });
		clearSidebarAutoHideTimer();

		cleanupEditorSubscription?.();
		cleanupUiSubscription?.();
		cleanupSettingsSubscription?.();
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
			<PrimaryRail
				activeSection={uiState.sidebarOpen ? (uiState.sidebarSection as Section) : null}
				onSectionChange={handleSectionChange}
			/>
		{/snippet}

		{#snippet sidebar()}
			<Sidebar
				isOpen={uiState.sidebarOpen}
				activeSection={uiState.sidebarOpen ? (uiState.sidebarSection as Section) : null}
				onMouseEnter={handleSidebarMouseEnter}
				onMouseLeave={handleSidebarMouseLeave}
			/>
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

{#if settingsState.autoHideSidebar && !uiState.sidebarOpen}
	<div
		class="fixed inset-y-0 left-0 z-20 w-2"
		onmouseenter={handleLeftEdgeHover}
		aria-hidden="true"
	></div>
{/if}

<!-- Omnibar overlay, rendered outside the AppShell so it floats above everything -->
{#if uiState.omnibarOpen}
	<Omnibar onclose={handleOmnibarClose} onselect={handleOmnibarSelect} />
{/if}
