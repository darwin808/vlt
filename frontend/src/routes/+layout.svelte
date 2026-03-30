<script lang="ts">
	import '../app.css';
	import { page } from '$app/state';
	import { auth } from '$lib/auth.svelte';

	let { children } = $props();

	const isAuthPage = $derived(
		page.url.pathname === '/login' || page.url.pathname === '/signup'
	);
	const showContent = $derived(auth.isReady || isAuthPage);
</script>

<svelte:head>
	<title>VLT</title>
	<meta name="description" content="Your secrets, your device." />
</svelte:head>

{#if showContent}
	{@render children()}
{:else}
	<div class="flex min-h-screen items-center justify-center bg-zinc-950">
		<span class="text-sm text-zinc-600">Loading...</span>
	</div>
{/if}
