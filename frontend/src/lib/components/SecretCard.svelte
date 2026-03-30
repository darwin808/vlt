<script lang="ts">
	import { auth } from '$lib/auth.svelte';
	import { decrypt } from '$lib/crypto';
	import { api } from '$lib/api';
	import type { SecretRecord } from '$lib/types';

	let {
		secret,
		selected = false,
		onDeleted
	}: { secret: SecretRecord; selected: boolean; onDeleted: () => void } = $props();

	let revealed = $state(false);
	let decryptedValue = $state('');
	let copied = $state(false);
	let confirmDelete = $state(false);

	const typeIcon: Record<string, string> = {
		password: '●',
		api_key: '◆',
		env_var: '▪',
		ssh_key: '⌂',
		note: '¶'
	};

	export async function copyPassword() {
		if (!auth.key) return;
		try {
			const value = revealed
				? decryptedValue
				: await decrypt(auth.key, secret.encrypted_value, secret.iv);
			await navigator.clipboard.writeText(value);
			copied = true;
			setTimeout(() => (copied = false), 1200);
		} catch {
			/* ignore */
		}
	}

	export async function copyUsername() {
		if (!secret.username) return;
		await navigator.clipboard.writeText(secret.username);
		copied = true;
		setTimeout(() => (copied = false), 1200);
	}

	export async function toggleReveal() {
		if (!auth.key) return;
		if (revealed) {
			revealed = false;
			decryptedValue = '';
			return;
		}
		try {
			decryptedValue = await decrypt(auth.key, secret.encrypted_value, secret.iv);
			revealed = true;
		} catch {
			decryptedValue = 'Decryption failed';
			revealed = true;
		}
	}

	async function handleDelete(e: Event) {
		e.stopPropagation();
		if (!confirmDelete) {
			confirmDelete = true;
			setTimeout(() => (confirmDelete = false), 3000);
			return;
		}
		await api.secrets.delete(secret.id);
		onDeleted();
	}
</script>

<div>
	<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
	<div
		class="flex items-center gap-3 px-3 py-2 transition-colors {selected
			? 'bg-zinc-800/60 border-l-2 border-emerald-500'
			: 'border-l-2 border-transparent hover:bg-zinc-900/80'}"
		onclick={copyPassword}
	>
		<!-- Type icon -->
		<span class="w-4 shrink-0 text-center text-xs text-zinc-600" title={secret.type}>
			{typeIcon[secret.type] || '●'}
		</span>

		<!-- Name -->
		<span class="w-48 shrink-0 truncate text-sm text-zinc-200">{secret.name}</span>

		<!-- Username -->
		<span class="min-w-0 flex-1 truncate text-sm text-zinc-500">
			{secret.username || '—'}
		</span>

		<!-- Actions -->
		<div class="flex shrink-0 items-center gap-0.5">
			{#if copied}
				<span class="px-2 text-xs text-emerald-400">Copied</span>
			{:else}
				<button
					onclick={(e) => { e.stopPropagation(); toggleReveal(); }}
					class="rounded p-1.5 text-zinc-600 transition hover:text-zinc-300"
					title={revealed ? 'Hide' : 'Reveal (Space)'}
				>
					{#if revealed}
						<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
							<path fill-rule="evenodd" d="M3.707 2.293a1 1 0 00-1.414 1.414l14 14a1 1 0 001.414-1.414l-1.473-1.473A10.014 10.014 0 0019.542 10C18.268 5.943 14.478 3 10 3a9.958 9.958 0 00-4.512 1.074l-1.78-1.781zm4.261 4.26l1.514 1.515a2.003 2.003 0 012.45 2.45l1.514 1.514a4 4 0 00-5.478-5.478z" clip-rule="evenodd" />
							<path d="M12.454 16.697L9.75 13.992a4 4 0 01-3.742-3.741L2.335 6.578A9.98 9.98 0 00.458 10c1.274 4.057 5.065 7 9.542 7 .847 0 1.669-.105 2.454-.303z" />
						</svg>
					{:else}
						<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
							<path d="M10 12a2 2 0 100-4 2 2 0 000 4z" />
							<path fill-rule="evenodd" d="M.458 10C1.732 5.943 5.522 3 10 3s8.268 2.943 9.542 7c-1.274 4.057-5.064 7-9.542 7S1.732 14.057.458 10zM14 10a4 4 0 11-8 0 4 4 0 018 0z" clip-rule="evenodd" />
						</svg>
					{/if}
				</button>

				<button
					onclick={handleDelete}
					class="rounded p-1.5 text-zinc-600 transition hover:text-red-400"
					title={confirmDelete ? 'Click again' : 'Delete'}
				>
					{#if confirmDelete}
						<span class="text-[10px] text-red-400">Sure?</span>
					{:else}
						<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
							<path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
						</svg>
					{/if}
				</button>
			{/if}
		</div>
	</div>

	{#if revealed}
		<div class="ml-7 border-l-2 border-zinc-800 bg-zinc-900/50 px-4 py-2">
			<pre class="font-mono text-xs text-zinc-400">{decryptedValue}</pre>
		</div>
	{/if}
</div>
