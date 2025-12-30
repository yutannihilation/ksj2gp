import { expect, test } from 'vitest';
import { render } from 'vitest-browser-svelte';
import HeroHeader from './HeroHeader.svelte';
import HeroHeaderTestHost from './HeroHeaderTestHost.svelte';

test('renders the title and default format', async () => {
	const screen = render(HeroHeader);

	await expect.element(screen.getByText('KSJ')).toBeVisible();
	await expect.element(screen.getByText('GeoParquet')).toBeVisible();
});

test('opens the format select', async () => {
	const screen = render(HeroHeader);

	await screen.getByRole('button', { name: '出力形式を選択' }).click();
	await expect.element(screen.getByText('GeoJson')).toBeVisible();
});

test('updates bound value when selecting a format', async () => {
	const screen = render(HeroHeaderTestHost);

	await expect.element(screen.getByText('Selected: GeoParquet')).toBeVisible();
	await screen.getByRole('button', { name: '出力形式を選択' }).click();
	await screen.getByText('GeoJson').click();
	await expect.element(screen.getByText('Selected: GeoJson')).toBeVisible();
});
