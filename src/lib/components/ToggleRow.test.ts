/// <reference types="vitest/browser/context" />
import { expect, test } from 'vite-plus/test';
import { render } from 'vitest-browser-svelte';
import ToggleRow from './ToggleRow.svelte';
import ToggleRowTestHost from './ToggleRowTestHost.svelte';

test('renders the label', async () => {
	const screen = render(ToggleRow, {
		id: 'feature-toggle',
		label: 'Enable feature'
	});

	// @ts-expect-error expect.element types not resolved under vite-plus alias
	await expect.element(screen.getByText('Enable feature')).toBeVisible();
});

test('updates bound checked when toggled', async () => {
	const screen = render(ToggleRowTestHost);

	// @ts-expect-error expect.element types not resolved under vite-plus alias
	await expect.element(screen.getByText('Checked: false')).toBeVisible();
	await screen.getByRole('switch').click();
	// @ts-expect-error expect.element types not resolved under vite-plus alias
	await expect.element(screen.getByText('Checked: true')).toBeVisible();
});
