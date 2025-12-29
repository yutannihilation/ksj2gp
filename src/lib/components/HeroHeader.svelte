<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Select } from 'bits-ui';
	import type { OutputFormat } from '$lib/types';

	export let value: OutputFormat;

	const formats: OutputFormat[] = ['GeoParquet', 'GeoJson'];

	function handleValueChange(next: string) {
		value = next as OutputFormat;
	}
</script>

<header class="text-center max-w-4xl mx-auto">
	<h1 class="text-7xl font-extrabold tracking-wider font-title mb-4">
		KSJ
		<Icon icon="mdi:arrow-right" class="inline" height="0.7em" />
		<Select.Root {value} onValueChange={handleValueChange} type="single">
			<Select.Trigger
				class="inline-block tracking-tight align-baseline min-w-[7.2em] border border-slate-700 py-3 relative"
				aria-label="出力形式を選択"
			>
				{value}
				<Icon icon="mdi:caret-down" class="absolute top-1/3 right-0 h-1/3" width="0.5em" />
			</Select.Trigger>
			<Select.Portal>
				<Select.Content
					class="focus-override text-center border-muted bg-white shadow-popover data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 outline-hidden z-50 max-h-[var(--bits-select-content-available-height)] w-[var(--bits-select-anchor-width)] min-w-[var(--bits-select-anchor-width)] select-none border data-[side=bottom]:translate-y-1 data-[side=left]:-translate-x-1 data-[side=right]:translate-x-1 data-[side=top]:-translate-y-1"
				>
					<Select.Viewport>
						{#each formats as option (option)}
							<Select.Item
								value={option}
								class="text-7xl font-extrabold font-title tracking-tight data-highlighted:bg-gray-300 pb-2"
								label={option}
							>
								{option}
							</Select.Item>
						{/each}
					</Select.Viewport>
				</Select.Content>
			</Select.Portal>
		</Select.Root>
	</h1>
	<p class="text-slate-700 text-base sm:text-lg">
		国土数値情報の Shapefile を、選択した形式に変換します。
	</p>
</header>
