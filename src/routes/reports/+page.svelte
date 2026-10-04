<script lang="ts">
	import { untrack } from 'svelte';
	import { balanceHistory, categoryBreakdown, monthlyTotals } from '#lib/api/reports.ts';
	import BarChart from '#lib/charts/BarChart.svelte';
	import HBarChart from '#lib/charts/HBarChart.svelte';
	import LineChart from '#lib/charts/LineChart.svelte';
	import AccountSelect from '#lib/components/AccountSelect.svelte';
	import PageHeader from '#lib/components/PageHeader.svelte';
	import States from '#lib/components/States.svelte';
	import { addMonths, formatMoney, formatMonth, monthStart, today } from '#lib/format.ts';
	import { refs } from '#lib/refs.svelte.ts';

	const ranges = [6, 12, 24];

	let months = $state(12);
	let accountId = $state<string | null>(null);

	async function load(range: number, account: string | null) {
		const from = monthStart(addMonths(today(), -(range - 1)));
		const to = today();
		const [totals, history, parents, leaves] = await Promise.all([
			monthlyTotals(from, to, account),
			balanceHistory(range, account),
			categoryBreakdown('OUTCOME', from, to, 'PARENT', account),
			categoryBreakdown('OUTCOME', from, to, 'LEAF', account)
		]);
		return { totals, history, parents, leaves };
	}

	let request = $state<ReturnType<typeof load>>(new Promise(() => {}));

	$effect(() => {
		refs.version;
		const [range, account] = [months, accountId];
		untrack(() => (request = load(range, account)));
	});

	const money = (v: number) => formatMoney(v);
</script>

<PageHeader title="Gráficos" subtitle="Evolução e distribuição das tuas finanças.">
	{#snippet actions()}
		<div class="w-48"><AccountSelect bind:value={accountId} allowEmpty /></div>
		{#each ranges as range (range)}
			<button class={months === range ? 'btn-primary' : 'btn-secondary'} onclick={() => (months = range)}>{range} meses</button>
		{/each}
	{/snippet}
</PageHeader>

{#await request}
	<States state="loading" />
{:then { totals, history, parents, leaves }}
	{@const labels = totals.map((t) => formatMonth(t.month))}
	<div class="grid gap-4 xl:grid-cols-2">
		<section class="card">
			<h2 class="mb-3 font-medium">Receitas vs despesas</h2>
			<BarChart
				{labels}
				series={[
					{ name: 'Receitas', color: 'var(--series-3)', values: totals.map((t) => t.income) },
					{ name: 'Despesas', color: 'var(--series-2)', values: totals.map((t) => t.outcome) }
				]}
				format={money}
			/>
		</section>
		<section class="card">
			<h2 class="mb-3 font-medium">Resultado mensal</h2>
			<BarChart
				{labels}
				series={[{ name: 'Resultado', color: 'var(--positive)', negativeColor: 'var(--negative)', values: totals.map((t) => t.net) }]}
				format={money}
			/>
		</section>
		<section class="card">
			<h2 class="mb-3 font-medium">{accountId ? 'Evolução da conta' : 'Evolução do saldo e do património'}</h2>
			<LineChart
				labels={history.map((p) => formatMonth(p.month))}
				series={[
					{ name: 'Saldo', color: 'var(--series-1)', values: history.map((p) => p.balance) },
					...(accountId ? [] : [{ name: 'Património líquido', color: 'var(--series-2)', values: history.map((p) => p.netWorth) }])
				]}
				format={money}
			/>
		</section>
		<section class="card">
			<h2 class="mb-3 font-medium">Gastos ao longo do tempo</h2>
			<LineChart labels={labels} series={[{ name: 'Despesas', color: 'var(--series-2)', values: totals.map((t) => t.outcome) }]} format={money} />
		</section>
		<section class="card">
			<h2 class="mb-3 font-medium">Gastos por categoria</h2>
			{#if parents.length === 0}<p class="muted">Sem despesas no período.</p>{:else}
				<HBarChart items={parents.map((c) => ({ label: c.name, value: c.amount }))} format={money} limit={10} />
			{/if}
		</section>
		<section class="card">
			<h2 class="mb-3 font-medium">Gastos por subcategoria</h2>
			{#if leaves.length === 0}<p class="muted">Sem despesas no período.</p>{:else}
				<HBarChart items={leaves.map((c) => ({ label: c.name, value: c.amount }))} format={money} limit={12} />
			{/if}
		</section>
	</div>
	<p class="mt-4 muted">
		Para comparar meses em detalhe usa a <a class="underline" href="/monthly">vista mensal</a>; a evolução de créditos e objetivos está nas
		respetivas páginas.
	</p>
{:catch error}
	<States state="error" {error} />
{/await}
