import { writable } from 'svelte/store';
import type { SettingsState } from '$lib/types/store';

function createSettingsStore() {
	const { subscribe, update, set } = writable<SettingsState>({
		theme: 'dark',
		fontSize: 14,
		fontFamily: 'JetBrains Mono, Fira Code, monospace',
		sidebarWidth: 260,
		editorWordWrap: true
	});

	return {
		subscribe,
		set,

		/** Update one or more settings fields. */
		updateSetting(partial: Partial<SettingsState>) {
			update((s) => ({ ...s, ...partial }));
		}
	};
}

export const settingsStore = createSettingsStore();
