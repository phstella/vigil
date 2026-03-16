/**
 * Workspace open lifecycle -- wires the openWorkspace IPC into the app flow.
 *
 * On app startup (or via an "Open Workspace" action), calls openWorkspace IPC
 * then populates the explorer tree via listDir. This replaces the previous
 * mock-data approach with real backend data.
 */

import { openWorkspace } from '$lib/ipc/files';
import { explorerStore } from '$lib/features/explorer/explorer-store.svelte';
import { filesStore } from '$lib/stores/files';

/**
 * Open a workspace by absolute path and initialize the explorer.
 *
 * Flow:
 * 1. Call openWorkspace IPC with the root path
 * 2. Set the workspace root in filesStore (for file-watcher compatibility)
 * 3. Load the explorer tree root via listDir (delegated to explorerStore)
 *
 * @param rootPath Absolute filesystem path to the workspace root
 * @returns The workspace canonical path, or throws on failure
 */
export async function openAndLoadWorkspace(rootPath: string): Promise<string> {
	// Reset explorer state for fresh workspace
	explorerStore.reset();

	// Call the backend to open/index the workspace
	const response = await openWorkspace(rootPath);

	// Set the workspace root in filesStore so file-watcher events are properly contextualized
	filesStore.setWorkspace(response.canonical_path);

	// Extract a display name from the canonical path
	const displayName =
		response.canonical_path.split('/').pop() ??
		response.canonical_path.split('\\').pop() ??
		'Workspace';

	// Populate the explorer tree from the real workspace
	await explorerStore.loadWorkspaceRoot(displayName, response.notes_count, response.files_count);

	return response.canonical_path;
}
