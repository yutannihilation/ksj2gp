<script lang="ts">
	import { Dialog } from 'bits-ui';
	import { status } from '$lib/stores/status.svelte';

	let {
		open = $bindable(false),
		shpFiles = [],
		onSelect
	}: {
		open?: boolean;
		shpFiles: string[];
		onSelect: (path: string) => void;
	} = $props();

	function cancelShpDialog() {
		open = false;
		status.busy = false;
	}
</script>

<!-- Bits UI: Shapefile selection dialog -->
<Dialog.Root bind:open>
	<Dialog.Content class="fixed inset-0 grid place-items-center p-4 z-50">
		<div
			class="bg-slate-900 text-indigo-50 border border-slate-700 rounded-xl p-4 w-full max-w-xl shadow-2xl"
		>
			<Dialog.Title class="font-bold mb-1">Shapefile を選択</Dialog.Title>
			<div class="text-indigo-200/80 mb-4">
				ZIP には複数の .shp が含まれています。変換するファイルを選択してください。
			</div>
			<div class="max-h-72 overflow-auto grid gap-2 mb-4">
				{#each shpFiles as shp (shp)}
					<button
						type="button"
						class="text-left rounded-lg w-full px-3 py-2 bg-slate-800/70 hover:bg-slate-800 border border-slate-700/70"
						onclick={() => onSelect(shp)}
					>
						{shp}
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
