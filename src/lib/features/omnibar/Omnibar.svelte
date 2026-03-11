<script lang="ts">
	/**
	 * Omnibar -- Floating overlay for fuzzy file search.
	 *
	 * Renders a centered modal near the top of the viewport with a text
	 * input and filtered results list. Keyboard navigation: ArrowUp/Down
	 * move selection, Enter opens selected, Escape closes. Clicking the
	 * backdrop also closes the overlay.
	 */

	import { omnibarStore } from './omnibar-store';
	import OmnibarItem from './OmnibarItem.svelte';

	let {
		onclose,
		onselect
	}: {
		onclose: () => void;
		onselect?: (path: string) => void;
	} = $props();

	let inputEl: HTMLInputElement | undefined = $state();

	/** Auto-focus the input whenever the component mounts. */
	$effect(() => {
		inputEl?.focus();
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
					onselect?.(selected.path);
					onclose();
				}
				break;
			}
			case 'Escape':
				e.preventDefault();
				onclose();
				break;
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

	function handleItemClick(path: string) {
		onselect?.(path);
		onclose();
	}
</script>

<div
	class="fixed inset-0 z-modal flex justify-center bg-black/50 pt-20"
	onkeydown={handleKeydown}
	onmousedown={handleBackdropClick}
	role="dialog"
	aria-modal="true"
	aria-label="File search"
	tabindex="-1"
>
	<div
		class="flex h-fit max-h-[400px] w-full max-w-[500px] flex-col overflow-hidden rounded-lg border border-surface-border bg-surface-raised shadow-2xl"
	>
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
				placeholder="Type to search files..."
				value={omnibarStore.query}
				oninput={handleInput}
				aria-label="Search files"
				aria-autocomplete="list"
				aria-controls="omnibar-results"
				role="combobox"
				aria-expanded="true"
			/>
		</div>

		<!-- Results list -->
		<div id="omnibar-results" class="overflow-y-auto" role="listbox" aria-label="Search results">
			{#each omnibarStore.results as item, i (item.id)}
				<OmnibarItem
					{item}
					isSelected={i === omnibarStore.selectedIndex}
					onclick={() => handleItemClick(item.path)}
				/>
			{:else}
				<div class="px-3 py-4 text-center text-sm text-text-muted">No matching files found.</div>
			{/each}
		</div>
	</div>
</div>
