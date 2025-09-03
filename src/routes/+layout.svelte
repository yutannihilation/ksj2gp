<script lang="ts">
    import favicon from '$lib/assets/favicon.ico';
    // Global styles (Tailwind entry and a couple custom utilities)
    import '../app.css';

    let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

{@render children?.()}
