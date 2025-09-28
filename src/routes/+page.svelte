<script lang="ts">
	import Icon from '@iconify/svelte';
	import { onMount } from 'svelte';
	// Use Bits UI for a nicer error dialog
	// Note: ensure `bits-ui` is installed locally
	import { Dialog } from 'bits-ui';
	import type { OutputFormat, WorkerResponse } from '$lib/types';

	let inputEl: HTMLInputElement;
	let dragover = false;
	let busy = false;
	let worker: Worker | null = null;
	let ready = false;
	let outputFormat: OutputFormat = 'GeoParquet';

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
		worker.postMessage({ file, outputFormat: outputFormat });
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
		worker.postMessage({ file: pendingZip, outputFormat, targetShp: path });
	}

	function cancelShpDialog() {
		shpDialogOpen = false;
		busy = false;
	}
</script>

<div
	class="min-h-dvh text-slate-900 flex flex-col gap-8 sm:gap-10 lg:gap-12 py-10 sm:py-12 lg:py-16 px-5 sm:px-6 lg:justify-center"
>
	<header class="text-center max-w-4xl mx-auto">
		<h1 class="text-4xl md:text-5xl lg:text-6xl font-extrabold tracking-tight mb-2">
			KSJ →
			<select
				bind:value={outputFormat}
				class="ml-2 inline-block align-middle border border-slate-700 rounded-md px-3 py-3"
				aria-label="出力形式を選択"
			>
				<option value="GeoParquet">GeoParquet</option>
				<option value="GeoJson">GeoJSON</option>
			</select>
		</h1>
		<p class="text-slate-700 text-base sm:text-lg">
			国土数値情報の Shapefile を、選択した形式に変換します。
		</p>
	</header>

	<div
		id="dropzone"
		class={`relative grid place-items-center ` +
			// Make the box more square-like: width drives height
			`w-1/3 aspect-square mx-auto ` +
			`p-16 ` +
			`border-4 border-dashed border-gray-400/50 ` +
			`outline-none transition ` +
			`${dragover ? 'border-sky-400/80 bg-sky-400/20' : ''}`}
		role="button"
		tabindex="0"
		aria-label="ファイルのドラッグ＆ドロップ領域"
		on:dragover={onDragOver}
		on:dragleave={onDragLeave}
		on:drop={onDrop}
		on:keydown={(e) => {
			if (e.key === 'Enter' || e.key === ' ') {
				pick();
			}
		}}
	>
		<div class="place-items-center">
			{#if bigLoading}
				<div class="animate-spin" aria-label="読み込み中"></div>
			{:else}
				<div class="" aria-label="読み込み中">
					<Icon icon="formkit:zip" width="6em" color="#bbb" />
				</div>
			{/if}
			<div class="text-gray-400 text-center font-extrabold leading-relaxed text-2xl py-8">
				{#if busy}
					変換中...
				{:else if !ready}
					読み込み中…
				{:else}
					ここに ZIP ファイルを<br />ドラッグ＆ドロップ<br />または
					<button
						class="text-blue-600 hover:underline"
						type="button"
						on:click|stopPropagation={pick}
						disabled={!ready || busy}
					>
						ZIP ファイルを選択
					</button>
				{/if}
				<input bind:this={inputEl} type="file" accept=".zip" hidden on:change={onInputChange} />
			</div>

			{#if busy && !bigLoading}
				<div class="absolute right-3 bottom-3 flex items-center gap-2 text-indigo-300/80 text-sm">
					<span class="w-[18px] h-[18px] border-2 border-white/25 animate-spin" aria-hidden="true"
					></span>
					<span class="sr-only">処理中</span>
				</div>
			{/if}

			{#if !busy && !ready && !bigLoading}
				<div class="absolute right-3 bottom-3 flex items-center gap-2 text-indigo-300/80 text-sm">
					<span
						class="w-[18px] h-[18px] border-2 border-white/25 border-t-sky-400 animate-spin"
						aria-hidden="true"
					></span>
					<span aria-hidden="true">初期化中…</span>
				</div>
			{/if}
		</div>
	</div>

	<!-- Bits UI: Error dialog -->
	<Dialog.Root bind:open={errorOpen}>
		<Dialog.Content class="fixed inset-0 grid place-items-center p-4 z-50">
			<div
				class="bg-slate-900 text-indigo-50 border border-slate-700 rounded-xl p-4 w-full max-w-lg shadow-2xl"
			>
				<Dialog.Title class="font-bold mb-1">エラーが発生しました</Dialog.Title>
				<div class="text-indigo-200/80 mb-3">{errorMessage}</div>
				<div class="flex justify-end">
					<Dialog.Close asChild>
						<button
							class="rounded-lg bg-gradient-to-b from-sky-400 to-blue-700 text-white px-4 py-2 font-semibold tracking-tight shadow-[0_6px_16px_rgba(64,149,255,0.35),inset_0_1px_0_rgba(255,255,255,0.35)]"
						>
							閉じる
						</button>
					</Dialog.Close>
				</div>
			</div>
		</Dialog.Content>
		<Dialog.Overlay class="fixed inset-0 bg-black/50 backdrop-blur-sm z-40" />
	</Dialog.Root>

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
					{#each shpOptions as opt}
						<button
							type="button"
							class="text-left rounded-lg w-full px-3 py-2 bg-slate-800/70 hover:bg-slate-800 border border-slate-700/70"
							on:click={() => chooseShp(opt, outputFormat)}
						>
							{opt}
						</button>
					{/each}
				</div>
				<div class="flex justify-end">
					<Dialog.Close asChild>
						<button
							type="button"
							class="rounded-lg bg-slate-700 text-white px-4 py-2 font-semibold tracking-tight"
							on:click={cancelShpDialog}
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
