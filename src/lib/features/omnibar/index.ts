// Barrel file for omnibar feature.
export { default as Omnibar } from './Omnibar.svelte';
export { default as OmnibarItem } from './OmnibarItem.svelte';
export { omnibarStore } from './omnibar-store.svelte';
export type {
	OmnibarResult,
	OmnibarFileResult,
	OmnibarContentResult,
	OmnibarCommand,
	OmnibarCommandResult
} from './omnibar-store.svelte';
export { parseOmnibarQuery, ensureCommandQuery } from './omnibar-parser';
export type { ParsedOmnibarQuery } from './omnibar-parser';
