<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { Attachment } from 'svelte/attachments';
	import { status } from '$lib/stores/status';

	let {
		onFile,
		onError
	}: {
		onFile: (file: File) => void;
		onError: (message: string) => void;
	} = $props();

	const showLoading = $derived(!$status.ready || $status.busy);

	let dragover = $state(false);
	let inputEl: HTMLInputElement | null = null;

	const attachInput: Attachment<HTMLInputElement> = (node) => {
		inputEl = node;
		return () => {
			if (inputEl === node) inputEl = null;
		};
	};

	function pick() {
		if (!$status.ready || $status.busy) return;
		inputEl?.click();
	}

	function emitFile(file: File | undefined | null) {
		if (!file) return;
		if (!file.name.toLowerCase().endsWith('.zip')) {
			onError('ZIP ファイルを選択してください');
			return;
		}
		onFile(file);
	}

	function onInputChange(e: Event) {
		const file = (e.currentTarget as HTMLInputElement).files?.[0];
		emitFile(file ?? null);
	}

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
		emitFile(e.dataTransfer?.files?.[0] ?? null);
	}
</script>

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
		{#if showLoading}
			<div class="animate-spin" aria-label="読み込み中"></div>
		{:else}
			<div class="" aria-label="読み込み中">
				<Icon icon="formkit:zip" width="6em" color="#bbb" />
			</div>
		{/if}
		<div class="text-gray-400 text-center font-bold leading-relaxed text-2xl py-8">
			{#if $status.busy}
				変換中...
			{:else if !$status.ready}
				読み込み中…
			{:else}
				ここに ZIP ファイルを<br />ドラッグ＆ドロップ<br />または
				<button
					class="text-blue-600 hover:underline"
					type="button"
					onclick={pick}
					disabled={!$status.ready || $status.busy}
				>
					ZIP ファイルを選択
				</button>
			{/if}
			<input {@attach attachInput} type="file" accept=".zip" hidden onchange={onInputChange} />
		</div>

		{#if $status.busy && !showLoading}
			<div class="absolute right-3 bottom-3 flex items-center gap-2 text-indigo-300/80 text-sm">
				<span class="w-4.5 h-4.5 border-2 border-white/25 animate-spin" aria-hidden="true"></span>
				<span class="sr-only">処理中</span>
			</div>
		{/if}

		{#if !$status.busy && !$status.ready && !showLoading}
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
