import { resolve } from 'path';
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import crelte from 'crelte/vite';

// https://vitejs.dev/config/
export default defineConfig(() => {
	return {
		css: {
			devSourcemap: true,
		},
		plugins: [svelte(), crelte()],
		resolve: {
			alias: [{ find: '@', replacement: resolve(__dirname, 'src') }],
		},
		server: {
			allowedHosts: ['.ddev.site'],
			hmr: {
				protocol: 'wss',
				clientPort: 443,
			},
		},
	};
});
