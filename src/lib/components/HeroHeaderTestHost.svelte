<script lang="ts">
	import HeroHeader from './HeroHeader.svelte';
	import type { OutputFormat } from '$lib/types';

	let selected: OutputFormat = 'GeoParquet';
</script>

<HeroHeader bind:value={selected} />
<div data-testid="selected-value">Selected: {selected}</div>
