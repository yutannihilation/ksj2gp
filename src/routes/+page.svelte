<script lang="ts">
	import Icon from '@iconify/svelte';
	import { onMount } from 'svelte';
	// Use Bits UI for a nicer error dialog
	// Note: ensure `bits-ui` is installed locally
	import { Dialog } from 'bits-ui';
	import HeroHeader from '$lib/components/HeroHeader.svelte';
	import ErrorDialog from '$lib/components/ErrorDialog.svelte';
	import ToggleRow from '$lib/components/ToggleRow.svelte';
	import type { OutputFormat, WorkerResponse } from '$lib/types';

	let inputEl: HTMLInputElement;
	let dragover = false;
	let busy = false;
	let worker: Worker | null = null;
	let ready = false;
	let outputFormat: OutputFormat = 'GeoParquet';
	let translateColumns = true;
	let translateContents = true;
	let ignoreTranslationErrors = true;

	// Multi-shp selection dialog state
	let shpDialogOpen = false;
	let shpOptions: string[] = [];
	let pendingZip: File | null = null;

	// Error dialog state
	let errorOpen = false;
	let errorMessage = '';
	$: bigLoading = !ready || busy;

	onMount(() => {
		worker = new Worker(new URL('$lib/worker.ts', import.meta.url), { type: 'module' });

		// Surface worker bootstrap errors in dev
		worker.onerror = (e: ErrorEvent) => {
			console.error('Worker error:', e.message, '@', e.filename, e.lineno + ':' + e.colno);
			if (!ready) showError(`ワーカーの初期化に失敗しました: ${e.message}`);
		};
		worker.onmessageerror = (e: MessageEvent) => {
			console.error('Worker message error:', e);
		};

		worker.onmessage = async (event: MessageEvent<WorkerResponse>) => {
			const data = event.data;

			const finish = () => {
				busy = false;
				pendingZip = null;
			};

			if (data.ready) {
				ready = true;
				return;
			}

			if (data.error) {
				showError(data.error);
				finish();
				return;
			}

			if (data.shpFileCandidates && data.shpFileCandidates.length > 1) {
				shpOptions = data.shpFileCandidates;
				shpDialogOpen = true;
				busy = false; // let user choose
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

	function pick() {
		if (!ready || busy) return;
		inputEl?.click();
	}

	function processFile(file: File | undefined | null) {
		if (!file || !worker) return;
		if (!ready) {
			showError('初期化中です。数秒後にもう一度お試しください。');
			return;
		}
		busy = true;
		pendingZip = file;
		worker.postMessage({
			file,
			translateColumns,
			translateContents,
			ignoreTranslationErrors,
			outputFormat: outputFormat
		});
	}

	function onInputChange(e: Event) {
		const file = (e.currentTarget as HTMLInputElement).files?.[0];
		processFile(file ?? null);
	}

	// Drag & drop handlers
	function onDragOver(e: DragEvent) {
		e.preventDefault();
		dragover = true;
	}

	function onDragLeave() {
		dragover = false;
	}

	function onDrop(e: DragEvent) {
		e.preventDefault();
		dragover = false;
		const file = e.dataTransfer?.files?.[0];
		if (!file) return;
		if (!file.name.toLowerCase().endsWith('.zip')) {
			showError('ZIP ファイルを選択してください');
			return;
		}
		processFile(file);
	}

	function showError(message: string) {
		errorMessage = message;
		errorOpen = true;
	}

	function chooseShp(path: string, outputFormat: OutputFormat) {
		if (!worker || !pendingZip) return;
		shpDialogOpen = false;
		busy = true;
		worker.postMessage({
			file: pendingZip,
			outputFormat,
			translateColumns,
			translateContents,
			ignoreTranslationErrors,
			targetShp: path
		});
	}

	function cancelShpDialog() {
		shpDialogOpen = false;
		busy = false;
	}
</script>

<div class="min-h-dvh text-slate-900 font-display flex flex-col gap-4 px-5 py-16 lg:py-24">
	<HeroHeader bind:value={outputFormat} />

	<div
		id="dropzone"
		class={`relative grid place-items-center ` +
			// Make the box more square-like: width drives height
			`w-1/3 aspect-square mx-auto min-w-120 ` +
			`p-16 ` +
			`border-4 border-dashed border-gray-400/50 ` +
			`outline-none transition ` +
			`${dragover ? 'border-sky-400/80 bg-sky-400/20' : ''}`}
		role="button"
		tabindex="0"
		aria-label="ファイルのドラッグ＆ドロップ領域"
		ondragover={onDragOver}
		ondragleave={onDragLeave}
		ondrop={onDrop}
		onkeydown={(e) => {
			if (e.key === 'Enter' || e.key === ' ') {
				pick();
			}
		}}
	>
		<div class="grid place-items-center text-center">
			{#if bigLoading}
				<div class="animate-spin" aria-label="読み込み中"></div>
			{:else}
				<div class="" aria-label="読み込み中">
					<Icon icon="formkit:zip" width="6em" color="#bbb" />
				</div>
			{/if}
			<div class="text-gray-400 text-center font-bold leading-relaxed text-2xl py-8">
				{#if busy}
					変換中...
				{:else if !ready}
					読み込み中…
				{:else}
					ここに ZIP ファイルを<br />ドラッグ＆ドロップ<br />または
					<button
						class="text-blue-600 hover:underline"
						type="button"
						onclick={pick}
						disabled={!ready || busy}
					>
						ZIP ファイルを選択
					</button>
				{/if}
				<input bind:this={inputEl} type="file" accept=".zip" hidden onchange={onInputChange} />
			</div>

			{#if busy && !bigLoading}
				<div class="absolute right-3 bottom-3 flex items-center gap-2 text-indigo-300/80 text-sm">
					<span class="w-4.5 h-4.5 border-2 border-white/25 animate-spin" aria-hidden="true"></span>
					<span class="sr-only">処理中</span>
				</div>
			{/if}

			{#if !busy && !ready && !bigLoading}
				<div class="absolute right-3 bottom-3 flex items-center gap-2 text-indigo-300/80 text-sm">
					<span
						class="w-4.5 h-4.5 border-2 border-white/25 border-t-sky-400 animate-spin"
						aria-hidden="true"
					></span>
					<span aria-hidden="true">初期化中…</span>
				</div>
			{/if}
		</div>
	</div>

	<ToggleRow
		id="translate_colnames"
		name="translate_colnames"
		bind:checked={translateColumns}
		label="属性名を変換する"
		switchClass="disabled:opacity-20"
	/>

	<ToggleRow
		id="translate_contents"
		name="translate_contents"
		bind:checked={translateContents}
		label="データの中身を変換する"
		switchClass="disabled:opacity-10"
		thumbClass="disabled:opacity-0"
	/>

	<ToggleRow
		id="ignore_translation_errors"
		name="ignore_translation_errors"
		bind:checked={ignoreTranslationErrors}
		label="変換エラーを無視する"
		switchClass="disabled:opacity-20"
	/>

	<ErrorDialog bind:open={errorOpen} message={errorMessage} />

	<!-- Bits UI: Shapefile selection dialog -->
	<Dialog.Root bind:open={shpDialogOpen}>
		<Dialog.Content class="fixed inset-0 grid place-items-center p-4 z-50">
			<div
				class="bg-slate-900 text-indigo-50 border border-slate-700 rounded-xl p-4 w-full max-w-xl shadow-2xl"
			>
				<Dialog.Title class="font-bold mb-1">Shapefile を選択</Dialog.Title>
				<div class="text-indigo-200/80 mb-4">
					ZIP には複数の .shp が含まれています。変換するファイルを選択してください。
				</div>
				<div class="max-h-72 overflow-auto grid gap-2 mb-4">
					{#each shpOptions as opt (opt)}
						<button
							type="button"
							class="text-left rounded-lg w-full px-3 py-2 bg-slate-800/70 hover:bg-slate-800 border border-slate-700/70"
							onclick={() => chooseShp(opt, outputFormat)}
						>
							{opt}
						</button>
					{/each}
				</div>
				<div class="flex justify-end">
					<Dialog.Close>
						<button
							type="button"
							class="rounded-lg bg-slate-700 text-white px-4 py-2 font-bold tracking-tight"
							onclick={cancelShpDialog}
						>
							キャンセル
						</button>
					</Dialog.Close>
				</div>
			</div>
		</Dialog.Content>
		<Dialog.Overlay class="fixed inset-0 bg-black/50 backdrop-blur-sm z-40" />
	</Dialog.Root>
</div>
