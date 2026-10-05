<script lang="ts">
	import { untrack } from 'svelte';
	import AccountForm from './AccountForm.svelte';
	import { accountKinds, deleteAccount, listAccounts, setAccountArchived, type Account } from '#lib/api/accounts.ts';
	import Modal from '#lib/components/Modal.svelte';
	import Money from '#lib/components/Money.svelte';
	import PageHeader from '#lib/components/PageHeader.svelte';
	import States from '#lib/components/States.svelte';
	import Toggle from '#lib/components/Toggle.svelte';
	import { confirmAction } from '#lib/dialogs.ts';
	import { balanceSummary, type BalanceSummary } from '#lib/api/reports.ts';
	import { dataChanged, refs } from '#lib/refs.svelte.ts';
	import { notify, notifyError } from '#lib/toasts.svelte.ts';

	let showArchived = $state(false);
	let editing = $state<Account | null | undefined>(undefined);
	let request = $state<Promise<{ accounts: Account[]; summary: BalanceSummary }>>(new Promise(() => {}));

	function load() {
		request = Promise.all([listAccounts(showArchived), balanceSummary()]).then(([accounts, summary]) => ({ accounts, summary }));
	}

	$effect(() => {
		refs.version;
		untrack(load);
	});

	async function saved() {
		editing = undefined;
		await dataChanged();
	}

	async function toggleArchive(account: Account) {
		const archive = !account.archivedAt;
		if (archive && !(await confirmAction(`Arquivar a conta "${account.name}"? Deixa de aparecer nas listas e previsões.`))) return;
		await setAccountArchived(account.id, archive).catch(notifyError);
		notify(archive ? 'Conta arquivada' : 'Conta reativada');
		await dataChanged();
	}

	async function remove(account: Account) {
		if (!(await confirmAction(`Eliminar definitivamente a conta "${account.name}"?`))) return;
		try {
			await deleteAccount(account.id);
			notify('Conta eliminada');
			await dataChanged();
		} catch (e) {
			notifyError(e);
		}
	}
</script>

<PageHeader title="Contas" subtitle="Contas bancárias, poupanças, cartões e dinheiro físico.">
	{#snippet actions()}
		<Toggle bind:checked={showArchived} onchange={load} label="Mostrar arquivadas" />
		<button class="btn-primary" onclick={() => (editing = null)}>Nova conta</button>
	{/snippet}
</PageHeader>

{#await request}
	<States state="loading" />
{:then { accounts, summary }}
	{#if accounts.length === 0}
		<States state="empty" message="Sem contas.">
			<button class="btn-primary" onclick={() => (editing = null)}>Criar a primeira conta</button>
		</States>
	{:else}
		<p class="mb-4 text-sm">
			<span class="muted">Saldo total:</span>
			<span class="font-semibold tabular-nums"><Money value={summary.total} /></span>
		</p>
		<div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
			{#each accounts as account (account.id)}
				<article class="card flex flex-col gap-3" class:opacity-60={account.archivedAt}>
					<a href="/accounts/{account.id}" class="flex items-center gap-3">
						<span class="flex size-9 items-center justify-center rounded-full text-lg" style:background-color={account.color ?? 'gray'}>
							{account.icon ?? ''}
						</span>
						<div class="flex-1">
							<h2 class="font-medium">{account.name}</h2>
							<p class="text-xs text-stone-500">
								{accountKinds[account.kind]} · {account.currency}{account.kind === 'CREDIT_CARD' || account.kind === 'MEAL' ? ' · fora do saldo total' : ''}
							</p>
							{#if account.overdraftLimit}
								<p class="text-xs text-stone-500">
									{account.kind === 'CREDIT_CARD' ? 'Plafond' : 'Descoberto autorizado'}: <Money value={account.overdraftLimit} currency={account.currency} />
								</p>
							{/if}
						</div>
					</a>
					<dl class="grid grid-cols-3 gap-2 text-sm">
						<div>
							<dt class="text-xs text-stone-500">Saldo atual</dt>
							<dd class="font-semibold tabular-nums"><Money value={account.balance} currency={account.currency} /></dd>
						</div>
						<div>
							<dt class="text-xs text-stone-500" title="Saldo atual menos despesas futuras já registadas">Disponível</dt>
							<dd class="tabular-nums"><Money value={account.availableBalance} currency={account.currency} /></dd>
						</div>
						<div>
							<dt class="text-xs text-stone-500" title="Inclui todos os movimentos futuros registados">Projetado</dt>
							<dd class="tabular-nums"><Money value={account.projectedBalance} currency={account.currency} /></dd>
						</div>
					</dl>
					<div class="flex flex-wrap gap-1">
						<a class="btn-ghost" href="/accounts/{account.id}">Histórico</a>
						<button class="btn-ghost" onclick={() => (editing = account)}>Editar</button>
						<button class="btn-ghost" onclick={() => toggleArchive(account)}>{account.archivedAt ? 'Reativar' : 'Arquivar'}</button>
						<button class="btn-ghost text-red-600" onclick={() => remove(account)}>Eliminar</button>
					</div>
				</article>
			{/each}
		</div>
	{/if}
{:catch error}
	<States state="error" {error} />
{/await}

{#if editing !== undefined}
	<Modal title={editing ? 'Editar conta' : 'Nova conta'} onclose={() => (editing = undefined)}>
		<AccountForm account={editing} onsaved={saved} oncancel={() => (editing = undefined)} />
	</Modal>
{/if}
