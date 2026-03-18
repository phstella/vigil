<script lang="ts">
	/**
	 * InlinePreview -- Rendered markdown preview surface.
	 *
	 * Takes raw markdown content and renders it as styled HTML.
	 * Used as the WYSIWYG/preview half of the Ctrl+. toggle in NoteEditor.
	 * The raw text is never modified by this component; it is read-only.
	 *
	 * Fenced ```mermaid blocks are detected in the rendered HTML (as .md-mermaid
	 * marker divs) and replaced with live MermaidBlock component instances
	 * that render diagrams via the mermaid library.
	 */

	import { markdownToHtml } from '$lib/utils/markdown';
	import { mount, unmount } from 'svelte';
	import MermaidBlock from './MermaidBlock.svelte';

	let { content }: { content: string } = $props();

	let renderedHtml = $derived(markdownToHtml(content));

	let previewRef: HTMLDivElement | null = $state(null);

	// Track mounted MermaidBlock instances for cleanup.
	let mountedBlocks: ReturnType<typeof mount>[] = [];

	function destroyMountedBlocks() {
		for (const block of mountedBlocks) {
			try {
				unmount(block);
			} catch {
				// Ignore unmount errors for already-removed DOM nodes.
			}
		}
		mountedBlocks = [];
	}

	// After each HTML render, scan for mermaid marker divs and mount components.
	$effect(() => {
		// Subscribe to renderedHtml so this re-runs on content changes.
		void renderedHtml;

		// Use a microtask to ensure the DOM has been updated by {@html}.
		const frame = requestAnimationFrame(() => {
			if (!previewRef) return;

			destroyMountedBlocks();

			const mermaidDivs = previewRef.querySelectorAll<HTMLDivElement>('.md-mermaid');
			for (const div of mermaidDivs) {
				const encoded = div.getAttribute('data-mermaid-source');
				if (!encoded) continue;

				try {
					const source = decodeURIComponent(escape(atob(encoded)));

					// Clear the marker div and mount a MermaidBlock into it.
					div.textContent = '';
					const block = mount(MermaidBlock, {
						target: div,
						props: { source }
					});
					mountedBlocks.push(block);
				} catch {
					// If base64 decoding fails, show raw encoded content as fallback.
					div.textContent = `[Mermaid decode error]`;
				}
			}
		});

		return () => {
			cancelAnimationFrame(frame);
			destroyMountedBlocks();
		};
	});
</script>

<div
	bind:this={previewRef}
	class="md-preview h-full overflow-auto p-6 font-sans text-sm leading-relaxed text-text-primary"
	role="document"
	aria-label="Markdown preview"
>
	<!-- eslint-disable-next-line svelte/no-at-html-tags -->
	{@html renderedHtml}
</div>

<style>
	/* Scoped styles for rendered markdown elements */
	.md-preview :global(h1) {
		font-size: 1.75rem;
		font-weight: 700;
		margin: 1.5rem 0 0.75rem;
		color: var(--color-text-primary);
		border-bottom: 1px solid var(--color-surface-border);
		padding-bottom: 0.5rem;
	}

	.md-preview :global(h2) {
		font-size: 1.375rem;
		font-weight: 600;
		margin: 1.25rem 0 0.5rem;
		color: var(--color-text-primary);
	}

	.md-preview :global(h3) {
		font-size: 1.125rem;
		font-weight: 600;
		margin: 1rem 0 0.5rem;
		color: var(--color-text-primary);
	}

	.md-preview :global(h4),
	.md-preview :global(h5),
	.md-preview :global(h6) {
		font-size: 1rem;
		font-weight: 600;
		margin: 0.75rem 0 0.375rem;
		color: var(--color-text-secondary);
	}

	.md-preview :global(p) {
		margin: 0.5rem 0;
		line-height: 1.7;
	}

	.md-preview :global(strong) {
		font-weight: 700;
		color: var(--color-text-primary);
	}

	.md-preview :global(em) {
		font-style: italic;
	}

	.md-preview :global(del) {
		text-decoration: line-through;
		opacity: 0.6;
	}

	.md-preview :global(a.md-link),
	.md-preview :global(a.md-wikilink) {
		color: var(--color-accent);
		text-decoration: none;
		border-bottom: 1px solid transparent;
		transition: border-color 0.15s ease;
	}

	.md-preview :global(a.md-link:hover),
	.md-preview :global(a.md-wikilink:hover) {
		border-bottom-color: var(--color-accent);
	}

	.md-preview :global(a.md-wikilink::before) {
		content: '[[';
		opacity: 0.4;
		font-size: 0.85em;
	}

	.md-preview :global(a.md-wikilink::after) {
		content: ']]';
		opacity: 0.4;
		font-size: 0.85em;
	}

	.md-preview :global(code.md-inline-code) {
		font-family: var(--font-mono);
		font-size: 0.85em;
		background: var(--color-surface-overlay);
		border-radius: 3px;
		padding: 0.15em 0.35em;
		color: var(--color-accent-strong);
	}

	.md-preview :global(pre.md-code-block) {
		font-family: var(--font-mono);
		font-size: 0.85rem;
		background: var(--color-surface-raised);
		border: 1px solid var(--color-surface-border);
		border-radius: 6px;
		padding: 0.75rem 1rem;
		margin: 0.75rem 0;
		overflow-x: auto;
		line-height: 1.5;
	}

	.md-preview :global(pre.md-code-block code) {
		background: none;
		padding: 0;
		color: var(--color-text-primary);
	}

	.md-preview :global(blockquote) {
		border-left: 3px solid var(--color-accent-muted);
		padding-left: 1rem;
		margin: 0.75rem 0;
		color: var(--color-text-secondary);
	}

	.md-preview :global(ul),
	.md-preview :global(ol) {
		margin: 0.5rem 0;
		padding-left: 1.5rem;
	}

	.md-preview :global(ul) {
		list-style-type: disc;
	}

	.md-preview :global(ol) {
		list-style-type: decimal;
	}

	.md-preview :global(li) {
		margin: 0.25rem 0;
		line-height: 1.6;
	}

	.md-preview :global(hr) {
		border: none;
		border-top: 1px solid var(--color-surface-border);
		margin: 1.5rem 0;
	}
</style>
