<script lang="ts">
	import { onMount } from 'svelte';
	// Use Bits UI for a nicer error dialog
	// Note: ensure `bits-ui` is installed locally
	import { Dialog } from 'bits-ui';

	let inputEl: HTMLInputElement;
	let dragover = false;
	let busy = false;
	let worker: Worker | null = null;
	let ready = false;

	// Error dialog state
	let errorOpen = false;
	let errorMessage = '';
	$: bigLoading = !ready || busy;

	onMount(() => {
		worker = new Worker(new URL('$lib/worker.ts', import.meta.url), { type: 'module' });

		worker.onmessage = async (event: MessageEvent) => {
			const data: any = event.data;

			const finish = () => {
				busy = false;
			};

			if (data && typeof data === 'object' && 'ready' in data && data.ready) {
				ready = true;
				return;
			}

			if (data && typeof data === 'object' && 'error' in data) {
				showError(String(data.error ?? '変換に失敗しました'));
				finish();
				return;
			}

			const fileHandle: any = data?.handle ?? data;
			if (!fileHandle || typeof fileHandle.getFile !== 'function') {
				showError('予期しない応答を受け取りました');
				finish();
				return;
			}

			const file = await fileHandle.getFile();
			const url = URL.createObjectURL(file);

			const a = document.createElement('a');
			a.href = url;
			a.download = file.name || 'tmp.parquet';
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
		worker.postMessage({ file });
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
</script>

<div
	class="min-h-dvh bg-hero text-indigo-50 flex flex-col gap-8 sm:gap-10 lg:gap-12 py-10 sm:py-12 lg:py-16 px-5 sm:px-6 lg:justify-center"
>
	<header class="text-center max-w-4xl mx-auto">
		<h1 class="text-4xl md:text-5xl lg:text-6xl font-extrabold tracking-tight mb-2">
			KSJ → GeoParquet
		</h1>
		<p class="text-indigo-200/80 text-base sm:text-lg">
			国土数値情報の ZIP をドラッグ＆ドロップすると、GeoParquet に変換します。
		</p>
	</header>

	<section
		class="max-w-7xl mx-auto glass-panel border border-slate-700/60 rounded-2xl p-10 sm:p-12 lg:p-14 shadow-[0_10px_30px_rgba(0,0,0,0.25),inset_0_1px_0_rgba(255,255,255,0.06)]"
	>
		<div
			id="dropzone"
			class={`relative grid place-items-center gap-5 ` +
				// Make the box more square-like: width drives height
				`w-full max-w-[56rem] sm:max-w-[64rem] lg:max-w-[70rem] aspect-square mx-auto ` +
				`p-12 sm:p-14 lg:p-16 ` +
				`border-2 border-dashed border-blue-800/70 rounded-xl ` +
				`bg-slate-900/60 outline-none transition ` +
				`${dragover ? 'ring-4 ring-sky-400/35 border-sky-400/80 -translate-y-0.5' : ''}`}
			role="button"
			tabindex="0"
			aria-label="ファイルのドラッグ＆ドロップ領域"
			on:dragover={onDragOver}
			on:dragleave={onDragLeave}
			on:drop={onDrop}
			on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && pick()}
		>
			{#if bigLoading}
				<div
					class="w-28 h-28 sm:w-32 sm:h-32 lg:w-36 lg:h-36 border-4 border-white/25 border-t-sky-400 rounded-full animate-spin"
					aria-label="読み込み中"
				/>
			{:else}
				<div class="text-8xl lg:text-9xl" aria-hidden="true">📦</div>
			{/if}
			<div class="text-indigo-200/80 text-center leading-relaxed text-xl sm:text-2xl lg:text-3xl">
				<strong>ここに ZIP をドロップ</strong><br />または下のボタンから選択
			</div>

			<div class="flex gap-2.5">
				<button
					type="button"
					class="rounded-lg bg-gradient-to-b from-sky-400 to-blue-700 text-white px-6 py-3.5 text-lg sm:text-xl font-semibold tracking-tight shadow-[0_6px_16px_rgba(64,149,255,0.35),inset_0_1px_0_rgba(255,255,255,0.35)] transition active:brightness-95 hover:brightness-105 disabled:opacity-60 disabled:cursor-not-allowed"
					on:click|stopPropagation={pick}
					disabled={!ready || busy}
				>
					{busy ? '変換中…' : !ready ? '読み込み中…' : 'ZIP を選択'}
				</button>
				<input bind:this={inputEl} type="file" accept=".zip" hidden on:change={onInputChange} />
			</div>

			{#if busy && !bigLoading}
				<div class="absolute right-3 bottom-3 flex items-center gap-2 text-indigo-300/80 text-sm">
					<span
						class="w-[18px] h-[18px] border-2 border-white/25 border-t-teal-400 rounded-full animate-spin"
						aria-hidden="true"
					></span>
					<span class="sr-only">処理中</span>
				</div>
			{/if}

			{#if !busy && !ready && !bigLoading}
				<div class="absolute right-3 bottom-3 flex items-center gap-2 text-indigo-300/80 text-sm">
					<span
						class="w-[18px] h-[18px] border-2 border-white/25 border-t-sky-400 rounded-full animate-spin"
						aria-hidden="true"
					></span>
					<span aria-hidden="true">初期化中…</span>
				</div>
			{/if}
		</div>
	</section>

	<!-- Bits UI: Error dialog -->
	<Dialog.Root bind:open={errorOpen}>
		<Dialog.Content class="fixed inset-0 grid place-items-center p-4">
			<div
				class="bg-slate-900 text-indigo-50 border border-slate-700 rounded-xl p-4 w-full max-w-lg shadow-2xl"
			>
				<Dialog.Title class="font-bold mb-1">エラーが発生しました</Dialog.Title>
				<div class="text-indigo-200/80 mb-3">{errorMessage}</div>
				<div class="flex justify-end">
					<Dialog.Close asChild>
						<button
							class="rounded-lg bg-gradient-to-b from-sky-400 to-blue-700 text-white px-4 py-2 font-semibold tracking-tight shadow-[0_6px_16px_rgba(64,149,255,0.35),inset_0_1px_0_rgba(255,255,255,0.35)]"
							>閉じる</button
						>
					</Dialog.Close>
				</div>
			</div>
		</Dialog.Content>
		<Dialog.Overlay class="fixed inset-0 bg-black/50 backdrop-blur-sm" />
	</Dialog.Root>
</div>
