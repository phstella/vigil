<script lang="ts">
	/**
	 * SplitPane -- Resizable split container for the editor workspace.
	 *
	 * Supports horizontal (side-by-side) and vertical (top/bottom) orientations.
	 * Uses pointer events for smooth drag interaction and CSS flex with
	 * percentage-based sizing to avoid layout thrashing.
	 *
	 * Panels are provided via Svelte 5 snippets: `first` and `second`.
	 */

	import type { Snippet } from 'svelte';

	const MIN_PANEL_PCT = 20;
	const MAX_PANEL_PCT = 80;
	const DIVIDER_PX = 4;

	let {
		direction = 'horizontal',
		initialSplit = 50,
		first,
		second
	}: {
		direction?: 'horizontal' | 'vertical';
		initialSplit?: number;
		first: Snippet;
		second: Snippet;
	} = $props();

	// Writable derived: tracks initialSplit prop, overridable by drag interaction.
	// Resets to clamped initialSplit when the prop changes; drag writes override until then.
	let splitPct = $derived(clamp(initialSplit));
	let dragging = $state(false);
	let containerEl: HTMLDivElement | undefined = $state();

	function clamp(value: number): number {
		return Math.min(MAX_PANEL_PCT, Math.max(MIN_PANEL_PCT, value));
	}

	function onPointerDown(e: PointerEvent) {
		e.preventDefault();
		dragging = true;
		(e.target as HTMLElement).setPointerCapture(e.pointerId);
	}

	function onPointerMove(e: PointerEvent) {
		if (!dragging || !containerEl) return;

		const rect = containerEl.getBoundingClientRect();
		let pct: number;

		if (direction === 'horizontal') {
			pct = ((e.clientX - rect.left) / rect.width) * 100;
		} else {
			pct = ((e.clientY - rect.top) / rect.height) * 100;
		}

		splitPct = clamp(pct);
	}

	function onPointerUp() {
		dragging = false;
	}

	function onKeyDown(e: KeyboardEvent) {
		const step = e.shiftKey ? 5 : 1;
		const grow = direction === 'horizontal' ? 'ArrowRight' : 'ArrowDown';
		const shrink = direction === 'horizontal' ? 'ArrowLeft' : 'ArrowUp';

		if (e.key === grow) {
			e.preventDefault();
			splitPct = clamp(splitPct + step);
		} else if (e.key === shrink) {
			e.preventDefault();
			splitPct = clamp(splitPct - step);
		}
	}
</script>

<div
	bind:this={containerEl}
	class="flex min-h-0 min-w-0 flex-1"
	class:flex-row={direction === 'horizontal'}
	class:flex-col={direction === 'vertical'}
	role="group"
	aria-label="Split pane"
>
	<!-- First panel -->
	<div
		class="min-h-0 min-w-0 overflow-auto"
		style="{direction === 'horizontal' ? 'width' : 'height'}: calc({splitPct}% - {DIVIDER_PX /
			2}px)"
	>
		{@render first()}
	</div>

	<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
	<div
		class="shrink-0 transition-colors duration-100"
		class:w-[4px]={direction === 'horizontal'}
		class:h-[4px]={direction === 'vertical'}
		class:cursor-col-resize={direction === 'horizontal'}
		class:cursor-row-resize={direction === 'vertical'}
		class:bg-accent={dragging}
		class:bg-surface-border={!dragging}
		class:hover:bg-accent-muted={!dragging}
		role="separator"
		aria-orientation={direction === 'horizontal' ? 'vertical' : 'horizontal'}
		aria-valuenow={Math.round(splitPct)}
		aria-valuemin={MIN_PANEL_PCT}
		aria-valuemax={MAX_PANEL_PCT}
		tabindex="0"
		onpointerdown={onPointerDown}
		onpointermove={onPointerMove}
		onpointerup={onPointerUp}
		onpointercancel={onPointerUp}
		onkeydown={onKeyDown}
	></div>

	<!-- Second panel -->
	<div
		class="min-h-0 min-w-0 overflow-auto"
		style="{direction === 'horizontal' ? 'width' : 'height'}: calc({100 -
			splitPct}% - {DIVIDER_PX / 2}px)"
	>
		{@render second()}
	</div>
</div>
