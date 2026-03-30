<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { auth } from '$lib/auth.svelte';
	import { api } from '$lib/api';
	import type { SecretRecord } from '$lib/types';
	import SecretCard from '$lib/components/SecretCard.svelte';
	import AddSecretModal from '$lib/components/AddSecretModal.svelte';
	import ImportModal from '$lib/components/ImportModal.svelte';

	let secrets = $state<SecretRecord[]>([]);
	let search = $state('');
	let showAdd = $state(false);
	let showImport = $state(false);
	let loading = $state(true);
	let selectedIndex = $state(0);
	let loadError = $state('');

	// Refs to SecretCard components for calling methods
	let cardRefs = $state<Record<string, SecretCard>>({});

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

	// Reset selection when filter changes
	$effect(() => {
		filtered; // track
		selectedIndex = 0;
	});

	async function loadSecrets(silent = false) {
		if (!silent) loading = true;
		loadError = '';
		try {
			const result = await api.secrets.list();
			secrets = result as SecretRecord[];
		} catch (e: any) {
			console.error('Failed to load secrets', e);
			loadError = e?.message || 'Failed to load secrets';
			if (loadError.includes('401') || loadError.includes('403')) {
				await auth.logout();
				goto('/login');
			}
		} finally {
			loading = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		const modal = showAdd || showImport;
		const searchFocused = document.activeElement?.id === 'search';

		// Global shortcuts
		if ((e.metaKey || e.ctrlKey) && e.key === 'n') {
			e.preventDefault();
			showAdd = true;
			return;
		}

		if (e.key === '/' && !modal) {
			e.preventDefault();
			document.getElementById('search')?.focus();
			return;
		}

		if (e.key === 'Escape' && searchFocused) {
			search = '';
			(document.activeElement as HTMLElement)?.blur();
			return;
		}

		if (modal) return;

		// Navigation
		if (e.key === 'ArrowDown' || (e.key === 'j' && !searchFocused)) {
			e.preventDefault();
			selectedIndex = Math.min(selectedIndex + 1, filtered.length - 1);
			scrollToSelected();
		}
		if (e.key === 'ArrowUp' || (e.key === 'k' && !searchFocused)) {
			e.preventDefault();
			selectedIndex = Math.max(selectedIndex - 1, 0);
			scrollToSelected();
		}

		// Actions on selected
		if (filtered.length > 0 && !searchFocused) {
			const selectedSecret = filtered[selectedIndex];
			const ref = cardRefs[selectedSecret?.id];

			if (e.key === 'Enter') {
				e.preventDefault();
				ref?.copyPassword();
			}
			if (e.key === 'u') {
				e.preventDefault();
				ref?.copyUsername();
			}
			if (e.key === ' ') {
				e.preventDefault();
				ref?.toggleReveal();
			}
		}
	}

	function scrollToSelected() {
		const el = document.querySelector(`[data-index="${selectedIndex}"]`);
		el?.scrollIntoView({ block: 'nearest' });
	}

	async function handleLogout() {
		await auth.logout();
		goto('/login');
	}

	onMount(async () => {
		while (!auth.isReady) {
			await new Promise((r) => setTimeout(r, 50));
		}
		if (!auth.isUnlocked) {
			goto('/login');
			return;
		}
		loadSecrets();
	});
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="flex min-h-screen flex-col bg-zinc-950">
	<!-- Header: search-first, minimal chrome -->
	<header class="sticky top-0 z-10 border-b border-zinc-800/50 bg-zinc-950/95 backdrop-blur-sm">
		<div class="mx-auto flex max-w-4xl items-center gap-3 px-4 py-2">
			<span class="text-sm font-bold tracking-tight text-zinc-500">VLT</span>

			<!-- Search -->
			<div class="relative flex-1">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-zinc-600"
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
					class="w-full rounded-lg border border-zinc-800/50 bg-zinc-900/50 py-1.5 pl-8 pr-3 text-sm text-zinc-200 placeholder-zinc-600 outline-none focus:border-zinc-700 focus:bg-zinc-900"
					placeholder="Search...  (/) "
				/>
			</div>

			<div class="flex items-center gap-1">
				<button
					onclick={() => (showImport = true)}
					class="rounded px-2 py-1 text-xs text-zinc-600 transition hover:text-zinc-300"
				>
					Import
				</button>
				<button
					onclick={() => (showAdd = true)}
					class="rounded bg-zinc-800 px-2 py-1 text-xs text-zinc-300 transition hover:bg-zinc-700"
				>
					+New
				</button>
				<button
					onclick={handleLogout}
					class="rounded px-2 py-1 text-xs text-zinc-600 transition hover:text-zinc-400"
				>
					Lock
				</button>
			</div>
		</div>

		<!-- Keyboard hints -->
		<div class="mx-auto flex max-w-4xl gap-4 border-t border-zinc-800/30 px-4 py-1 text-[10px] text-zinc-700">
			<span><kbd class="text-zinc-600">↑↓</kbd> navigate</span>
			<span><kbd class="text-zinc-600">Enter</kbd> copy password</span>
			<span><kbd class="text-zinc-600">u</kbd> copy username</span>
			<span><kbd class="text-zinc-600">Space</kbd> reveal</span>
			<span><kbd class="text-zinc-600">⌘N</kbd> new</span>
			<span><kbd class="text-zinc-600">/</kbd> search</span>
		</div>
	</header>

	{#if loadError}
		<div class="mx-auto max-w-4xl px-4 pt-2">
			<p class="rounded bg-red-500/10 px-3 py-1.5 text-xs text-red-400">{loadError}</p>
		</div>
	{/if}

	<!-- Table header -->
	{#if !loading && filtered.length > 0}
		<div class="mx-auto w-full max-w-4xl">
			<div class="flex items-center gap-2 px-3 py-1.5 text-[10px] font-medium uppercase tracking-wider text-zinc-700">
				<span class="w-4 shrink-0"></span>
				<span class="w-40 shrink-0">Name</span>
				<span class="w-36 shrink-0">Username</span>
				<span class="min-w-0 flex-1">Password</span>
				<span class="w-16 shrink-0"></span>
			</div>
		</div>
	{/if}

	<!-- Secrets list -->
	<div class="mx-auto w-full max-w-4xl flex-1 overflow-y-auto">
		{#if loading}
			<div class="py-20 text-center text-sm text-zinc-600">Loading...</div>
		{:else if filtered.length === 0}
			<div class="py-20 text-center">
				{#if search}
					<p class="text-sm text-zinc-600">No match for "{search}"</p>
				{:else}
					<p class="text-sm text-zinc-600">Empty vault</p>
					<div class="mt-3 flex justify-center gap-3">
						<button
							onclick={() => (showAdd = true)}
							class="text-xs text-emerald-500 hover:underline"
						>
							Add secret
						</button>
						<button
							onclick={() => (showImport = true)}
							class="text-xs text-zinc-500 hover:underline"
						>
							Import from browser
						</button>
					</div>
				{/if}
			</div>
		{:else}
			<div class="divide-y divide-zinc-800/30">
				{#each filtered as secret, i (secret.id)}
					<div data-index={i}>
						<SecretCard
							{secret}
							selected={i === selectedIndex}
							onDeleted={loadSecrets}
							onUsed={() => loadSecrets(true)}
							bind:this={cardRefs[secret.id]}
						/>
					</div>
				{/each}
			</div>
			<div class="px-4 py-2 text-right text-[10px] text-zinc-700">
				{filtered.length}{filtered.length !== secrets.length ? ` / ${secrets.length}` : ''} secrets
			</div>
		{/if}
	</div>
</div>

<AddSecretModal bind:open={showAdd} onSaved={loadSecrets} />
<ImportModal bind:open={showImport} onDone={loadSecrets} />
