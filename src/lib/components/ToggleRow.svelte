<script lang="ts">
	import { Label, Switch } from 'bits-ui';

	export let id: string;
	export let name: string = id;
	export let label: string | null = null;
	export let checked: boolean;
	export let disabled = false;
	export let containerClass = '';
	export let switchClass = '';
	export let thumbClass = '';
	export let labelClass = '';
</script>

<div class={`mx-auto grid w-full max-w-3xl grid-cols-[1fr_2fr] items-center gap-3 ${containerClass}`}>
	<Switch.Root
		bind:checked
		{disabled}
		{id}
		{name}
		class={`peer inline-flex h-[36px] min-h-[36px] w-[64px] shrink-0 cursor-pointer items-center justify-self-end rounded-full bg-slate-300 px-[4px] transition-colors focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-foreground focus-visible:ring-offset-2 focus-visible:ring-offset-background data-[state=checked]:bg-slate-700 disabled:cursor-not-allowed ${switchClass}`}
	>
		<Switch.Thumb
			class={`pointer-events-none block size-[28px] shrink-0 rounded-full bg-white shadow transition-transform data-[state=checked]:translate-x-[28px] data-[state=unchecked]:translate-x-0 ${thumbClass}`}
		/>
	</Switch.Root>
	<Label.Root for={id} class={`font-bold text-2xl text-slate-700 ${labelClass}`}>
		<slot>{label}</slot>
	</Label.Root>
</div>
