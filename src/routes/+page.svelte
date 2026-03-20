<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import AppShell from '$lib/components/layout/AppShell.svelte';
	import Sidebar from '$lib/components/layout/Sidebar.svelte';
	import TitleBar from '$lib/components/layout/TitleBar.svelte';
	import WorkspaceGrid from '$lib/components/layout/WorkspaceGrid.svelte';
	import PrimaryRail from '$lib/components/chrome/PrimaryRail.svelte';
	import EditorRouter from '$lib/features/editor/EditorRouter.svelte';
	import Omnibar from '$lib/features/omnibar/Omnibar.svelte';
	import StatusBar from '$lib/features/status/StatusBar.svelte';
	import { statusStore } from '$lib/features/status/status-store';
	import {
		initFileWatcher,
		teardownFileWatcher,
		openAndLoadWorkspace
	} from '$lib/features/workspace';
	import { editorStore } from '$lib/stores/editor';
	import { settingsStore } from '$lib/stores/settings';
	import { uiStore } from '$lib/stores/ui';
	import { shortcutRegistry } from '$lib/utils/shortcuts';
	import { noteStore } from '$lib/features/editor/note-store.svelte';
	import { codeStore, detectLanguage } from '$lib/features/editor/code-store.svelte';
	import { readFile, writeFile } from '$lib/ipc/files';
	import { isMarkdownFile } from '$lib/utils/file-routing';
	import type { Section } from '$lib/components/chrome/PrimaryRail.svelte';
	import type { EditorState, SettingsState, UiState } from '$lib/types/store';

	const SIDEBAR_AUTO_HIDE_MS = 3000;

	// Subscribe to the global editor store to feed the EditorRouter.
	let editorState: EditorState = $state({
		activeFile: null,
		openFiles: [],
		isDirty: false,
		content: '',
		language: 'plaintext',
		noteFile: null,
		noteContent: '',
		codeFile: null,
		codeContent: '',
		codeLanguage: 'plaintext',
		conflictFiles: new Set<string>()
	});

	// Subscribe to the UI store to track omnibar visibility and mode.
	let uiState: UiState = $state({
		sidebarOpen: true,
		sidebarSection: 'explorer',
		omnibarOpen: false,
		omnibarMode: 'file',
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
		uiStore.closeOmnibar();
	}

	async function handleOmnibarSelect(path: string, lineNumber?: number) {
		try {
			const response = await readFile(path);
			const language = isMarkdownFile(path) ? 'markdown' : detectLanguage(path);
			editorStore.openFileRouted(path, response.content, language);
			if (lineNumber !== undefined) {
				// Line number is available for content search results.
				// Editors will use this to jump to the matching line once go-to-line is wired.
				console.debug('[omnibar] open at line:', lineNumber);
			}
		} catch (err) {
			console.error('[omnibar] failed to open file:', path, err);
		}
	}

	/** Save the active file via IPC (notes use write_file; code is placeholder). */
	async function saveCurrentFile() {
		if (noteStore.isDirty) {
			const success = await noteStore.save();
			if (!success) {
				console.error('[shortcut] note save failed:', noteStore.lastError);
			}
			return;
		}
		if (codeStore.isDirty && codeStore.filePath) {
			try {
				await writeFile(codeStore.filePath, codeStore.content);
				codeStore.markClean();
				editorStore.setDirty(false);
			} catch (err: unknown) {
				const message = err instanceof Error ? err.message : String(err);
				console.error('[shortcut] code save failed:', message);
			}
			return;
		}
	}

	/** Placeholder new note action. */
	function createNewNote() {
		// TODO: Wire to create_note IPC command once available.
		console.log('[shortcut] create new note');
	}

	/**
	 * Flush live editor content from codeStore/noteStore back into editorStore
	 * so the tab cache captures unsaved edits before any tab switch or close.
	 */
	function flushLiveContent() {
		if (!editorState.activeFile) return;
		if (
			!isMarkdownFile(editorState.activeFile) &&
			codeStore.filePath === editorState.activeFile
		) {
			editorStore.updateContent(codeStore.content);
			if (codeStore.isDirty !== editorState.isDirty) {
				editorStore.setDirty(codeStore.isDirty);
			}
		}
		if (
			isMarkdownFile(editorState.activeFile) &&
			noteStore.filePath === editorState.activeFile
		) {
			editorStore.updateContent(noteStore.content);
			if (noteStore.isDirty !== editorState.isDirty) {
				editorStore.setDirty(noteStore.isDirty);
			}
		}
	}

	/** Handle tab activation from the tab bar. */
	function handleTabActivate(path: string) {
		flushLiveContent();
		editorStore.activateTab(path);
	}

	/** Handle tab close from the tab bar. */
	function handleTabClose(path: string) {
		flushLiveContent();
		editorStore.closeFile(path);
	}

	// Side-by-side is an explicit user mode. Default editing uses a single adaptive pane.
	let sideBySideMode = $derived(uiState.rightPanelOpen);

	// In single-pane mode, route the active file to its matching editor.
	let activePaneFile = $derived(editorState.activeFile);
	let activePaneContent = $derived.by(() => {
		const activePath = editorState.activeFile;
		if (!activePath) return '';
		if (isMarkdownFile(activePath)) {
			return editorState.noteFile === activePath
				? editorState.noteContent
				: editorState.content;
		}
		return editorState.codeFile === activePath ? editorState.codeContent : editorState.content;
	});

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
		shortcutRegistry.register('ctrl+shift+f', () => uiStore.openOmnibar('content'), {
			global: true
		});
		shortcutRegistry.register('ctrl+s', saveCurrentFile, { global: true });
		shortcutRegistry.register('ctrl+b', () => uiStore.toggleSidebar(), { global: true });
		shortcutRegistry.register('ctrl+\\', () => uiStore.toggleRightPanel(), { global: true });
		shortcutRegistry.register('ctrl+n', createNewNote, { global: true });
		shortcutRegistry.register('ctrl+.', () => noteStore.toggleViewMode(), { global: true });
		// Tab navigation shortcuts — flush live edits before switching
		shortcutRegistry.register(
			'ctrl+tab',
			() => {
				flushLiveContent();
				editorStore.nextTab();
			},
			{ global: true }
		);
		shortcutRegistry.register(
			'ctrl+shift+tab',
			() => {
				flushLiveContent();
				editorStore.prevTab();
			},
			{ global: true }
		);
		shortcutRegistry.register(
			'ctrl+w',
			() => {
				flushLiveContent();
				editorStore.closeActiveTab();
			},
			{ global: true }
		);

		window.addEventListener('mousemove', handleActivity, { capture: true, passive: true });
		window.addEventListener('mousedown', handleActivity, { capture: true, passive: true });
		window.addEventListener('keydown', handleActivity, { capture: true });
		scheduleSidebarAutoHide();

		// Wire file-watcher events from Rust backend to UI stores
		initFileWatcher().catch((err) => {
			console.error('[vigil] failed to initialize file watcher listeners:', err);
		});

		// Attempt to open a workspace on startup.
		// In production, the path comes from CLI args, recent workspaces, or a dialog.
		// The app gracefully shows an empty explorer if no workspace is opened.
		openAndLoadWorkspace('.')
			.then(() => {
				// Re-fetch status now that workspace is open and backend is ready
				return statusStore.initialize();
			})
			.catch((err) => {
				// Expected to fail in dev/browser mode; workspace will be opened via user action
				console.debug('[vigil] initial workspace open skipped or failed:', err);
			});
	});

	onDestroy(() => {
		shortcutRegistry.unregister('ctrl+p');
		shortcutRegistry.unregister('ctrl+shift+f');
		shortcutRegistry.unregister('ctrl+s');
		shortcutRegistry.unregister('ctrl+b');
		shortcutRegistry.unregister('ctrl+\\');
		shortcutRegistry.unregister('ctrl+n');
		shortcutRegistry.unregister('ctrl+.');
		shortcutRegistry.unregister('ctrl+tab');
		shortcutRegistry.unregister('ctrl+shift+tab');
		shortcutRegistry.unregister('ctrl+w');

		window.removeEventListener('mousemove', handleActivity, { capture: true });
		window.removeEventListener('mousedown', handleActivity, { capture: true });
		window.removeEventListener('keydown', handleActivity, { capture: true });
		clearSidebarAutoHideTimer();

		// Tear down file-watcher event subscriptions
		teardownFileWatcher();

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

	<WorkspaceGrid splitEnabled={sideBySideMode}>
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
				filePath={editorState.codeFile}
				content={editorState.codeContent}
				pane="code"
			/>
		{/snippet}

		{#if sideBySideMode}
			<!-- Split view: center pane is always the note editor surface. -->
			<EditorRouter
				filePath={editorState.noteFile}
				content={editorState.noteContent}
				pane="note"
				activeTab={editorState.activeFile}
				tabs={editorState.openFiles}
				onactivatetab={handleTabActivate}
				onclosetab={handleTabClose}
			/>
		{:else}
			<!-- Default mode: one pane that switches between note/code by active file type. -->
			<EditorRouter
				filePath={activePaneFile}
				content={activePaneContent}
				pane="auto"
				activeTab={editorState.activeFile}
				tabs={editorState.openFiles}
				onactivatetab={handleTabActivate}
				onclosetab={handleTabClose}
			/>
		{/if}
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
	<Omnibar
		onclose={handleOmnibarClose}
		onselect={handleOmnibarSelect}
		initialMode={uiState.omnibarMode}
	/>
{/if}
