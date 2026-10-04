<script lang="ts">
	import { untrack } from 'svelte';
	import { page } from '$app/state';
	import { accountKinds, getAccount } from '#lib/api/accounts.ts';
	import { balanceHistory, monthlyTotals } from '#lib/api/reports.ts';
	import BarChart from '#lib/charts/BarChart.svelte';
	import LineChart from '#lib/charts/LineChart.svelte';
	import BalanceRecords from '#lib/components/BalanceRecords.svelte';
	import PageHeader from '#lib/components/PageHeader.svelte';
	import StatCard from '#lib/components/StatCard.svelte';
	import States from '#lib/components/States.svelte';
	import TransactionList from '#lib/components/TransactionList.svelte';
	import { addMonths, formatMonth, formatMoney, today } from '#lib/format.ts';
	import { refs } from '#lib/refs.svelte.ts';
	import { openQuickAdd } from '#lib/ui.svelte.ts';

	const MONTHS = 12;
	const id = $derived(page.params.id ?? '');

	async function load(accountId: string) {
		const [account, history, totals] = await Promise.all([
			getAccount(accountId),
			balanceHistory(MONTHS, accountId),
			monthlyTotals(addMonths(today(), -(MONTHS - 1)), today(), accountId)
		]);
		return { account, history, totals };
	}

	let request = $state<ReturnType<typeof load>>(new Promise(() => {}));

	$effect(() => {
		refs.version;
		const accountId = id;
		untrack(() => (request = load(accountId)));
	});
</script>

{#await request}
	<States state="loading" />
{:then { account, history, totals }}
	<PageHeader title={account.name} subtitle="{accountKinds[account.kind]} · {account.currency}">
		{#snippet actions()}
			<a class="btn-secondary" href="/reconciliation?account={account.id}">Reconciliar</a>
			<button class="btn-primary" onclick={() => openQuickAdd('transaction', { accountId: account.id })}>Novo movimento</button>
		{/snippet}
	</PageHeader>

	<div class="mb-6 grid gap-4 md:grid-cols-3">
		<StatCard label="Saldo atual" value={formatMoney(account.balance, account.currency)} />
		<StatCard label="Saldo disponível" value={formatMoney(account.availableBalance, account.currency)} hint="Depois das despesas futuras registadas" />
		<StatCard label="Saldo projetado" value={formatMoney(account.projectedBalance, account.currency)} hint="Com todos os movimentos futuros" />
	</div>

	<div class="mb-6 grid gap-4 lg:grid-cols-2 xl:grid-cols-3">
		<section class="card">
			<h2 class="mb-3 font-medium">Evolução do saldo</h2>
			<LineChart
				labels={history.map((p) => formatMonth(p.month))}
				series={[{ name: 'Saldo', color: 'var(--series-1)', values: history.map((p) => p.balance) }]}
				format={(v) => formatMoney(v, account.currency)}
			/>
		</section>
		<section class="card">
			<h2 class="mb-3 font-medium">Receitas e despesas</h2>
			<BarChart
				labels={totals.map((t) => formatMonth(t.month))}
				series={[
					{ name: 'Receitas', color: 'var(--series-3)', values: totals.map((t) => t.income) },
					{ name: 'Despesas', color: 'var(--series-2)', values: totals.map((t) => t.outcome) }
				]}
				format={(v) => formatMoney(v, account.currency)}
			/>
		</section>
		<BalanceRecords accountId={account.id} currency={account.currency} />
	</div>

	<h2 class="mb-3 text-lg font-semibold">Histórico</h2>
	{#key account.id}
		<TransactionList fixedAccountId={account.id} />
	{/key}
{:catch error}
	<States state="error" {error} />
{/await}
