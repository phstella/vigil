/**
 * Keyboard shortcut registry.
 *
 * Provides a centralized system for registering global keyboard shortcuts.
 * Shortcuts are matched against keydown events using a normalized combo
 * string format (e.g. "ctrl+p", "ctrl+shift+s"). The registry handles
 * preventDefault, case-insensitive matching, and skipping input-focused
 * events for shortcuts that should not fire during text editing.
 */

export interface ShortcutEntry {
	/** Normalized combo string, e.g. "ctrl+shift+p". */
	combo: string;
	/** Handler invoked when the shortcut fires. */
	handler: () => void;
	/** When true, the shortcut fires even when an input or textarea has focus. */
	global: boolean;
}

/** Tags whose focused elements suppress non-global shortcuts. */
const INPUT_TAGS = new Set(['INPUT', 'TEXTAREA', 'SELECT']);

/**
 * Normalize a combo string to a canonical, comparable form.
 * Sorts modifier keys (alt, ctrl, meta, shift) before the base key.
 */
function normalizeCombo(combo: string): string {
	const parts = combo
		.toLowerCase()
		.split('+')
		.map((p) => p.trim());
	const modifiers: string[] = [];
	let key = '';
	for (const part of parts) {
		if (['ctrl', 'alt', 'shift', 'meta'].includes(part)) {
			modifiers.push(part);
		} else {
			key = part;
		}
	}
	modifiers.sort();
	return [...modifiers, key].join('+');
}

/** Build a normalized combo string from a KeyboardEvent. */
function comboFromEvent(e: KeyboardEvent): string {
	const modifiers: string[] = [];
	if (e.altKey) modifiers.push('alt');
	if (e.ctrlKey || e.metaKey) modifiers.push('ctrl');
	if (e.shiftKey) modifiers.push('shift');
	modifiers.sort();
	const key = e.key.toLowerCase();
	return [...modifiers, key].join('+');
}

/** Check whether the active element is a text input or editable region. */
function isInputFocused(): boolean {
	const el = document.activeElement;
	if (!el) return false;
	if (INPUT_TAGS.has(el.tagName)) return true;
	if ((el as HTMLElement).isContentEditable) return true;
	return false;
}

/**
 * Create a ShortcutRegistry that attaches a single global keydown listener
 * and dispatches to registered handlers.
 */
function createShortcutRegistry() {
	const entries = new Map<string, ShortcutEntry>();
	let listening = false;

	function handleKeydown(e: KeyboardEvent) {
		const combo = comboFromEvent(e);
		const entry = entries.get(combo);
		if (!entry) return;

		// Skip non-global shortcuts when focus is in an input field.
		if (!entry.global && isInputFocused()) return;

		e.preventDefault();
		e.stopPropagation();
		entry.handler();
	}

	function ensureListener() {
		if (listening) return;
		window.addEventListener('keydown', handleKeydown, { capture: true });
		listening = true;
	}

	function removeListener() {
		if (!listening) return;
		window.removeEventListener('keydown', handleKeydown, { capture: true });
		listening = false;
	}

	return {
		/**
		 * Register a keyboard shortcut.
		 *
		 * @param combo  Key combination string, e.g. "ctrl+p".
		 * @param handler  Callback to invoke.
		 * @param options.global  When true, fires even inside input fields.
		 */
		register(combo: string, handler: () => void, options?: { global?: boolean }) {
			const normalized = normalizeCombo(combo);
			entries.set(normalized, {
				combo: normalized,
				handler,
				global: options?.global ?? false
			});
			ensureListener();
		},

		/** Unregister a previously registered shortcut. */
		unregister(combo: string) {
			const normalized = normalizeCombo(combo);
			entries.delete(normalized);
			if (entries.size === 0) {
				removeListener();
			}
		},

		/** Remove all shortcuts and detach the listener. */
		destroy() {
			entries.clear();
			removeListener();
		}
	};
}

export type ShortcutRegistry = ReturnType<typeof createShortcutRegistry>;

/** Singleton shortcut registry for the application. */
export const shortcutRegistry = createShortcutRegistry();
