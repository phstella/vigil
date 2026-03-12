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

		/** Open sidebar without changing section. */
		openSidebar() {
			update((s) => ({ ...s, sidebarOpen: true }));
		},

		/** Close sidebar without clearing section. */
		closeSidebar() {
			update((s) => ({ ...s, sidebarOpen: false }));
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
		},

		/** Open the right (code) panel. */
		openRightPanel() {
			update((s) => ({ ...s, rightPanelOpen: true }));
		},

		/** Close the right (code) panel. */
		closeRightPanel() {
			update((s) => ({ ...s, rightPanelOpen: false }));
		}
	};
}

export const uiStore = createUiStore();
