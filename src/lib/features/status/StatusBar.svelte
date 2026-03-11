<script lang="ts">
	import { statusStore } from './status-store';
	import type { WorkspaceStatus, SyncState } from '$lib/types/ipc';

	let status: WorkspaceStatus = $state({
		branch: null,
		sync_state: 'unknown',
		notes_count: 0,
		tags_count: 0,
		files_count: 0,
		version: '',
		last_index_update_ms: 0
	});

	statusStore.subscribe((s) => {
		status = s;
	});

	const syncLabel: Record<SyncState, string> = {
		synced: 'Clean',
		ahead: 'Ahead',
		behind: 'Behind',
		diverged: 'Diverged',
		unknown: 'Unknown'
	};

	const syncIcon: Record<SyncState, string> = {
		synced: '\u2713',
		ahead: '\u2191',
		behind: '\u2193',
		diverged: '\u21c5',
		unknown: '?'
	};
</script>

<footer
	class="flex h-6 shrink-0 items-center justify-between border-t border-surface-border bg-surface-raised px-3 text-xs text-text-secondary"
>
	<!-- Left section: branch + sync state -->
	<div class="flex items-center gap-3">
		{#if status.branch}
			<span class="flex items-center gap-1" title="Git branch: {status.branch}">
				<svg
					class="h-3 w-3"
					viewBox="0 0 16 16"
					fill="currentColor"
					aria-hidden="true"
				>
					<path
						d="M9.5 3.25a2.25 2.25 0 1 1 3 2.122V6A2.5 2.5 0 0 1 10 8.5H6a1 1 0 0 0-1 1v1.128a2.251 2.251 0 1 1-1.5 0V5.372a2.25 2.25 0 1 1 1.5 0v1.836A2.5 2.5 0 0 1 6 7h4a1 1 0 0 0 1-1v-.628A2.25 2.25 0 0 1 9.5 3.25Zm-6 0a.75.75 0 1 0 1.5 0 .75.75 0 0 0-1.5 0Zm8.25-.75a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5ZM4.25 12a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5Z"
					/>
				</svg>
				{status.branch}
			</span>
		{/if}
		<span
			class="flex items-center gap-1"
			title="Sync: {syncLabel[status.sync_state]}"
		>
			<span class="leading-none">{syncIcon[status.sync_state]}</span>
			{syncLabel[status.sync_state]}
		</span>
	</div>

	<!-- Right section: counts + version -->
	<div class="flex items-center gap-3">
		<span title="{status.notes_count} notes">{status.notes_count} notes</span>
		<span title="{status.tags_count} tags">{status.tags_count} tags</span>
		<span title="{status.files_count} files">{status.files_count} files</span>
		<span class="text-text-muted" title="Vigil v{status.version}">v{status.version}</span>
	</div>
</footer>
