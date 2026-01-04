<script lang="ts">
	import { onMount } from 'svelte';
	// Use Bits UI for a nicer error dialog
	// Note: ensure `bits-ui` is installed locally
	import HeroHeader from '$lib/components/HeroHeader.svelte';
	import Dropzone from '$lib/components/Dropzone.svelte';
	import ErrorDialog from '$lib/components/ErrorDialog.svelte';
	import ShpDialog from '$lib/components/ShpDialog.svelte';
	import ToggleRow from '$lib/components/ToggleRow.svelte';
	import { status } from '$lib/stores/status';
	import type { OutputFormat, WorkerResponse } from '$lib/types';

	// Conversion options
	let outputFormat = $state<OutputFormat>('GeoParquet');
	let translateColumns = $state(true);
	let translateContents = $state(true);
	let ignoreTranslationErrors = $state(true);

	// Multi-shp selection dialog state
	let shpDialogOpen = $state(false);
	let shpFiles = $state<string[]>([]);
	let pendingZip = $state<File | null>(null);

	// Error dialog state
	let errorOpen = $state(false);
	let errorMessage = $state('');

	// worker is initialized only once, so we doe't need to track the state.
	let worker: Worker | null = null;

	onMount(() => {
		worker = new Worker(new URL('$lib/worker.ts', import.meta.url), { type: 'module' });

		// Surface worker bootstrap errors in dev
		worker.onerror = (e: ErrorEvent) => {
			console.error('Worker error:', e.message, '@', e.filename, e.lineno + ':' + e.colno);
			if (!$status.ready) showError(`ワーカーの初期化に失敗しました: ${e.message}`);
		};
		worker.onmessageerror = (e: MessageEvent) => {
			console.error('Worker message error:', e);
		};

		worker.onmessage = async (event: MessageEvent<WorkerResponse>) => {
			const data = event.data;

			const finish = () => {
				status.update((current) => ({ ...current, busy: false }));
				pendingZip = null;
			};

			if (data.ready) {
				status.update((current) => ({ ...current, ready: true }));
				return;
			}

			if (data.error) {
				showError(data.error);
				finish();
				return;
			}

			if (data.shpFileCandidates && data.shpFileCandidates.length > 1) {
				shpFiles = data.shpFileCandidates;
				shpDialogOpen = true;
				status.update((current) => ({ ...current, busy: false })); // let user choose
				return;
			}

			if (!data.output) return;

			const file = await data.output.handle.getFile();
			const url = URL.createObjectURL(file);

			const a = document.createElement('a');
			a.href = url;
			a.download = data.output.filename;
			document.body.appendChild(a);
			a.click();

			setTimeout(() => {
				URL.revokeObjectURL(url);
				a.remove();
				finish();
			}, 600);
		};

		return () => {
			worker?.terminate();
			worker = null;
		};
	});

	function processFile(file: File | undefined | null) {
		if (!file || !worker) return;
		if (!$status.ready) {
			showError('初期化中です。数秒後にもう一度お試しください。');
			return;
		}
		status.update((current) => ({ ...current, busy: true }));
		pendingZip = file;
		worker.postMessage({
			file,
			translateColumns,
			translateContents,
			ignoreTranslationErrors,
			outputFormat: outputFormat
		});
	}

	function showError(message: string) {
		errorMessage = message;
		errorOpen = true;
	}

	function chooseShp(path: string) {
		if (!worker || !pendingZip) return;
		shpDialogOpen = false;
		status.update((current) => ({ ...current, busy: true }));
		worker.postMessage({
			file: pendingZip,
			outputFormat,
			translateColumns,
			translateContents,
			ignoreTranslationErrors,
			targetShp: path
		});
	}
</script>

<div class="min-h-dvh text-slate-900 font-display flex flex-col gap-4 px-5 py-16 lg:py-24">
	<HeroHeader bind:value={outputFormat} />

	<Dropzone onError={showError} onFile={processFile} />

	<ToggleRow id="translate_colnames" bind:checked={translateColumns} label="属性名を変換する" />

	<ToggleRow
		id="translate_contents"
		bind:checked={translateContents}
		label="データの中身を変換する"
	/>

	<ToggleRow
		id="ignore_translation_errors"
		bind:checked={ignoreTranslationErrors}
		label="変換エラーを無視する"
	/>

	<ErrorDialog bind:open={errorOpen} message={errorMessage} />

	<ShpDialog bind:open={shpDialogOpen} {shpFiles} onSelect={chooseShp} />
</div>
