import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vitest/config';
import wasm from 'vite-plugin-wasm';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
	plugins: [sveltekit(), tailwindcss(), wasm()],
	worker: {
		format: 'es',
		plugins: () => [wasm()]
	},
	// It seems this is necessary for `vite dev` to work
	optimizeDeps: {
		exclude: ['ksj2gp']
	},
	base: '/ksj2gp',
	test: {
		include: ['src/**/*.{test,spec}.{js,ts}']
	}
});
