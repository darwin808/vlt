<script lang="ts">
	import { goto } from '$app/navigation';
	import { auth } from '$lib/auth.svelte';

	let email = $state('');
	let password = $state('');
	let error = $state('');
	let loading = $state(false);

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = '';
		loading = true;

		try {
			await auth.login(email, password);
			goto('/');
		} catch (err: any) {
			error = err?.message || 'Authentication failed';
		} finally {
			loading = false;
		}
	}
</script>

<div class="flex min-h-screen items-center justify-center bg-zinc-950 px-4">
	<div class="w-full max-w-xs">
		<div class="mb-6 text-center">
			<h1 class="text-xl font-bold tracking-tight text-zinc-300">VLT</h1>
			<p class="mt-0.5 text-xs text-zinc-600">developer secrets vault</p>
		</div>

		<form onsubmit={handleSubmit} class="space-y-3">
			<input
				id="email"
				type="email"
				bind:value={email}
				required
				class="w-full rounded border border-zinc-800 bg-zinc-900/50 px-3 py-2 text-sm text-zinc-200 placeholder-zinc-600 outline-none focus:border-zinc-600"
				placeholder="Email"
			/>
			<input
				id="password"
				type="password"
				bind:value={password}
				required
				class="w-full rounded border border-zinc-800 bg-zinc-900/50 px-3 py-2 text-sm text-zinc-200 placeholder-zinc-600 outline-none focus:border-zinc-600"
				placeholder="Master password"
			/>

			{#if error}
				<p class="text-xs text-red-400">{error}</p>
			{/if}

			<button
				type="submit"
				disabled={loading}
				class="w-full rounded bg-zinc-800 px-4 py-2 text-sm font-medium text-zinc-200 transition hover:bg-zinc-700 disabled:opacity-50"
			>
				{loading ? '...' : 'Unlock'}
			</button>
		</form>

		<p class="mt-3 text-center text-xs text-zinc-600">
			No account? <a href="/signup" class="text-zinc-400 hover:text-zinc-200">Create one</a>
		</p>
	</div>
</div>
