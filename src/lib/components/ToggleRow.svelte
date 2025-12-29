<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Label, Switch } from 'bits-ui';

	let {
		id,
		label = null,
		checked = $bindable(false),
		children
	} = $props<{
		id: string;
		label?: string | null;
		checked?: boolean;
		children?: Snippet;
	}>();
</script>

<div class="mx-auto grid w-full max-w-3xl grid-cols-[1fr_2fr] items-center gap-3">
	<Switch.Root
		bind:checked
		{id}
		class="peer inline-flex h-9 min-h-9 w-16 shrink-0 cursor-pointer items-center justify-self-end rounded-full bg-slate-300 px-1 transition-colors focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-foreground focus-visible:ring-offset-2 focus-visible:ring-offset-background data-[state=checked]:bg-slate-700 disabled:cursor-not-allowed disabled:opacity-20"
	>
		<Switch.Thumb
			class="pointer-events-none block size-7 shrink-0 rounded-full bg-white shadow transition-transform data-[state=checked]:translate-x-7 data-[state=unchecked]:translate-x-0"
		/>
	</Switch.Root>
	<Label.Root for={id} class="font-bold text-2xl text-slate-700">
		{#if children}
			{@render children()}
		{:else}
			{label}
		{/if}
	</Label.Root>
</div>
