/**
 * Omnibar local UI state store.
 *
 * Manages the search query, filtered result list, and keyboard
 * selection index for the floating omnibar overlay. Uses mock file
 * entries for demo purposes -- backend fuzzy_find will replace these
 * once the IPC layer is wired in a later ticket.
 */

export interface OmnibarResult {
	/** Unique identifier for the result item. */
	id: string;
	/** Display name (file name). */
	name: string;
	/** Workspace-relative path. */
	path: string;
	/** File extension without the leading dot, or null. */
	ext: string | null;
}

/** Static mock results used for demo/testing purposes. */
const MOCK_FILES: OmnibarResult[] = [
	{ id: '1', name: 'README.md', path: 'README.md', ext: 'md' },
	{ id: '2', name: 'architecture.md', path: 'docs/architecture.md', ext: 'md' },
	{ id: '3', name: 'daily-log.md', path: 'notes/daily-log.md', ext: 'md' },
	{ id: '4', name: 'todo.md', path: 'notes/todo.md', ext: 'md' },
	{ id: '5', name: 'config.ts', path: 'src/config.ts', ext: 'ts' },
	{ id: '6', name: 'utils.ts', path: 'src/lib/utils.ts', ext: 'ts' },
	{ id: '7', name: 'app.css', path: 'src/app.css', ext: 'css' },
	{ id: '8', name: 'meeting-notes.md', path: 'notes/meeting-notes.md', ext: 'md' },
	{ id: '9', name: 'index.ts', path: 'src/lib/index.ts', ext: 'ts' },
	{ id: '10', name: 'design-spec.md', path: 'docs/design-spec.md', ext: 'md' }
];

function createOmnibarStore() {
	let query = $state('');
	let selectedIndex = $state(0);

	const filtered = $derived.by(() => {
		if (query.trim() === '') return MOCK_FILES;
		const lower = query.toLowerCase();
		return MOCK_FILES.filter(
			(f) => f.name.toLowerCase().includes(lower) || f.path.toLowerCase().includes(lower)
		);
	});

	return {
		get query() {
			return query;
		},
		get selectedIndex() {
			return selectedIndex;
		},
		get results(): OmnibarResult[] {
			return filtered;
		},

		/** Update the search query and reset selection to the first item. */
		setQuery(value: string) {
			query = value;
			selectedIndex = 0;
		},

		/** Move selection to the next result, wrapping at the end. */
		selectNext() {
			if (filtered.length === 0) return;
			selectedIndex = (selectedIndex + 1) % filtered.length;
		},

		/** Move selection to the previous result, wrapping at the start. */
		selectPrev() {
			if (filtered.length === 0) return;
			selectedIndex = (selectedIndex - 1 + filtered.length) % filtered.length;
		},

		/** Return the currently selected result, or null if the list is empty. */
		selectCurrent(): OmnibarResult | null {
			if (filtered.length === 0) return null;
			return filtered[selectedIndex] ?? null;
		},

		/** Reset the store to its initial state. */
		reset() {
			query = '';
			selectedIndex = 0;
		}
	};
}

export const omnibarStore = createOmnibarStore();
