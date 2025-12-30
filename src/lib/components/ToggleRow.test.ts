import { expect, test } from 'vitest';
import { render } from 'vitest-browser-svelte';
import ToggleRow from './ToggleRow.svelte';
import ToggleRowTestHost from './ToggleRowTestHost.svelte';

test('renders the label', async () => {
	const screen = render(ToggleRow, {
		id: 'feature-toggle',
		label: 'Enable feature'
	});

	await expect.element(screen.getByText('Enable feature')).toBeVisible();
});

test('updates bound checked when toggled', async () => {
	const screen = render(ToggleRowTestHost);

	await expect.element(screen.getByText('Checked: false')).toBeVisible();
	await screen.getByRole('switch').click();
	await expect.element(screen.getByText('Checked: true')).toBeVisible();
});
