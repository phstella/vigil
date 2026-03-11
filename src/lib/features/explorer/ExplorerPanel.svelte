<script lang="ts">
	/**
	 * ExplorerPanel -- File tree panel shell for the sidebar.
	 * Displays workspace name header and renders the FileTree component
	 * using state from the explorer store.
	 */

	import { explorerStore } from './explorer-store';
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
			{explorerStore.workspaceName}
		</span>
	</div>

	<div class="flex-1 overflow-y-auto px-1 pb-2">
		<FileTree
			nodes={explorerStore.tree}
			expandedDirs={explorerStore.expandedDirs}
			selectedFile={explorerStore.selectedFile}
			onToggle={(path) => explorerStore.toggleExpand(path)}
			onSelect={(path) => explorerStore.selectFile(path)}
		/>
	</div>
</div>
