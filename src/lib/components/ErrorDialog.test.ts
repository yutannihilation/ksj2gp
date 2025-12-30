import { expect, test } from 'vitest';
import { render } from 'vitest-browser-svelte';
import ErrorDialog from './ErrorDialog.svelte';
import ErrorDialogTestHost from './ErrorDialogTestHost.svelte';

test('renders the title and message when open', async () => {
	const screen = render(ErrorDialog, {
		open: true,
		message: 'Something went wrong.'
	});

	await expect.element(screen.getByText('Something went wrong.')).toBeVisible();
});

test('closes via the button and updates bound state', async () => {
	const screen = render(ErrorDialogTestHost);

	await expect.element(screen.getByText('Open: true')).toBeVisible();
	await screen.getByText('閉じる').click();
	await expect.element(screen.getByText('Open: false')).toBeVisible();
});
