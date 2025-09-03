<script lang="ts">
	import { onMount } from 'svelte';

	let inputEl: HTMLInputElement;
	let dragover = false;
	let busy = false;
	let worker: Worker | null = null;

	onMount(() => {
		worker = new Worker(new URL('$lib/worker.ts', import.meta.url), { type: 'module' });

		worker.onmessage = async (event: MessageEvent) => {
			const data: any = event.data;

			const finish = () => {
				busy = false;
			};

			if (data && typeof data === 'object' && 'error' in data) {
				alert(`エラー: ${data.error}`);
				finish();
				return;
			}

			const fileHandle: any = data?.handle ?? data;
			if (!fileHandle || typeof fileHandle.getFile !== 'function') {
				alert('予期しない応答を受け取りました');
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
		inputEl?.click();
	}

	function processFile(file: File | undefined | null) {
		if (!file || !worker) return;
		busy = true;
		worker.postMessage({ file });
	}

	function onInputChange(e: Event) {
		const file = (e.currentTarget as HTMLInputElement).files?.[0];
		processFile(file ?? null);
	}
</script>

<div class="container">
	<header>
		<h1>KSJ → GeoParquet</h1>
		<p>国土数値情報の ZIP をドラッグ＆ドロップすると、GeoParquet ファイルに変換します。</p>
	</header>

	<div class="card">
		<div
			id="dropzone"
			class="dropzone {dragover ? 'dragover' : ''}"
			role="button"
			tabindex="0"
			aria-label="ファイルのドラッグ＆ドロップ領域"
		>
			<div class="dz-icon">📦</div>
			<div class="dz-text"><strong>ここに ZIP をドロップ</strong>、または下のボタンをクリック</div>

			<div class="row">
				<button type="button" class="btn" on:click|stopPropagation={pick} disabled={busy}
					>ZIPを選択</button
				>
				<input bind:this={inputEl} type="file" accept=".zip" hidden on:change={onInputChange} />
			</div>

			{#if busy}
				<div class="status">変換中…</div>
			{/if}
		</div>
	</div>
</div>
