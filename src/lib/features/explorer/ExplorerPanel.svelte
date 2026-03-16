<script lang="ts">
	/**
	 * ExplorerPanel -- File tree panel shell for the sidebar.
	 * Displays workspace name header and renders the FileTree component
	 * using IPC-backed state from the explorer store.
	 */

	import { explorerStore } from './explorer-store.svelte';
	import FileTree from './FileTree.svelte';
</script>

<div class="flex h-full flex-col">
	<div class="flex items-center gap-1.5 px-3 py-2">
		<svg
			class="h-3.5 w-3.5 shrink-0 text-text-muted"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="1.5"
		>
			<path d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
		</svg>
		<span class="truncate text-xs font-medium text-text-secondary">
			{explorerStore.workspaceName || 'No workspace'}
		</span>
	</div>

	{#if explorerStore.loading}
		<!-- Loading state -->
		<div class="flex flex-col items-center gap-2 px-3 py-8 text-center">
			<svg
				class="h-6 w-6 animate-spin text-text-muted"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
			>
				<circle cx="12" cy="12" r="10" stroke-opacity="0.25" />
				<path d="M12 2a10 10 0 019.95 9" stroke-opacity="0.75" />
			</svg>
			<p class="text-xs text-text-muted">Loading workspace...</p>
		</div>
	{:else if explorerStore.error}
		<!-- Error state -->
		<div class="flex flex-col items-center gap-2 px-3 py-8 text-center">
			<p class="text-xs text-red-400">{explorerStore.error}</p>
		</div>
	{:else if explorerStore.tree.length === 0}
		<!-- Empty state: no workspace opened or empty workspace -->
		<div class="flex flex-col items-center gap-2 px-3 py-8 text-center">
			<p class="text-xs text-text-muted">
				{explorerStore.workspaceName
					? 'This workspace is empty.'
					: 'Open a workspace to browse files.'}
			</p>
		</div>
	{:else}
		<div class="px-3 pb-2">
			<div class="rounded-md border border-surface-border bg-surface-overlay/40 px-2 py-1.5">
				<div class="text-[10px] font-semibold uppercase tracking-wider text-text-muted">
					Notes
				</div>
				<div class="text-sm font-semibold text-text-primary">
					{explorerStore.notesCount}
				</div>
			</div>
		</div>

		{#if explorerStore.collections.length > 0}
			<div class="px-3 pb-2">
				<div
					class="mb-1 text-[10px] font-semibold uppercase tracking-wider text-text-muted"
				>
					Collections
				</div>
				<ul class="space-y-1">
					{#each explorerStore.collections as collection (collection.path)}
						<li
							class="flex items-center justify-between rounded-sm px-1 py-0.5 text-xs text-text-secondary"
						>
							<span class="truncate">{collection.name}</span>
							<span class="shrink-0 text-text-muted">
								{collection.notesCount} notes / {collection.filesCount} files
							</span>
						</li>
					{/each}
				</ul>
			</div>
		{/if}

		<div class="flex-1 overflow-y-auto px-1 pb-2">
			<FileTree
				nodes={explorerStore.tree}
				expandedDirs={explorerStore.expandedDirs}
				selectedFile={explorerStore.selectedFile}
				onToggle={(path) => explorerStore.toggleExpand(path)}
				onSelect={(path) => explorerStore.selectFile(path)}
			/>
		</div>
	{/if}
</div>
