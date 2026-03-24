<script lang="ts">
	import { auth } from '$lib/auth.svelte';
	import { decrypt } from '$lib/crypto';
	import pb from '$lib/pb';
	import type { SecretRecord } from '$lib/types';

	let { secret, onDeleted }: { secret: SecretRecord; onDeleted: () => void } = $props();

	let revealed = $state(false);
	let decryptedValue = $state('');
	let copied = $state(false);
	let loading = $state(false);
	let confirmDelete = $state(false);

	const typeBadge: Record<string, { label: string; color: string }> = {
		password: { label: 'Password', color: 'bg-blue-500/20 text-blue-400' },
		api_key: { label: 'API Key', color: 'bg-amber-500/20 text-amber-400' },
		env_var: { label: 'Env Var', color: 'bg-purple-500/20 text-purple-400' },
		ssh_key: { label: 'SSH Key', color: 'bg-cyan-500/20 text-cyan-400' },
		note: { label: 'Note', color: 'bg-zinc-500/20 text-zinc-400' }
	};

	const badge = $derived(typeBadge[secret.type] || typeBadge.note);

	async function reveal() {
		if (!auth.key) return;
		if (revealed) {
			revealed = false;
			decryptedValue = '';
			return;
		}

		loading = true;
		try {
			decryptedValue = await decrypt(auth.key, secret.encrypted_value, secret.iv);
			revealed = true;
		} catch {
			decryptedValue = 'Decryption failed';
			revealed = true;
		} finally {
			loading = false;
		}
	}

	async function copy() {
		if (!auth.key) return;

		try {
			const value = revealed
				? decryptedValue
				: await decrypt(auth.key, secret.encrypted_value, secret.iv);
			await navigator.clipboard.writeText(value);
			copied = true;
			setTimeout(() => (copied = false), 1500);
		} catch {
			/* ignore */
		}
	}

	async function handleDelete() {
		if (!confirmDelete) {
			confirmDelete = true;
			setTimeout(() => (confirmDelete = false), 3000);
			return;
		}
		await pb.collection('secrets').delete(secret.id);
		onDeleted();
	}
</script>

<div class="group rounded-xl border border-zinc-800 bg-zinc-900 p-4 transition hover:border-zinc-700">
	<div class="flex items-start justify-between gap-3">
		<div class="min-w-0 flex-1">
			<div class="flex items-center gap-2">
				<h3 class="truncate font-medium text-zinc-100">{secret.name}</h3>
				<span class="shrink-0 rounded-md px-2 py-0.5 text-xs font-medium {badge.color}">
					{badge.label}
				</span>
			</div>
			{#if secret.username}
				<p class="mt-0.5 truncate text-sm text-zinc-400">{secret.username}</p>
			{/if}
			{#if secret.url}
				<p class="mt-0.5 truncate text-sm text-zinc-500">{secret.url}</p>
			{/if}
		</div>

		<div class="flex shrink-0 items-center gap-1">
			<button
				onclick={reveal}
				class="rounded-lg p-2 text-zinc-500 transition hover:bg-zinc-800 hover:text-zinc-300"
				title={revealed ? 'Hide' : 'Reveal'}
			>
				{#if loading}
					<span class="text-xs">...</span>
				{:else if revealed}
					<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
						<path fill-rule="evenodd" d="M3.707 2.293a1 1 0 00-1.414 1.414l14 14a1 1 0 001.414-1.414l-1.473-1.473A10.014 10.014 0 0019.542 10C18.268 5.943 14.478 3 10 3a9.958 9.958 0 00-4.512 1.074l-1.78-1.781zm4.261 4.26l1.514 1.515a2.003 2.003 0 012.45 2.45l1.514 1.514a4 4 0 00-5.478-5.478z" clip-rule="evenodd" />
						<path d="M12.454 16.697L9.75 13.992a4 4 0 01-3.742-3.741L2.335 6.578A9.98 9.98 0 00.458 10c1.274 4.057 5.065 7 9.542 7 .847 0 1.669-.105 2.454-.303z" />
					</svg>
				{:else}
					<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
						<path d="M10 12a2 2 0 100-4 2 2 0 000 4z" />
						<path fill-rule="evenodd" d="M.458 10C1.732 5.943 5.522 3 10 3s8.268 2.943 9.542 7c-1.274 4.057-5.064 7-9.542 7S1.732 14.057.458 10zM14 10a4 4 0 11-8 0 4 4 0 018 0z" clip-rule="evenodd" />
					</svg>
				{/if}
			</button>

			<button
				onclick={copy}
				class="rounded-lg p-2 text-zinc-500 transition hover:bg-zinc-800 hover:text-zinc-300"
				title="Copy"
			>
				{#if copied}
					<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-emerald-400" viewBox="0 0 20 20" fill="currentColor">
						<path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
					</svg>
				{:else}
					<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
						<path d="M8 3a1 1 0 011-1h2a1 1 0 110 2H9a1 1 0 01-1-1z" />
						<path d="M6 3a2 2 0 00-2 2v11a2 2 0 002 2h8a2 2 0 002-2V5a2 2 0 00-2-2 3 3 0 01-3 3H9a3 3 0 01-3-3z" />
					</svg>
				{/if}
			</button>

			<button
				onclick={handleDelete}
				class="rounded-lg p-2 text-zinc-500 transition hover:bg-zinc-800 hover:text-red-400"
				title={confirmDelete ? 'Click again to confirm' : 'Delete'}
			>
				{#if confirmDelete}
					<span class="text-xs text-red-400">Sure?</span>
				{:else}
					<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
						<path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
					</svg>
				{/if}
			</button>
		</div>
	</div>

	{#if revealed}
		<div class="mt-3 rounded-lg bg-zinc-800 p-3">
			<pre class="overflow-x-auto whitespace-pre-wrap break-all font-mono text-sm text-zinc-300">{decryptedValue}</pre>
		</div>
	{/if}
</div>
