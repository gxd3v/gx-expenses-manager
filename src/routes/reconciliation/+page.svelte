<script lang="ts">
	import { untrack } from 'svelte';
	import { page } from '$app/state';
	import {
		forgottenTransactions,
		listReconciliations,
		reconcile,
		reconciliationStatus
	} from '#lib/api/reconciliation.ts';
	import { listTransactions, setConfirmed } from '#lib/api/transactions.ts';
	import AccountSelect from '#lib/components/AccountSelect.svelte';
	import Amount from '#lib/components/Amount.svelte';
	import MoneyInput from '#lib/components/MoneyInput.svelte';
	import PageHeader from '#lib/components/PageHeader.svelte';
	import StatCard from '#lib/components/StatCard.svelte';
	import States from '#lib/components/States.svelte';
	import { addMonths, formatDate, formatMoney, monthStart, today } from '#lib/format.ts';
	import { dataChanged, refs } from '#lib/refs.svelte.ts';
	import { notify, notifyError } from '#lib/toasts.svelte.ts';

	let accountId = $state<string | null>(untrack(() => page.url.searchParams.get('account') ?? refs.accounts[0]?.id ?? null));
	let date = $state(today());
	let from = $state(monthStart(addMonths(today(), -1)));
	let statement = $state<number | null>(null);

	async function load(account: string, until: string, since: string) {
		const [status, unconfirmed, history, forgotten] = await Promise.all([
			reconciliationStatus(account, until),
			listTransactions({ accountId: account, confirmed: false, dateFrom: since, dateTo: until }, 200),
			listReconciliations(account),
			forgottenTransactions(account, until)
		]);
		return { status, unconfirmed: unconfirmed.items, history, forgotten };
	}

	let request = $state<ReturnType<typeof load> | null>(null);

	$effect(() => {
		refs.version;
		const args = [accountId, date, from] as const;
		untrack(() => (request = args[0] ? load(args[0], args[1], args[2]) : null));
	});

	async function confirmOne(id: string) {
		await setConfirmed([id], true).catch(notifyError);
		await dataChanged();
	}

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		if (!accountId || statement === null) return;
		try {
			const result = await reconcile(accountId, date, statement, from);
			notify(result.difference === 0 ? 'Conta reconciliada sem diferenças' : `Reconciliação registada com diferença de ${formatMoney(result.difference)}`);
			await dataChanged();
		} catch (e) {
			notifyError(e);
		}
	}
</script>

<PageHeader title="Reconciliação" subtitle="Compara o saldo calculado com o saldo real do banco." />

<form onsubmit={submit} class="card mb-6 grid grid-cols-2 gap-4 md:grid-cols-5">
	<label class="label">
		Conta
		<AccountSelect bind:value={accountId} />
	</label>
	<label class="label">
		Período desde
		<input type="date" bind:value={from} class="input" />
	</label>
	<label class="label">
		Até (data do extrato)
		<input type="date" bind:value={date} class="input" />
	</label>
	<label class="label">
		Saldo real no banco
		<MoneyInput bind:value={statement} required />
	</label>
	<div class="flex items-end">
		<button class="btn-primary w-full" disabled={statement === null}>Reconciliar período</button>
	</div>
	<p class="col-span-2 muted md:col-span-5">Reconciliar marca como confirmados os movimentos do período e guarda o resultado no histórico.</p>
</form>

{#if !request}
	<States state="empty" message="Escolhe uma conta." />
{:else}
	{#await request}
		<States state="loading" />
	{:then { status, unconfirmed, history, forgotten }}
		<div class="mb-6 grid gap-4 md:grid-cols-4">
			<StatCard label="Saldo calculado" value={formatMoney(status.calculatedBalance)} hint="Todos os movimentos até {formatDate(date)}" />
			<StatCard label="Saldo confirmado" value={formatMoney(status.confirmedBalance)} hint="Só movimentos confirmados" />
			<StatCard
				label="Diferença para o banco"
				value={statement === null ? '—' : formatMoney(statement - status.calculatedBalance)}
				tone={statement === null || statement === status.calculatedBalance ? 'neutral' : 'negative'}
			/>
			<StatCard label="Por confirmar" value={String(status.unconfirmedCount)} hint={formatMoney(status.unconfirmedTotal)} />
		</div>

		<div class="grid gap-4 xl:grid-cols-2">
			<section class="card">
				<h2 class="mb-3 font-medium">Movimentos por confirmar no período</h2>
				{#if unconfirmed.length === 0}
					<p class="muted">Tudo confirmado.</p>
				{:else}
					<ul class="divide-y divide-stone-100 text-sm dark:divide-stone-800">
						{#each unconfirmed as transaction (transaction.id)}
							<li class="flex items-center gap-3 py-1.5">
								<span class="w-24 text-stone-500">{formatDate(transaction.date)}</span>
								<span class="flex-1 truncate">{transaction.description || '—'}</span>
								<Amount value={transaction.amount} />
								<button class="btn-ghost" onclick={() => confirmOne(transaction.id)}>Confirmar</button>
							</li>
						{/each}
					</ul>
				{/if}
			</section>
			<section class="card">
				<h2 class="mb-1 font-medium">Possíveis movimentos esquecidos</h2>
				<p class="mb-3 muted">Movimentos habituais nos meses anteriores que ainda não aparecem este mês.</p>
				{#if forgotten.length === 0}
					<p class="muted">Nada em falta detetado.</p>
				{:else}
					<ul class="divide-y divide-stone-100 text-sm dark:divide-stone-800">
						{#each forgotten as item (item.description)}
							<li class="flex justify-between gap-3 py-1.5">
								<span>{item.description} <span class="text-xs text-stone-500">· {item.categoryName ?? 'Sem categoria'} · {item.monthsSeen} meses</span></span>
								<Amount value={item.averageAmount} />
							</li>
						{/each}
					</ul>
				{/if}
			</section>
		</div>

		<section class="card mt-6">
			<h2 class="mb-3 font-medium">Histórico de reconciliações</h2>
			{#if history.length === 0}
				<p class="muted">Ainda não fizeste reconciliações nesta conta.</p>
			{:else}
				<table class="table-base">
					<thead><tr><th>Data</th><th class="text-right">Banco</th><th class="text-right">Calculado</th><th class="text-right">Diferença</th></tr></thead>
					<tbody>
						{#each history as item (item.id)}
							<tr>
								<td>{formatDate(item.date)}</td>
								<td class="text-right tabular-nums">{formatMoney(item.statementBalance)}</td>
								<td class="text-right tabular-nums">{formatMoney(item.calculatedBalance)}</td>
								<td class="text-right"><Amount value={item.difference} /></td>
							</tr>
						{/each}
					</tbody>
				</table>
			{/if}
		</section>
	{:catch error}
		<States state="error" {error} />
	{/await}
{/if}
