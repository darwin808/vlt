<script lang="ts">
	import '../app.css';

	let { children } = $props();
</script>

<svelte:head>
	<title>VLT</title>
	<meta name="description" content="Your secrets, your device." />
</svelte:head>

{@render children()}
