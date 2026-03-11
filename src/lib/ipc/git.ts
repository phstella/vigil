/**
 * IPC wrappers for git commands.
 *
 * Commands: get_git_hunks, get_git_status
 */

import { invokeCommand } from './tauri';
import type { GitHunksResponse, GitStatusEntry } from '$lib/types/ipc';

/** Get line-level diff hunks for a file against git HEAD. */
export function getGitHunks(path: string): Promise<GitHunksResponse> {
	return invokeCommand<GitHunksResponse>('get_git_hunks', { path });
}

/** Get git status of all files in the workspace. */
export function getGitStatus(): Promise<GitStatusEntry[]> {
	return invokeCommand<GitStatusEntry[]>('get_git_status');
}
