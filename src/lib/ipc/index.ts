// Barrel file for Tauri IPC wrappers.

export { isVigilError } from './tauri';

export {
	openWorkspace,
	listDir,
	readFile,
	writeFile,
	createNote,
	renameFile,
	deleteFile
} from './files';

export { fuzzyFind, searchContent, getAllTags, getFilesByTag } from './search';

export { getGitHunks, getGitStatus } from './git';

export { getBacklinks } from './links';

export { workspaceStatus } from './status';

export {
	EVENTS,
	onIndexUpdated,
	onIndexReady,
	onGitHunks,
	onStatusUpdated,
	onFsRenamed
} from './events';
