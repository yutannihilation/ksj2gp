<script lang="ts">
	import ToggleRow from './ToggleRow.svelte';

	let checked = false;
</script>

<ToggleRow id="toggle-row-test" label="Enable feature" bind:checked />
<div data-testid="checked-value">Checked: {checked ? 'true' : 'false'}</div>
