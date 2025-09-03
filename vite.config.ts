import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
	plugins: [sveltekit(), tailwindcss(), wasm()],
	worker: {
		format: 'es',
		plugins: () => [wasm()]
	},
	base: '/ksj2gp'
});
