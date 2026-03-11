/**
 * IPC wrappers for workspace status commands.
 *
 * Commands: workspace_status
 */

import { invokeCommand } from './tauri';
import type { WorkspaceStatus } from '$lib/types/ipc';

/** Get current workspace status for the footer status bar. */
export function workspaceStatus(): Promise<WorkspaceStatus> {
	return invokeCommand<WorkspaceStatus>('workspace_status');
}
