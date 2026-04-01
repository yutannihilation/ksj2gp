/// <reference types="vitest/browser/context" />
import { expect, test } from 'vite-plus/test';
import { render } from 'vitest-browser-svelte';
import ErrorDialog from './ErrorDialog.svelte';
import ErrorDialogTestHost from './ErrorDialogTestHost.svelte';

test('renders the title and message when open', async () => {
	const screen = render(ErrorDialog, {
		open: true,
		message: 'Something went wrong.'
	});

	// @ts-expect-error expect.element types not resolved under vite-plus alias
	await expect.element(screen.getByText('Something went wrong.')).toBeVisible();
});

test('closes via the button and updates bound state', async () => {
	const screen = render(ErrorDialogTestHost);

	// @ts-expect-error expect.element types not resolved under vite-plus alias
	await expect.element(screen.getByText('Open: true')).toBeVisible();
	await screen.getByText('閉じる').click();
	// @ts-expect-error expect.element types not resolved under vite-plus alias
	await expect.element(screen.getByText('Open: false')).toBeVisible();
});
