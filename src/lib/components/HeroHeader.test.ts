/// <reference types="vitest/browser/context" />
import { expect, test } from 'vite-plus/test';
import { render } from 'vitest-browser-svelte';
import HeroHeader from './HeroHeader.svelte';
import HeroHeaderTestHost from './HeroHeaderTestHost.svelte';

test('renders the title and default format', async () => {
	const screen = render(HeroHeader);

	// @ts-ignore expect.element types not resolved under vite-plus alias
	await expect.element(screen.getByText('KSJ')).toBeVisible();
	// @ts-ignore expect.element types not resolved under vite-plus alias
	await expect.element(screen.getByText('GeoParquet')).toBeVisible();
});

test('opens the format select', async () => {
	const screen = render(HeroHeader);

	await screen.getByRole('button', { name: '出力形式を選択' }).click();
	// @ts-ignore expect.element types not resolved under vite-plus alias
	await expect.element(screen.getByText('GeoJson')).toBeVisible();
});

test('updates bound value when selecting a format', async () => {
	const screen = render(HeroHeaderTestHost);

	// @ts-ignore expect.element types not resolved under vite-plus alias
	await expect.element(screen.getByText('Selected: GeoParquet')).toBeVisible();
	await screen.getByRole('button', { name: '出力形式を選択' }).click();
	await screen.getByText('GeoJson').click();
	// @ts-ignore expect.element types not resolved under vite-plus alias
	await expect.element(screen.getByText('Selected: GeoJson')).toBeVisible();
});
