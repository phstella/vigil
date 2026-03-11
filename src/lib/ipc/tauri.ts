/**
 * Thin typed wrapper around Tauri v2 invoke/listen with error handling.
 *
 * All IPC calls flow through this module so the rest of the frontend never
 * imports from `@tauri-apps/api` directly.
 */

import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { listen as tauriListen } from '@tauri-apps/api/event';
import type { UnlistenFn } from '@tauri-apps/api/event';
import type { VigilError } from '$lib/types/ipc';

/**
 * Type guard that checks whether an unknown rejection value looks like a
 * serialized `VigilError` envelope from the Rust backend.
 */
export function isVigilError(value: unknown): value is VigilError {
	if (typeof value !== 'object' || value === null) return false;
	const obj = value as Record<string, unknown>;
	return typeof obj.code === 'string' && typeof obj.message === 'string';
}

/**
 * Invoke a Tauri command with typed arguments and response.
 *
 * On success, returns the deserialized response payload `T`.
 * On failure, throws a `VigilError` envelope if the backend returned one,
 * or re-throws the original error otherwise.
 */
export async function invokeCommand<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
	try {
		return await tauriInvoke<T>(cmd, args);
	} catch (err: unknown) {
		if (isVigilError(err)) {
			throw err;
		}
		// Wrap unexpected errors so callers always see a VigilError shape.
		throw {
			code: 'INTERNAL_ERROR',
			message: typeof err === 'string' ? err : 'Unknown IPC error'
		} satisfies VigilError;
	}
}

/**
 * Subscribe to a Tauri event channel with a typed payload.
 *
 * Returns an unlisten function that the caller must invoke to unsubscribe.
 */
export async function listenEvent<T>(
	event: string,
	handler: (payload: T) => void
): Promise<UnlistenFn> {
	return tauriListen<T>(event, (ev) => handler(ev.payload));
}
