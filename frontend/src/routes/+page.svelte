<script lang="ts">
	import { goto } from '$app/navigation';
	import { auth } from '$lib/auth.svelte';
	import pb from '$lib/pb';
	import type { SecretRecord } from '$lib/types';
	import SecretCard from '$lib/components/SecretCard.svelte';
	import AddSecretModal from '$lib/components/AddSecretModal.svelte';
	import ImportModal from '$lib/components/ImportModal.svelte';

	let secrets = $state<SecretRecord[]>([]);
	let search = $state('');
	let showAdd = $state(false);
	let showImport = $state(false);
	let loading = $state(true);

	const filtered = $derived(
		search
			? secrets.filter(
					(s) =>
						s.name.toLowerCase().includes(search.toLowerCase()) ||
						s.type.toLowerCase().includes(search.toLowerCase()) ||
						(s.username && s.username.toLowerCase().includes(search.toLowerCase())) ||
						(s.url && s.url.toLowerCase().includes(search.toLowerCase()))
				)
			: secrets
	);

	async function loadSecrets() {
		if (!auth.isUnlocked) {
			goto('/login');
			return;
		}

		loading = true;
		try {
			const result = await pb.collection('secrets').getFullList<SecretRecord>({
				sort: '-created',
				filter: `user = "${auth.user?.id}"`
			});
			secrets = result;
		} catch {
			secrets = [];
		} finally {
			loading = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if ((e.metaKey || e.ctrlKey) && e.key === 'n') {
			e.preventDefault();
			showAdd = true;
		}
		if (e.key === '/' && !showAdd && !showImport) {
			e.preventDefault();
			document.getElementById('search')?.focus();
		}
	}

	function handleLogout() {
		auth.logout();
		goto('/login');
	}

	$effect(() => {
		if (auth.isUnlocked) {
			loadSecrets();
		} else {
			goto('/login');
		}
	});
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="min-h-screen bg-zinc-950">
	<!-- Header -->
	<header class="border-b border-zinc-800 bg-zinc-950/80 backdrop-blur-sm">
		<div class="mx-auto flex max-w-3xl items-center justify-between px-4 py-3">
			<h1 class="text-lg font-bold tracking-tight text-zinc-100">VLT</h1>
			<div class="flex items-center gap-2">
				<button
					onclick={() => (showImport = true)}
					class="rounded-lg px-3 py-1.5 text-sm text-zinc-500 transition hover:text-zinc-300"
				>
					Import
				</button>
				<button
					onclick={() => (showAdd = true)}
					class="rounded-lg bg-emerald-600 px-3 py-1.5 text-sm font-medium text-white transition hover:bg-emerald-500"
				>
					+ New <span class="ml-1 text-xs text-emerald-200/60">⌘N</span>
				</button>
				<button
					onclick={handleLogout}
					class="rounded-lg px-3 py-1.5 text-sm text-zinc-500 transition hover:text-red-400"
				>
					Logout
				</button>
			</div>
		</div>
	</header>

	<!-- Search -->
	<div class="mx-auto max-w-3xl px-4 pt-4">
		<div class="relative">
			<svg
				xmlns="http://www.w3.org/2000/svg"
				class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-zinc-500"
				viewBox="0 0 20 20"
				fill="currentColor"
			>
				<path
					fill-rule="evenodd"
					d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z"
					clip-rule="evenodd"
				/>
			</svg>
			<input
				id="search"
				type="text"
				bind:value={search}
				class="w-full rounded-xl border border-zinc-800 bg-zinc-900 py-2.5 pl-10 pr-4 text-zinc-100 placeholder-zinc-600 outline-none focus:border-zinc-700"
				placeholder="Search secrets...  (/)"
			/>
		</div>
	</div>

	<!-- Secrets list -->
	<div class="mx-auto max-w-3xl px-4 py-4">
		{#if loading}
			<div class="py-20 text-center text-zinc-500">Loading...</div>
		{:else if filtered.length === 0}
			<div class="py-20 text-center">
				{#if search}
					<p class="text-zinc-500">No secrets match "{search}"</p>
				{:else}
					<p class="text-zinc-500">No secrets yet</p>
					<button
						onclick={() => (showAdd = true)}
						class="mt-3 text-sm text-emerald-400 hover:underline"
					>
						Add your first secret
					</button>
				{/if}
			</div>
		{:else}
			<div class="space-y-2">
				{#each filtered as secret (secret.id)}
					<SecretCard {secret} onDeleted={loadSecrets} />
				{/each}
			</div>
		{/if}
	</div>
</div>

<AddSecretModal bind:open={showAdd} onSaved={loadSecrets} />
<ImportModal bind:open={showImport} onDone={loadSecrets} />
