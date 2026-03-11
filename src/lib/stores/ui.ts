import { writable } from 'svelte/store';
import type { SidebarSection, UiState } from '$lib/types/store';

function createUiStore() {
	const { subscribe, update, set } = writable<UiState>({
		sidebarOpen: true,
		sidebarSection: 'explorer',
		omnibarOpen: false,
		rightPanelOpen: false
	});

	return {
		subscribe,
		set,

		/** Toggle sidebar visibility. */
		toggleSidebar() {
			update((s) => ({ ...s, sidebarOpen: !s.sidebarOpen }));
		},

		/** Switch the active sidebar panel. */
		setSidebarSection(section: SidebarSection) {
			update((s) => ({ ...s, sidebarSection: section, sidebarOpen: true }));
		},

		/** Toggle the omnibar overlay. */
		toggleOmnibar() {
			update((s) => ({ ...s, omnibarOpen: !s.omnibarOpen }));
		},

		/** Toggle the right (code) panel. */
		toggleRightPanel() {
			update((s) => ({ ...s, rightPanelOpen: !s.rightPanelOpen }));
		}
	};
}

export const uiStore = createUiStore();
