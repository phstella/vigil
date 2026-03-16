/**
 * IPC wrappers for workspace and file CRUD commands.
 *
 * Commands: open_workspace, list_dir, read_file, write_file,
 *           create_note, rename_file, delete_file
 */

import { invokeCommand } from './tauri';
import type {
	OpenWorkspaceResponse,
	ListDirResponse,
	ReadFileResponse,
	WriteFileResponse,
	CreateNoteResponse,
	RenameFileResponse,
	DeleteFileResponse
} from '$lib/types/ipc';

/** Open (or switch to) a workspace directory. */
export function openWorkspace(rootPath: string): Promise<OpenWorkspaceResponse> {
	// Tauri command arg names are camelCase on the JS side.
	return invokeCommand<OpenWorkspaceResponse>('open_workspace', { rootPath });
}

/** List entries in a workspace directory. Pass `""` for the root. */
export function listDir(path: string): Promise<ListDirResponse> {
	return invokeCommand<ListDirResponse>('list_dir', { path });
}

/** Read a text file's content and metadata. */
export function readFile(path: string): Promise<ReadFileResponse> {
	return invokeCommand<ReadFileResponse>('read_file', { path });
}

/** Write content to a file, optionally with optimistic concurrency check. */
export function writeFile(
	path: string,
	content: string,
	etag?: string | null
): Promise<WriteFileResponse> {
	return invokeCommand<WriteFileResponse>('write_file', {
		request: { path, content, etag: etag ?? null }
	});
}

/** Create a new markdown note file. */
export function createNote(path: string): Promise<CreateNoteResponse> {
	return invokeCommand<CreateNoteResponse>('create_note', { request: { path } });
}

/** Rename or move a file within the workspace. */
export function renameFile(oldPath: string, newPath: string): Promise<RenameFileResponse> {
	return invokeCommand<RenameFileResponse>('rename_file', {
		oldPath,
		newPath
	});
}

/** Delete a file or empty directory within the workspace. */
export function deleteFile(path: string): Promise<DeleteFileResponse> {
	return invokeCommand<DeleteFileResponse>('delete_file', { path });
}
