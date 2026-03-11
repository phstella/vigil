<script lang="ts">
	// Minimal "zen" title bar with Tauri drag region and window controls.
	// Window control buttons dispatch Tauri window actions when the backend is available.

	async function minimize() {
		try {
			const { getCurrentWindow } = await import('@tauri-apps/api/window');
			await getCurrentWindow().minimize();
		} catch {
			// Tauri API unavailable (e.g. during SSR or browser preview)
		}
	}

	async function toggleMaximize() {
		try {
			const { getCurrentWindow } = await import('@tauri-apps/api/window');
			await getCurrentWindow().toggleMaximize();
		} catch {
			// Tauri API unavailable
		}
	}

	async function close() {
		try {
			const { getCurrentWindow } = await import('@tauri-apps/api/window');
			await getCurrentWindow().close();
		} catch {
			// Tauri API unavailable
		}
	}
</script>

<header
	data-tauri-drag-region
	class="flex h-8 shrink-0 select-none items-center justify-between bg-surface-base px-3"
>
	<span data-tauri-drag-region class="text-xs font-medium tracking-wide text-text-secondary">
		Vigil
	</span>

	<div class="flex items-center gap-1">
		<button
			onclick={minimize}
			class="flex h-5 w-5 items-center justify-center rounded-sm text-text-muted transition-colors hover:bg-surface-overlay hover:text-text-primary"
			aria-label="Minimize"
		>
			<svg class="h-3 w-3" viewBox="0 0 12 12" fill="none">
				<rect x="2" y="5.5" width="8" height="1" rx="0.5" fill="currentColor" />
			</svg>
		</button>

		<button
			onclick={toggleMaximize}
			class="flex h-5 w-5 items-center justify-center rounded-sm text-text-muted transition-colors hover:bg-surface-overlay hover:text-text-primary"
			aria-label="Maximize"
		>
			<svg class="h-3 w-3" viewBox="0 0 12 12" fill="none">
				<rect x="2" y="2" width="8" height="8" rx="1" stroke="currentColor" fill="none" />
			</svg>
		</button>

		<button
			onclick={close}
			class="flex h-5 w-5 items-center justify-center rounded-sm text-text-muted transition-colors hover:bg-error/20 hover:text-error"
			aria-label="Close"
		>
			<svg class="h-3 w-3" viewBox="0 0 12 12" fill="none">
				<path d="M3 3l6 6M9 3l-6 6" stroke="currentColor" stroke-width="1.5" />
			</svg>
		</button>
	</div>
</header>
