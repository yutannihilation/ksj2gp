import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite-plus';
import wasm from 'vite-plugin-wasm';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
	staged: {
		'*': 'vp check --fix'
	},
	lint: {
		jsPlugins: ['eslint-plugin-svelte'],
		// oxlint's correctness category covers ESLint's recommended rules.
		categories: {
			correctness: 'error'
		},
		ignorePatterns: [
			'**/node_modules',
			'**/.output',
			'**/.vercel',
			'**/.netlify',
			'**/.wrangler',
			'.svelte-kit',
			'build',
			'dist',
			'**/.DS_Store',
			'**/Thumbs.db',
			'**/.env',
			'**/.env.*',
			'!**/.env.example',
			'!**/.env.test',
			'**/vite.config.js.timestamp-*',
			'**/vite.config.ts.timestamp-*',
			'npm/*',
			'!npm/package.json',
			'.claude/settings.local.json'
		],
		rules: {
			// typescript-eslint's recommended rules are not covered by oxlint's
			// correctness category, so enable them explicitly.
			'typescript/ban-ts-comment': 'error',
			'typescript/no-duplicate-enum-values': 'error',
			'typescript/no-empty-object-type': 'error',
			'typescript/no-explicit-any': 'error',
			'typescript/no-extra-non-null-assertion': 'error',
			'typescript/no-misused-new': 'error',
			'typescript/no-namespace': 'error',
			'typescript/no-non-null-asserted-optional-chain': 'error',
			'typescript/no-require-imports': 'error',
			'typescript/no-this-alias': 'error',
			'typescript/no-unnecessary-type-constraint': 'error',
			'typescript/no-unsafe-declaration-merging': 'error',
			'typescript/no-unsafe-function-type': 'error',
			'typescript/no-wrapper-object-types': 'error',
			'typescript/prefer-as-const': 'error',
			'typescript/prefer-namespace-keyword': 'error',
			'typescript/triple-slash-reference': 'error',
			// Stylistic rules from typescript-eslint's recommended set.
			'no-array-constructor': 'error',
			// eslint-plugin-svelte's recommended rules. jsPlugin rules are off
			// by default, so they need to be listed explicitly.
			'svelte/comment-directive': 'error',
			'svelte/infinite-reactive-loop': 'error',
			'svelte/no-at-debug-tags': 'warn',
			'svelte/no-at-html-tags': 'error',
			'svelte/no-dom-manipulating': 'error',
			'svelte/no-dupe-else-if-blocks': 'error',
			'svelte/no-dupe-on-directives': 'error',
			'svelte/no-dupe-style-properties': 'error',
			'svelte/no-dupe-use-directives': 'error',
			'svelte/no-export-load-in-svelte-module-in-kit-pages': 'error',
			'svelte/no-immutable-reactive-statements': 'error',
			'svelte/no-inner-declarations': 'error',
			'svelte/no-inspect': 'warn',
			'svelte/no-navigation-without-resolve': 'error',
			'svelte/no-not-function-handler': 'error',
			'svelte/no-object-in-text-mustaches': 'error',
			'svelte/no-raw-special-elements': 'error',
			'svelte/no-reactive-functions': 'error',
			'svelte/no-reactive-literals': 'error',
			'svelte/no-reactive-reassign': 'error',
			'svelte/no-shorthand-style-property-overrides': 'error',
			'svelte/no-store-async': 'error',
			'svelte/no-svelte-internal': 'error',
			'svelte/no-unknown-style-directive-property': 'error',
			'svelte/no-unnecessary-state-wrap': 'error',
			'svelte/no-unused-props': 'error',
			'svelte/no-unused-svelte-ignore': 'error',
			'svelte/no-useless-children-snippet': 'error',
			'svelte/no-useless-mustaches': 'error',
			'svelte/prefer-svelte-reactivity': 'error',
			'svelte/prefer-writable-derived': 'error',
			'svelte/require-each-key': 'error',
			'svelte/require-event-dispatcher-types': 'error',
			'svelte/require-store-reactive-access': 'error',
			'svelte/system': 'error',
			'svelte/valid-each-key': 'error',
			'svelte/valid-prop-names-in-kit-pages': 'error'
		},
		overrides: [
			{
				// Turn off core rules that the TypeScript compiler already checks
				// (same as typescript-eslint's eslint-recommended) to avoid
				// duplicate errors when typeCheck is enabled.
				files: ['**/*.ts', '**/*.tsx', '**/*.mts', '**/*.cts'],
				rules: {
					'constructor-super': 'off',
					'getter-return': 'off',
					'no-class-assign': 'off',
					'no-const-assign': 'off',
					'no-dupe-class-members': 'off',
					'no-dupe-keys': 'off',
					'no-func-assign': 'off',
					'no-import-assign': 'off',
					'no-new-native-nonconstructor': 'off',
					'no-obj-calls': 'off',
					'no-redeclare': 'off',
					'no-setter-return': 'off',
					'no-this-before-super': 'off',
					'no-undef': 'off',
					'no-unreachable': 'off',
					'no-unsafe-negation': 'off',
					'no-with': 'off',
					'no-var': 'error',
					'prefer-const': 'error',
					'prefer-rest-params': 'error',
					'prefer-spread': 'error'
				}
			},
			{
				files: ['*.svelte', '**/*.svelte'],
				rules: {
					'no-inner-declarations': 'off',
					'no-self-assign': 'off'
				},
				jsPlugins: ['eslint-plugin-svelte']
			}
		],
		options: {
			typeAware: true,
			typeCheck: true
		}
	},
	fmt: {
		useTabs: true,
		singleQuote: true,
		trailingComma: 'none',
		printWidth: 100,
		sortPackageJson: false,
		ignorePatterns: [
			'package-lock.json',
			'yarn.lock',
			'bun.lock',
			'bun.lockb',
			'/static/',
			'/rust/',
			'CLAUDE.md'
		]
	},
	plugins: [sveltekit(), tailwindcss(), wasm()],
	worker: {
		format: 'es',
		plugins: () => [wasm()]
	},
	// It seems this is necessary for `vite dev` to work
	optimizeDeps: {
		exclude: ['ksj2gp']
	},
	test: {
		include: ['src/**/*.{test,spec}.{js,ts}']
	}
});
