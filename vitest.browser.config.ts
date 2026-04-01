import { defineConfig } from 'vite-plus';
import { playwright } from 'vite-plus/test/browser-playwright';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
	plugins: [svelte()],
	resolve: {
		alias: {
			$lib: import.meta.dirname + '/src/lib'
		}
	},
	test: {
		browser: {
			enabled: true,
			provider: playwright(),
			// https://vitest.dev/config/browser/playwright
			instances: [{ browser: 'chromium', headless: true }]
		}
	}
});
