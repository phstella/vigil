<script lang="ts">
	/**
	 * Omnibar -- Floating overlay for file and content search.
	 *
	 * Renders a centered modal near the top of the viewport with a text
	 * input, mode tabs (File / Content), and live search results from the
	 * backend. Keyboard navigation: ArrowUp/Down move selection, Enter
	 * opens selected, Escape closes, Tab switches mode.
	 * Clicking the backdrop also closes the overlay.
	 */

	import { omnibarStore } from './omnibar-store';
	import OmnibarItem from './OmnibarItem.svelte';
	import { perfTimer } from '$lib/utils/perf';
	import type { OmnibarMode } from '$lib/types/store';

	let {
		onclose,
		onselect,
		initialMode = 'file' as OmnibarMode
	}: {
		onclose: () => void;
		onselect?: (path: string, lineNumber?: number) => void;
		initialMode?: OmnibarMode;
	} = $props();

	let inputEl: HTMLInputElement | undefined = $state();

	/** Perf timer: measures time from omnibar open to first paint. */
	const openTimer = perfTimer('omnibar-open', 80);

	/** Auto-focus the input and trigger initial search when the component mounts. */
	$effect(() => {
		inputEl?.focus();
		omnibarStore.initialize(initialMode);
		// Measure first paint after mount
		queueMicrotask(() => {
			openTimer.stop();
		});
	});

	/** Reset the store when the component is destroyed. */
	$effect(() => {
		return () => {
			omnibarStore.reset();
		};
	});

	function handleKeydown(e: KeyboardEvent) {
		switch (e.key) {
			case 'ArrowDown':
				e.preventDefault();
				omnibarStore.selectNext();
				break;
			case 'ArrowUp':
				e.preventDefault();
				omnibarStore.selectPrev();
				break;
			case 'Enter': {
				e.preventDefault();
				const selected = omnibarStore.selectCurrent();
				if (selected) {
					const lineNumber = selected.type === 'content' ? selected.lineNumber : undefined;
					onselect?.(selected.path, lineNumber);
					onclose();
				}
				break;
			}
			case 'Escape':
				e.preventDefault();
				onclose();
				break;
			case 'Tab': {
				// Tab toggles between file and content mode.
				e.preventDefault();
				const nextMode: OmnibarMode = omnibarStore.mode === 'file' ? 'content' : 'file';
				omnibarStore.setMode(nextMode);
				break;
			}
		}
	}

	function handleInput(e: Event) {
		const target = e.target as HTMLInputElement;
		omnibarStore.setQuery(target.value);
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			onclose();
		}
	}

	function handleItemClick(path: string, lineNumber?: number) {
		onselect?.(path, lineNumber);
		onclose();
	}

	function switchMode(newMode: OmnibarMode) {
		omnibarStore.setMode(newMode);
		inputEl?.focus();
	}

	let placeholderText = $derived(
		omnibarStore.mode === 'content'
			? 'Search file contents...'
			: 'Type to search files...'
	);

	let emptyMessage = $derived(
		omnibarStore.mode === 'content'
			? 'No content matches found.'
			: 'No matching files found.'
	);
</script>

<div
	class="fixed inset-0 z-modal flex justify-center bg-black/50 pt-20"
	onkeydown={handleKeydown}
	onmousedown={handleBackdropClick}
	role="dialog"
	aria-modal="true"
	aria-label={omnibarStore.mode === 'content' ? 'Content search' : 'File search'}
	tabindex="-1"
>
	<div
		class="flex h-fit max-h-[400px] w-full max-w-[500px] flex-col overflow-hidden rounded-lg border border-surface-border bg-surface-raised shadow-2xl"
	>
		<!-- Mode tabs -->
		<div class="flex border-b border-surface-border">
			<button
				type="button"
				class="flex-1 px-3 py-1.5 text-xs font-medium transition-colors {omnibarStore.mode === 'file'
					? 'border-b-2 border-accent text-accent'
					: 'text-text-muted hover:text-text-secondary'}"
				onclick={() => switchMode('file')}
			>
				Files
				<span class="ml-1 text-[10px] text-text-muted">Ctrl+P</span>
			</button>
			<button
				type="button"
				class="flex-1 px-3 py-1.5 text-xs font-medium transition-colors {omnibarStore.mode === 'content'
					? 'border-b-2 border-accent text-accent'
					: 'text-text-muted hover:text-text-secondary'}"
				onclick={() => switchMode('content')}
			>
				Content
				<span class="ml-1 text-[10px] text-text-muted">Ctrl+Shift+F</span>
			</button>
		</div>

		<!-- Search input -->
		<div class="flex items-center border-b border-surface-border px-3">
			<svg
				class="mr-2 h-4 w-4 shrink-0 text-text-muted"
				viewBox="0 0 16 16"
				fill="none"
				aria-hidden="true"
			>
				<circle cx="7" cy="7" r="5" stroke="currentColor" stroke-width="1.5" />
				<path d="M11 11l3 3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
			</svg>
			<input
				bind:this={inputEl}
				type="text"
				class="h-10 flex-1 bg-transparent text-sm text-text-primary outline-none placeholder:text-text-muted"
				placeholder={placeholderText}
				value={omnibarStore.query}
				oninput={handleInput}
				aria-label={omnibarStore.mode === 'content' ? 'Search content' : 'Search files'}
				aria-autocomplete="list"
				aria-controls="omnibar-results"
				role="combobox"
				aria-expanded="true"
			/>
			{#if omnibarStore.isLoading}
				<span class="ml-2 text-xs text-text-muted" aria-live="polite">Searching...</span>
			{/if}
		</div>

		<!-- Results list -->
		<div id="omnibar-results" class="overflow-y-auto" role="listbox" aria-label="Search results">
			{#if omnibarStore.error}
				<div class="px-3 py-4 text-center text-sm text-red-400">{omnibarStore.error}</div>
			{:else if omnibarStore.results.length > 0}
				{#each omnibarStore.results as item, i (item.id)}
					<OmnibarItem
						{item}
						isSelected={i === omnibarStore.selectedIndex}
						onclick={() => {
							const lineNumber = item.type === 'content' ? item.lineNumber : undefined;
							handleItemClick(item.path, lineNumber);
						}}
					/>
				{/each}
			{:else if !omnibarStore.isLoading}
				<div class="px-3 py-4 text-center text-sm text-text-muted">{emptyMessage}</div>
			{/if}
		</div>
	</div>
</div>
