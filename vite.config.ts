import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	build: {
		rollupOptions: {
			output: {
				manualChunks(id) {
					const normalizedId = id.replaceAll('\\', '/');
					if (normalizedId.includes('/node_modules/svelte/')) {
						return 'svelte-runtime';
					}
				}
			}
		}
	}
});
