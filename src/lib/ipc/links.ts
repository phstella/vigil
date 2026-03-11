/**
 * IPC wrappers for link graph commands.
 *
 * Commands: get_backlinks
 */

import { invokeCommand } from './tauri';
import type { BacklinksResponse } from '$lib/types/ipc';

/** Get notes that link to the specified file. */
export function getBacklinks(path: string): Promise<BacklinksResponse> {
	return invokeCommand<BacklinksResponse>('get_backlinks', { path });
}
