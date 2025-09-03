import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';

export default defineConfig({
	plugins: [sveltekit(), wasm()],
	worker: {
		format: 'es',
		plugins: () => [wasm()]
	},
	base: '/ksj2gp'
});
