<script lang="ts">
	/**
	 * NoteEditor -- Markdown editing surface with IPC-backed open/save/autosave,
	 * live preview toggle, `[[` wikilink autocomplete, and backlinks panel.
	 *
	 * Displays a file path header/tab area with dirty/saving indicators and a
	 * view-mode toggle button. In 'edit' mode: shows a monospace textarea for
	 * raw markdown editing with `[[` autocomplete. In 'preview' mode: shows
	 * rendered markdown via InlinePreview. Raw text is always preserved as the
	 * source of truth. Toggle via Ctrl+. or the header button.
	 *
	 * The BacklinksPanel is displayed below the editor and updates when the
	 * active file changes or content is saved.
	 */

	import { onDestroy } from 'svelte';
	import { noteStore } from './note-store';
	import InlinePreview from '$lib/features/preview/InlinePreview.svelte';
	import BacklinksPanel from '$lib/features/links/BacklinksPanel.svelte';
	import WikilinkAutocomplete from '$lib/features/links/WikilinkAutocomplete.svelte';
	import { linksStore } from '$lib/features/links/links-store';
	import { detectWikilinkTrigger, insertWikilink } from '$lib/utils/markdown';
	import { editorStore } from '$lib/stores/editor';
	import { readFile } from '$lib/ipc/files';
	import { isMarkdownFile } from '$lib/utils/file-routing';
	import { detectLanguage } from './code-store';

	let {
		filePath,
		content
	}: {
		filePath: string;
		content: string;
	} = $props();

	// Track which file path we last loaded so we can detect prop changes.
	let loadedPath: string | null = null;

	// Wikilink autocomplete state
	let autocompleteVisible = $state(false);
	let autocompleteQuery = $state('');
	let autocompleteX = $state(0);
	let autocompleteY = $state(0);
	let textareaRef: HTMLTextAreaElement | null = $state(null);
	let autocompleteRef: WikilinkAutocomplete | null = $state(null);

	// When filePath or content props change, load into the note store.
	$effect(() => {
		if (filePath && filePath !== loadedPath) {
			loadedPath = filePath;
			// Open via IPC to get etag for optimistic concurrency.
			// Fall back to direct load if IPC is unavailable (dev/test).
			const targetPath = filePath;
			const fallbackContent = content;
			noteStore.open(targetPath).then((success) => {
				if (!success) {
					noteStore.load(targetPath, fallbackContent);
				}
			});
			// Update backlinks for the new file
			void linksStore.setActivePath(filePath);
		}
	});

	// When the note saves successfully (dirty becomes false after being true),
	// schedule a backlinks refresh since the saved content may contain new/removed links.
	let wasDirty = false;
	$effect(() => {
		const dirty = noteStore.isDirty;
		const saving = noteStore.isSaving;
		if (wasDirty && !dirty && !saving) {
			// Just finished saving - refresh backlinks
			linksStore.scheduleRefresh();
		}
		wasDirty = dirty;
	});

	function handleInput(e: Event) {
		const target = e.target as HTMLTextAreaElement;
		noteStore.updateContent(target.value);
		checkAutocomplete(target);
	}

	function checkAutocomplete(textarea: HTMLTextAreaElement) {
		const cursorPos = textarea.selectionStart;
		const trigger = detectWikilinkTrigger(textarea.value, cursorPos);

		if (trigger !== null) {
			autocompleteQuery = trigger;
			autocompleteVisible = true;
			// Position the dropdown near the cursor
			updateAutocompletePosition(textarea, cursorPos);
		} else {
			autocompleteVisible = false;
			autocompleteQuery = '';
		}
	}

	function updateAutocompletePosition(textarea: HTMLTextAreaElement, cursorPos: number) {
		// Approximate position based on character offset
		// Use a mirror element technique for accurate positioning
		const text = textarea.value.substring(0, cursorPos);
		const lines = text.split('\n');
		const lineNum = lines.length - 1;
		const lineHeight = parseFloat(getComputedStyle(textarea).lineHeight) || 20;
		const charWidth = 8; // approximate for monospace

		// Get textarea's parent position for relative placement
		const parentRect = textarea.parentElement?.getBoundingClientRect();
		if (!parentRect) return;

		const relX = (lines[lineNum]?.length ?? 0) * charWidth;
		const relY = (lineNum + 1) * lineHeight;

		// Clamp X to not overflow
		autocompleteX = Math.min(relX, parentRect.width - 270);
		autocompleteY = Math.min(relY + 4, parentRect.height - 200);
	}

	function handleAutocompleteSelect(noteName: string) {
		if (!textareaRef) return;
		const cursorPos = textareaRef.selectionStart;
		const result = insertWikilink(textareaRef.value, cursorPos, noteName);
		noteStore.updateContent(result.text);

		// Update textarea and move cursor
		textareaRef.value = result.text;
		textareaRef.selectionStart = result.cursor;
		textareaRef.selectionEnd = result.cursor;
		textareaRef.focus();

		autocompleteVisible = false;
		autocompleteQuery = '';
	}

	function handleAutocompleteClose() {
		autocompleteVisible = false;
		autocompleteQuery = '';
	}

	function handleKeydown(e: KeyboardEvent) {
		// Let autocomplete handle keys first when visible
		if (autocompleteVisible && autocompleteRef) {
			const handled = autocompleteRef.handleKeydown(e);
			if (handled) return;
		}
	}

	function handleToggle() {
		noteStore.toggleViewMode();
	}

	async function handleBacklinkNavigate(path: string) {
		try {
			const response = await readFile(path);
			const language = isMarkdownFile(path) ? 'markdown' : detectLanguage(path);
			editorStore.openFileRouted(path, response.content, language);
		} catch (err) {
			console.error('[backlinks] failed to open file:', path, err);
		}
	}

	let isPreview = $derived(noteStore.viewMode === 'preview');

	// Clean up on destroy.
	onDestroy(() => {
		noteStore.cancelAutosave();
		linksStore.cancelRefresh();
	});
</script>

<div class="flex h-full flex-col bg-surface-base">
	<!-- Tab / header bar -->
	<header
		class="flex h-9 shrink-0 items-center gap-2 border-b border-surface-border bg-surface-raised px-3"
	>
		<svg
			class="h-3.5 w-3.5 shrink-0 text-text-muted"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="1.5"
		>
			<path d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
		</svg>
		<span class="truncate text-xs font-medium text-text-secondary" title={filePath}>
			{filePath}
		</span>

		<!-- View mode toggle button -->
		<button
			class="ml-auto flex items-center gap-1 rounded px-1.5 py-0.5 text-[10px] font-medium uppercase transition-colors hover:bg-surface-overlay"
			class:text-accent={isPreview}
			class:text-text-muted={!isPreview}
			onclick={handleToggle}
			title={isPreview ? 'Switch to edit mode (Ctrl+.)' : 'Switch to preview mode (Ctrl+.)'}
			aria-label={isPreview ? 'Switch to edit mode' : 'Switch to preview mode'}
		>
			{#if isPreview}
				<!-- Pencil icon for "switch to edit" -->
				<svg class="h-3 w-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7" />
					<path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z" />
				</svg>
				Edit
			{:else}
				<!-- Eye icon for "switch to preview" -->
				<svg class="h-3 w-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
					<circle cx="12" cy="12" r="3" />
				</svg>
				Preview
			{/if}
		</button>

		{#if noteStore.isSaving}
			<span
				class="text-[10px] font-medium text-text-muted"
				title="Saving..."
			>saving</span>
		{:else if noteStore.isDirty}
			<span
				class="h-2 w-2 shrink-0 rounded-full bg-accent"
				title="Unsaved changes"
			></span>
		{/if}
		{#if noteStore.lastError}
			<span
				class="ml-1 truncate text-[10px] text-red-400"
				title={noteStore.lastError}
			>{noteStore.lastError}</span>
		{/if}
	</header>

	<!-- Content area: edit or preview based on viewMode -->
	{#if isPreview}
		<div class="flex-1 overflow-auto">
			<InlinePreview content={noteStore.content} />
		</div>
	{:else}
		<div class="relative flex-1 overflow-auto p-4">
			<textarea
				bind:this={textareaRef}
				class="h-full w-full resize-none border-none bg-transparent font-mono text-sm leading-relaxed text-text-primary outline-none placeholder:text-text-muted"
				placeholder="Start writing..."
				value={noteStore.content}
				oninput={handleInput}
				onkeydown={handleKeydown}
				spellcheck="true"
				aria-label="Markdown editor"
			></textarea>

			<!-- Wikilink autocomplete dropdown -->
			<WikilinkAutocomplete
				bind:this={autocompleteRef}
				query={autocompleteQuery}
				visible={autocompleteVisible}
				anchorX={autocompleteX}
				anchorY={autocompleteY}
				onSelect={handleAutocompleteSelect}
				onClose={handleAutocompleteClose}
			/>
		</div>
	{/if}

	<!-- Backlinks panel at the bottom of the editor -->
	<BacklinksPanel onNavigate={handleBacklinkNavigate} />
</div>
