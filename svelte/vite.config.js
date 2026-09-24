import { resolve } from 'path';
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import crelte from 'crelte/vite';
import postcssLogical from 'postcss-logical';

// https://vitejs.dev/config/
export default defineConfig(({ command }) => {
	return {
		css: {
			devSourcemap: true,
			// Keep logical properties readable during development and convert them
			// to physical properties for production browser compatibility.
			postcss:
				command === 'build'
					? { plugins: [postcssLogical({ preserve: false })] }
					: undefined,
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
