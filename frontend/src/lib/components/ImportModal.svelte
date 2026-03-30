<script lang="ts">
	import Papa from 'papaparse';
	import { auth } from '$lib/auth.svelte';
	import { encrypt } from '$lib/crypto';
	import { api } from '$lib/api';
	import type { BrowserPasswordEntry } from '$lib/types';

	let { open = $bindable(false), onDone }: { open: boolean; onDone: () => void } = $props();

	let entries = $state<BrowserPasswordEntry[]>([]);
	let importing = $state(false);
	let progress = $state(0);
	let error = $state('');
	let step = $state<'upload' | 'preview' | 'importing' | 'done'>('upload');
	let skipped = $state(0);
	let detectedHeaders = $state<string[]>([]);

	function close() {
		open = false;
		entries = [];
		error = '';
		progress = 0;
		skipped = 0;
		step = 'upload';
	}

	function handleFile(e: Event) {
		const input = e.target as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;

		error = '';

		Papa.parse(file, {
			header: true,
			skipEmptyLines: true,
			complete(results) {
				const headers = results.meta.fields || [];
				// Normalize: find columns case-insensitively
				const h = new Set(headers.map((h) => h.toLowerCase()));

				const hasPassword = h.has('password');
				const hasUrl = h.has('url');

				if (!hasPassword) {
					error = `No "password" column found. Headers: ${headers.join(', ')}`;
					return;
				}

				// Build a case-insensitive column lookup
				const colMap: Record<string, string> = {};
				for (const header of headers) {
					colMap[header.toLowerCase()] = header;
				}

				const parsed: BrowserPasswordEntry[] = [];

				for (const row of results.data as Record<string, string>[]) {
					const get = (key: string) => row[colMap[key] || ''] || '';

					const entry: BrowserPasswordEntry = {
						name: get('name') || get('title') || '',
						url: get('url') || '',
						username: get('username') || get('login') || '',
						password: get('password') || '',
						note: get('note') || get('notes') || ''
					};

					console.log('Parsed entry:', entry.name, 'username:', entry.username);

					// Skip entries with no password
					if (!entry.password) {
						skipped++;
						continue;
					}

					// Use hostname as name if name is empty
					if (!entry.name && entry.url) {
						try {
							entry.name = new URL(entry.url).hostname;
						} catch {
							entry.name = entry.url;
						}
					}

					parsed.push(entry);
				}

				entries = parsed;
				detectedHeaders = headers;
				step = 'preview';
			},
			error(err) {
				error = `Failed to parse CSV: ${err.message}`;
			}
		});

		// Reset input so same file can be re-selected
		input.value = '';
	}

	async function handleImport() {
		if (!auth.key) return;

		step = 'importing';
		importing = true;
		progress = 0;

		try {
			for (let i = 0; i < entries.length; i++) {
				const entry = entries[i];
				const { ciphertext, iv } = await encrypt(auth.key, entry.password);

				await api.secrets.create({
					name: entry.name,
					type: 'password',
					encrypted_value: ciphertext,
					iv,
					username: entry.username || '',
					url: entry.url || ''
				});

				progress = i + 1;
			}

			step = 'done';
		} catch (err: any) {
			error = `Import failed at ${progress + 1}/${entries.length}: ${err?.message || 'Unknown error'}`;
			step = 'preview';
		} finally {
			importing = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape' && !importing) close();
	}
</script>

{#if open}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 px-4"
		onkeydown={handleKeydown}
	>
		<!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
		<div class="fixed inset-0" onclick={() => !importing && close()}></div>

		<div class="relative w-full max-w-lg rounded-xl border border-zinc-800 bg-zinc-900 p-6">
			{#if step === 'upload'}
				<h2 class="mb-4 text-lg font-semibold text-zinc-100">Import from Browser</h2>
				<p class="mb-4 text-sm text-zinc-400">
					Export your passwords as CSV from your browser, then upload the file here.
				</p>
				<p class="mb-4 text-xs text-zinc-500">
					Brave/Chrome: Settings → Passwords → Export passwords<br />
					Firefox: Settings → Passwords → Export<br />
					Safari: Settings → Passwords → Export All Passwords
				</p>

				<label
					class="flex cursor-pointer flex-col items-center justify-center rounded-xl border-2 border-dashed border-zinc-700 bg-zinc-800/50 p-8 transition hover:border-zinc-600"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="mb-2 h-8 w-8 text-zinc-500"
						viewBox="0 0 20 20"
						fill="currentColor"
					>
						<path
							fill-rule="evenodd"
							d="M3 17a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM6.293 6.707a1 1 0 010-1.414l3-3a1 1 0 011.414 0l3 3a1 1 0 01-1.414 1.414L11 5.414V13a1 1 0 11-2 0V5.414L7.707 6.707a1 1 0 01-1.414 0z"
							clip-rule="evenodd"
						/>
					</svg>
					<span class="text-sm text-zinc-400">Choose CSV file</span>
					<input type="file" accept=".csv" class="hidden" onchange={handleFile} />
				</label>

				{#if error}
					<p class="mt-3 text-sm text-red-400">{error}</p>
				{/if}

				<div class="mt-4 flex justify-end">
					<button
						onclick={close}
						class="rounded-lg px-4 py-2 text-sm text-zinc-400 transition hover:text-zinc-200"
					>
						Cancel
					</button>
				</div>

			{:else if step === 'preview'}
				<h2 class="mb-1 text-lg font-semibold text-zinc-100">Preview Import</h2>
				<p class="mb-2 text-sm text-zinc-400">
					{entries.length} passwords found{skipped ? `, ${skipped} skipped (no password)` : ''}
				</p>
				<p class="mb-4 text-xs text-zinc-600">
					CSV columns: {detectedHeaders.join(', ')}
				</p>

				<div class="max-h-64 space-y-1 overflow-y-auto rounded-lg bg-zinc-800 p-3">
					{#each entries as entry, i}
						<div class="flex items-center justify-between rounded px-2 py-1.5 text-sm hover:bg-zinc-700/50">
							<div class="min-w-0 flex-1">
								<span class="truncate text-zinc-200">{entry.name}</span>
								{#if entry.username}
									<span class="ml-2 text-zinc-500">{entry.username}</span>
								{/if}
							</div>
						</div>
					{/each}
				</div>

				{#if error}
					<p class="mt-3 text-sm text-red-400">{error}</p>
				{/if}

				<div class="mt-4 flex justify-end gap-3">
					<button
						onclick={() => { step = 'upload'; entries = []; skipped = 0; }}
						class="rounded-lg px-4 py-2 text-sm text-zinc-400 transition hover:text-zinc-200"
					>
						Back
					</button>
					<button
						onclick={handleImport}
						class="rounded-lg bg-emerald-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-emerald-500"
					>
						Import {entries.length} passwords
					</button>
				</div>

			{:else if step === 'importing'}
				<h2 class="mb-4 text-lg font-semibold text-zinc-100">Importing...</h2>
				<div class="mb-2 h-2 overflow-hidden rounded-full bg-zinc-800">
					<div
						class="h-full rounded-full bg-emerald-500 transition-all"
						style="width: {(progress / entries.length) * 100}%"
					></div>
				</div>
				<p class="text-sm text-zinc-400">
					Encrypting and saving {progress}/{entries.length}
				</p>

			{:else if step === 'done'}
				<div class="py-4 text-center">
					<div class="mb-3 text-4xl">&#10003;</div>
					<h2 class="text-lg font-semibold text-zinc-100">Import Complete</h2>
					<p class="mt-1 text-sm text-zinc-400">
						{entries.length} passwords encrypted and saved
					</p>
					<button
						onclick={() => { close(); onDone(); }}
						class="mt-4 rounded-lg bg-emerald-600 px-6 py-2 text-sm font-medium text-white transition hover:bg-emerald-500"
					>
						Done
					</button>
				</div>
			{/if}
		</div>
	</div>
{/if}
