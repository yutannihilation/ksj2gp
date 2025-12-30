<script lang="ts">
	import ErrorDialog from './ErrorDialog.svelte';

	let open = true;
</script>

<ErrorDialog bind:open message="Something went wrong." />
<div data-testid="open-state">Open: {open ? 'true' : 'false'}</div>
