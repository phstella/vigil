<script lang="ts">
	/**
	 * MermaidBlock -- Renders a single Mermaid diagram from a raw definition string.
	 *
	 * Uses the mermaid library to parse and render diagrams as inline SVG.
	 * On invalid/malformed input, displays a graceful error fallback showing
	 * the raw source alongside a human-readable error message.
	 *
	 * Security: mermaid is initialized with securityLevel 'strict' (default).
	 * No remote resources are loaded.
	 */

	import { onMount } from 'svelte';

	let { source }: { source: string } = $props();

	let containerRef: HTMLDivElement | null = $state(null);
	let errorMessage = $state<string | null>(null);

	// Unique ID counter for mermaid render calls (must be unique per render).
	let renderCounter = 0;

	onMount(() => {
		void renderDiagram();
	});

	async function renderDiagram() {
		if (!containerRef || !source.trim()) return;

		try {
			// Dynamic import to code-split mermaid out of the main bundle.
			const mermaid = await import('mermaid');

			mermaid.default.initialize({
				startOnLoad: false,
				securityLevel: 'strict',
				theme: 'dark',
				fontFamily: 'var(--font-mono, monospace)',
				// Suppress log noise in production.
				logLevel: 4 // ERROR only
			});

			renderCounter += 1;
			const id = `mermaid-block-${Date.now()}-${renderCounter}`;

			const { svg } = await mermaid.default.render(id, source.trim());
			if (containerRef) {
				// eslint-disable-next-line svelte/no-dom-manipulating -- mermaid outputs raw SVG strings; direct innerHTML is the only viable injection path.
				containerRef.innerHTML = svg;
				errorMessage = null;
			}
		} catch (err: unknown) {
			const msg = err instanceof Error ? err.message : String(err);
			// Strip common mermaid prefix noise for readability.
			errorMessage = msg.replace(/^[\s\S]*?Parse error on line/, 'Parse error on line');

			// Clean up any orphaned mermaid error elements in the DOM.
			if (containerRef) {
				// eslint-disable-next-line svelte/no-dom-manipulating -- clearing container after mermaid render failure.
				containerRef.innerHTML = '';
			}
			// Also remove any stray mermaid error divs that mermaid appends to document.body.
			cleanupMermaidErrors();
		}
	}

	function cleanupMermaidErrors() {
		if (typeof document === 'undefined') return;
		const orphans = document.querySelectorAll('#d' + 'mermaid-block');
		orphans.forEach((el) => el.remove());
	}
</script>

<div class="md-mermaid-container">
	{#if errorMessage}
		<div class="md-mermaid-error">
			<div class="md-mermaid-error-label">Mermaid diagram error</div>
			<div class="md-mermaid-error-message">{errorMessage}</div>
			<pre class="md-mermaid-error-source"><code>{source}</code></pre>
		</div>
	{/if}
	<div bind:this={containerRef} class="md-mermaid-render" class:hidden={!!errorMessage}></div>
</div>

<style>
	.md-mermaid-container {
		margin: 0.75rem 0;
	}

	.md-mermaid-render {
		display: flex;
		justify-content: center;
		overflow-x: auto;
		padding: 0.5rem 0;
	}

	.md-mermaid-render.hidden {
		display: none;
	}

	/* Ensure rendered SVG scales properly */
	.md-mermaid-render :global(svg) {
		max-width: 100%;
		height: auto;
	}

	.md-mermaid-error {
		background: var(--color-surface-raised, #1a1e2e);
		border: 1px solid rgba(239, 68, 68, 0.3);
		border-radius: 6px;
		padding: 0.75rem 1rem;
		margin: 0.75rem 0;
	}

	.md-mermaid-error-label {
		font-size: 0.75rem;
		font-weight: 600;
		color: rgb(239, 68, 68);
		margin-bottom: 0.375rem;
		text-transform: uppercase;
		letter-spacing: 0.025em;
	}

	.md-mermaid-error-message {
		font-size: 0.8rem;
		color: var(--color-text-secondary, #94a3b8);
		margin-bottom: 0.5rem;
		font-family: var(--font-mono, monospace);
		white-space: pre-wrap;
		word-break: break-word;
	}

	.md-mermaid-error-source {
		font-family: var(--font-mono, monospace);
		font-size: 0.8rem;
		background: var(--color-surface-overlay, rgba(0, 0, 0, 0.3));
		border-radius: 4px;
		padding: 0.5rem 0.75rem;
		overflow-x: auto;
		color: var(--color-text-muted, #64748b);
		margin: 0;
	}

	.md-mermaid-error-source code {
		background: none;
		padding: 0;
	}
</style>
