import { join } from 'path';
import { dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { sveltePreprocess } from 'svelte-preprocess';

// this is unforntunate but else svelte lsp will throw an error
const sassDir = join(dirname(fileURLToPath(import.meta.url)), 'src/sass');

/** @type {import('@sveltejs/vite-plugin-svelte').SvelteConfig} */
const config = {
	preprocess: [
		sveltePreprocess({
			scss: {
				prependData: `@use '${sassDir}/vars.scss' as *;`,
			},
		}),
	],
	compilerOptions: {
		// runes: true,
	},
};

export default config;
