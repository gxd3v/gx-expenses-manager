<script lang="ts">
	import { onMount } from 'svelte';
	import AccountForm from './AccountForm.svelte';
	import { accountKinds, archiveAccount, listAccounts, type Account } from '#lib/accounts.ts';
	import { formatMoney } from '#lib/money.ts';

	let accounts = $state<Account[]>([]);
	let showArchived = $state(false);
	let formOpen = $state(false);
	let editing = $state<Account | null>(null);
	let error = $state('');

	async function load() {
		try {
			accounts = await listAccounts(showArchived);
			error = '';
		} catch (e) {
			error = String(e);
		}
	}

	function openForm(account: Account | null) {
		editing = account;
		formOpen = true;
	}

	async function saved() {
		formOpen = false;
		await load();
	}

	async function archive(account: Account) {
		if (!confirm(`Arquivar a conta "${account.name}"?`)) return;
		try {
			await archiveAccount(account.id);
			await load();
		} catch (e) {
			error = String(e);
		}
	}

	onMount(load);
</script>

<div class="space-y-6">
	<header class="flex items-center justify-between">
		<h1 class="text-2xl font-semibold">Contas</h1>
		<div class="flex items-center gap-4">
			<label class="flex items-center gap-2 text-sm">
				<input type="checkbox" bind:checked={showArchived} onchange={load} class="rounded" />
				Mostrar arquivadas
			</label>
			<button onclick={() => openForm(null)} class="btn-primary">Nova conta</button>
		</div>
	</header>

	{#if formOpen}
		{#key editing}
			<AccountForm account={editing} onsaved={saved} oncancel={() => (formOpen = false)} />
		{/key}
	{/if}

	{#if error}
		<p class="text-sm text-red-600">{error}</p>
	{/if}

	{#if accounts.length === 0}
		<p class="text-sm text-slate-500">Ainda não tens contas. Cria a primeira para começar.</p>
	{:else}
		<ul class="divide-y divide-slate-200 rounded-xl border border-slate-200 dark:divide-slate-800 dark:border-slate-800">
			{#each accounts as account (account.id)}
				<li class="flex items-center gap-4 px-4 py-3" class:opacity-50={account.archivedAt}>
					<span class="size-3 rounded-full" style:background-color={account.color ?? 'gray'}></span>
					<div class="flex-1">
						<p class="font-medium">{account.name}</p>
						<p class="text-xs text-slate-500">{accountKinds[account.kind]}</p>
					</div>
					<span class="tabular-nums">{formatMoney(account.initialBalance, account.currency)}</span>
					{#if !account.archivedAt}
						<button onclick={() => openForm(account)} class="btn-secondary">Editar</button>
						<button onclick={() => archive(account)} class="btn-secondary">Arquivar</button>
					{/if}
				</li>
			{/each}
		</ul>
	{/if}
</div>
