/**
 * IPC wrappers for link graph commands.
 *
 * Commands: get_backlinks, get_note_graph
 */

import { invokeCommand } from './tauri';
import type { BacklinksResponse, NoteGraphResponse } from '$lib/types/ipc';

/** Get notes that link to the specified file. */
export function getBacklinks(path: string): Promise<BacklinksResponse> {
	return invokeCommand<BacklinksResponse>('get_backlinks', { path });
}

/** Get the full note link graph for visualization. */
export function getNoteGraph(): Promise<NoteGraphResponse> {
	return invokeCommand<NoteGraphResponse>('get_note_graph');
}
