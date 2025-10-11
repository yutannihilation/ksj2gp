<script lang="ts">
	import { Dialog } from 'bits-ui';

	export let open = false;
	export let message = '';
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="fixed inset-0 grid place-items-center p-4 z-50">
		<div class="bg-slate-900 text-indigo-50 border border-slate-700 rounded-xl p-4 w-full max-w-lg shadow-2xl">
			<Dialog.Title class="font-bold mb-1">エラーが発生しました</Dialog.Title>
			<div class="text-indigo-200/80 mb-3">{message}</div>
			<div class="flex justify-end">
				<Dialog.Close asChild>
					<button
						class="rounded-lg bg-gradient-to-b from-sky-400 to-blue-700 text-white px-4 py-2 font-bold tracking-tight shadow-[0_6px_16px_rgba(64,149,255,0.35),inset_0_1px_0_rgba(255,255,255,0.35)]"
					>
						閉じる
					</button>
				</Dialog.Close>
			</div>
		</div>
	</Dialog.Content>
	<Dialog.Overlay class="fixed inset-0 bg-black/50 backdrop-blur-sm z-40" />
</Dialog.Root>
