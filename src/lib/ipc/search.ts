/**
 * IPC wrappers for search commands.
 *
 * Commands: fuzzy_find, search_content, get_all_tags, get_files_by_tag
 */

import { invokeCommand } from './tauri';
import type { FuzzyFindResponse, SearchContentResponse, Tag } from '$lib/types/ipc';

/** Fuzzy-search workspace file names (Ctrl+P omnibar). */
export function fuzzyFind(query: string, limit?: number): Promise<FuzzyFindResponse> {
	return invokeCommand<FuzzyFindResponse>('fuzzy_find', { query, limit: limit ?? null });
}

/** Full-text content search with snippet extraction. */
export function searchContent(query: string, limit?: number): Promise<SearchContentResponse> {
	return invokeCommand<SearchContentResponse>('search_content', { query, limit: limit ?? null });
}

/** Get all tags across the workspace, sorted by usage count descending. */
export function getAllTags(): Promise<Tag[]> {
	return invokeCommand<Tag[]>('get_all_tags');
}

/** Get workspace-relative file paths that contain a given tag. */
export function getFilesByTag(tag: string): Promise<string[]> {
	return invokeCommand<string[]>('get_files_by_tag', { tag });
}
