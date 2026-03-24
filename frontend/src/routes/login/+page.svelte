<script lang="ts">
	import { goto } from '$app/navigation';
	import { auth } from '$lib/auth.svelte';

	let email = $state('');
	let password = $state('');
	let error = $state('');
	let loading = $state(false);

	const isLocked = $derived(auth.isAuthenticated && !auth.isUnlocked);

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = '';
		loading = true;

		try {
			if (isLocked) {
				await auth.unlock(password);
			} else {
				await auth.login(email, password);
			}
			goto('/');
		} catch (err: any) {
			error = err?.message || 'Authentication failed';
		} finally {
			loading = false;
		}
	}
</script>

<div class="flex min-h-screen items-center justify-center bg-zinc-950 px-4">
	<div class="w-full max-w-sm">
		<div class="mb-8 text-center">
			<h1 class="text-3xl font-bold tracking-tight text-zinc-100">VLT</h1>
			<p class="mt-1 text-sm text-zinc-500">
				{isLocked ? 'Enter master password to unlock' : 'Sign in to your vault'}
			</p>
		</div>

		<form onsubmit={handleSubmit} class="space-y-4">
			{#if !isLocked}
				<div>
					<label for="email" class="mb-1 block text-sm text-zinc-400">Email</label>
					<input
						id="email"
						type="email"
						bind:value={email}
						required
						class="w-full rounded-lg border border-zinc-800 bg-zinc-900 px-3 py-2 text-zinc-100 placeholder-zinc-600 outline-none focus:border-emerald-500"
						placeholder="you@example.com"
					/>
				</div>
			{/if}

			<div>
				<label for="password" class="mb-1 block text-sm text-zinc-400">Master Password</label>
				<input
					id="password"
					type="password"
					bind:value={password}
					required
					class="w-full rounded-lg border border-zinc-800 bg-zinc-900 px-3 py-2 text-zinc-100 placeholder-zinc-600 outline-none focus:border-emerald-500"
					placeholder="••••••••••••"
				/>
			</div>

			{#if error}
				<p class="text-sm text-red-400">{error}</p>
			{/if}

			<button
				type="submit"
				disabled={loading}
				class="w-full rounded-lg bg-emerald-600 px-4 py-2 font-medium text-white transition hover:bg-emerald-500 disabled:opacity-50"
			>
				{loading ? '...' : isLocked ? 'Unlock' : 'Sign In'}
			</button>
		</form>

		{#if !isLocked}
			<p class="mt-4 text-center text-sm text-zinc-500">
				No account? <a href="/signup" class="text-emerald-400 hover:underline">Create one</a>
			</p>
		{/if}
	</div>
</div>
