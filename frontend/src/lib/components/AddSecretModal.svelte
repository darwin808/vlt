<script lang="ts">
	import { auth } from '$lib/auth.svelte';
	import { encrypt } from '$lib/crypto';
	import pb from '$lib/pb';
	import type { SecretType } from '$lib/types';

	let { open = $bindable(false), onSaved }: { open: boolean; onSaved: () => void } = $props();

	let name = $state('');
	let type: SecretType = $state('password');
	let value = $state('');
	let username = $state('');
	let url = $state('');
	let error = $state('');
	let loading = $state(false);

	const types: { value: SecretType; label: string }[] = [
		{ value: 'password', label: 'Password' },
		{ value: 'api_key', label: 'API Key' },
		{ value: 'env_var', label: 'Env Var' },
		{ value: 'ssh_key', label: 'SSH Key' },
		{ value: 'note', label: 'Note' }
	];

	function reset() {
		name = '';
		type = 'password';
		value = '';
		username = '';
		url = '';
		error = '';
	}

	function close() {
		open = false;
		reset();
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();
		if (!auth.key || !auth.user) return;

		error = '';
		loading = true;

		try {
			const { ciphertext, iv } = await encrypt(auth.key, value);

			await pb.collection('secrets').create({
				name,
				type,
				encrypted_value: ciphertext,
				iv,
				username: username || '',
				url: url || '',
				user: auth.user.id
			});

			close();
			onSaved();
		} catch (err: any) {
			error = err?.message || 'Failed to save secret';
		} finally {
			loading = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') close();
	}
</script>

{#if open}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 px-4"
		onkeydown={handleKeydown}
	>
		<!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
		<div class="fixed inset-0" onclick={close}></div>

		<div class="relative w-full max-w-md rounded-xl border border-zinc-800 bg-zinc-900 p-6">
			<h2 class="mb-4 text-lg font-semibold text-zinc-100">Add Secret</h2>

			<form onsubmit={handleSubmit} class="space-y-4">
				<div>
					<label for="s-name" class="mb-1 block text-sm text-zinc-400">Name</label>
					<input
						id="s-name"
						type="text"
						bind:value={name}
						required
						class="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-zinc-100 outline-none focus:border-emerald-500"
						placeholder="GitHub API Key"
					/>
				</div>

				<div>
					<label for="s-username" class="mb-1 block text-sm text-zinc-400">Username (optional)</label>
					<input
						id="s-username"
						type="text"
						bind:value={username}
						class="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-zinc-100 outline-none focus:border-emerald-500"
						placeholder="user@example.com"
					/>
				</div>

				<div>
					<label for="s-type" class="mb-1 block text-sm text-zinc-400">Type</label>
					<select
						id="s-type"
						bind:value={type}
						class="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-zinc-100 outline-none focus:border-emerald-500"
					>
						{#each types as t}
							<option value={t.value}>{t.label}</option>
						{/each}
					</select>
				</div>

				<div>
					<label for="s-value" class="mb-1 block text-sm text-zinc-400">Value</label>
					{#if type === 'ssh_key' || type === 'note'}
						<textarea
							id="s-value"
							bind:value={value}
							required
							rows="4"
							class="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 font-mono text-sm text-zinc-100 outline-none focus:border-emerald-500"
							placeholder={type === 'ssh_key' ? '-----BEGIN OPENSSH PRIVATE KEY-----' : 'Your note...'}
						></textarea>
					{:else}
						<input
							id="s-value"
							type="password"
							bind:value={value}
							required
							class="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 font-mono text-zinc-100 outline-none focus:border-emerald-500"
							placeholder="••••••••••••"
						/>
					{/if}
				</div>

				<div>
					<label for="s-url" class="mb-1 block text-sm text-zinc-400">URL (optional)</label>
					<input
						id="s-url"
						type="url"
						bind:value={url}
						class="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-zinc-100 outline-none focus:border-emerald-500"
						placeholder="https://github.com"
					/>
				</div>

				{#if error}
					<p class="text-sm text-red-400">{error}</p>
				{/if}

				<div class="flex justify-end gap-3 pt-2">
					<button
						type="button"
						onclick={close}
						class="rounded-lg px-4 py-2 text-sm text-zinc-400 transition hover:text-zinc-200"
					>
						Cancel
					</button>
					<button
						type="submit"
						disabled={loading}
						class="rounded-lg bg-emerald-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-emerald-500 disabled:opacity-50"
					>
						{loading ? 'Saving...' : 'Save'}
					</button>
				</div>
			</form>
		</div>
	</div>
{/if}
