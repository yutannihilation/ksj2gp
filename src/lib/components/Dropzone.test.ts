import { expect, test, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import Dropzone from './Dropzone.svelte';

function setInputFile(input: HTMLInputElement, file: File) {
	const dataTransfer = new DataTransfer();
	dataTransfer.items.add(file);
	Object.defineProperty(input, 'files', {
		value: dataTransfer.files,
		configurable: true
	});
	input.dispatchEvent(new Event('change', { bubbles: true }));
}

test('calls onFile for a zip upload', async () => {
	const onFile = vi.fn();
	const onError = vi.fn();
	const screen = render(Dropzone, {
		ready: true,
		busy: false,
		onFile,
		onError
	});

	const input = screen.container.querySelector('input[type="file"]');
	if (!input) throw new Error('file input not found');
	const file = new File(['test'], 'sample.zip', { type: 'application/zip' });
	setInputFile(input as HTMLInputElement, file);

	await expect.poll(() => onFile.mock.calls.length).toBe(1);
	await expect.poll(() => onError.mock.calls.length).toBe(0);
});

test('calls onError for a non-zip upload', async () => {
	const onFile = vi.fn();
	const onError = vi.fn();
	const screen = render(Dropzone, {
		ready: true,
		busy: false,
		onFile,
		onError
	});

	const input = screen.container.querySelector('input[type="file"]');
	if (!input) throw new Error('file input not found');
	const file = new File(['test'], 'notes.txt', { type: 'text/plain' });
	setInputFile(input as HTMLInputElement, file);

	await expect.poll(() => onError.mock.calls.length).toBe(1);
	await expect.poll(() => onFile.mock.calls.length).toBe(0);
});
