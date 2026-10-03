<script lang="ts">
	import './layout.css';
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import UnlockScreen from '#lib/components/UnlockScreen.svelte';
	import { lock, status, type Status } from '#lib/session.ts';

	let { children } = $props();

	let session = $state<Status | null>(null);

	const links = [{ href: '/accounts', label: 'Contas' }];

	async function refresh() {
		session = await status();
	}

	async function handleLock() {
		await lock();
		await refresh();
	}

	onMount(refresh);
</script>

{#if session && !session.unlocked}
	<UnlockScreen initialized={session.initialized} onunlock={refresh} />
{:else if session}
	<div class="flex h-screen">
		<nav class="flex w-52 flex-col gap-1 border-r border-slate-200 p-4 dark:border-slate-800">
			<span class="mb-4 font-semibold">GX Expenses</span>
			{#each links as link (link.href)}
				<a
					href={link.href}
					class="rounded-md px-3 py-1.5 text-sm hover:bg-slate-100 dark:hover:bg-slate-800"
					class:font-medium={page.url.pathname.startsWith(link.href)}
				>
					{link.label}
				</a>
			{/each}
			<button onclick={handleLock} class="btn-secondary mt-auto">Bloquear</button>
		</nav>
		<main class="flex-1 overflow-y-auto p-8">
			{@render children()}
		</main>
	</div>
{/if}
