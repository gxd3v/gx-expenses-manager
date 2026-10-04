<script lang="ts">
	import { untrack } from 'svelte';
	import { listOccurrences } from '#lib/api/recurrences.ts';
	import { monthSummary } from '#lib/api/reports.ts';
	import { listTransactions } from '#lib/api/transactions.ts';
	import HBarChart from '#lib/charts/HBarChart.svelte';
	import Amount from '#lib/components/Amount.svelte';
	import Money from '#lib/components/Money.svelte';
	import PageHeader from '#lib/components/PageHeader.svelte';
	import StatCard from '#lib/components/StatCard.svelte';
	import States from '#lib/components/States.svelte';
	import { addDays, addMonths, formatDate, formatMoney, formatMonth, monthEnd, monthStart, today } from '#lib/format.ts';
	import { refs } from '#lib/refs.svelte.ts';

	let month = $state(monthStart(today()));

	async function load(selected: string) {
		const start = monthStart(selected);
		const end = monthEnd(selected);
		const pendingFrom = start > today() ? start : addDays(today(), 1);
		const [summary, occurrences, future] = await Promise.all([
			monthSummary(start),
			pendingFrom <= end ? listOccurrences(pendingFrom, end) : Promise.resolve([]),
			pendingFrom <= end ? listTransactions({ dateFrom: pendingFrom, dateTo: end }, 200) : Promise.resolve(null)
		]);

		const planned = [
			...occurrences
				.filter((o) => !o.toAccountId)
				.map((o) => ({
				key: `${o.recurrenceId}-${o.occurrenceDate}`,
				date: o.date,
				description: o.description,
				amount: o.kind === 'INCOME' ? o.amount : -o.amount
			})),
			...(future?.items ?? [])
				.filter((t) => t.kind !== 'TRANSFER')
				.map((t) => ({ key: t.id, date: t.date, description: t.description, amount: t.amount }))
		].sort((a, b) => a.date.localeCompare(b.date));

		return { summary, planned };
	}

	let request = $state<ReturnType<typeof load>>(new Promise(() => {}));

	$effect(() => {
		refs.version;
		const selected = month;
		untrack(() => (request = load(selected)));
	});

	function delta(current: number, reference: number): string {
		if (reference === 0) return '';
		const change = ((current - reference) / reference) * 100;
		return `${change >= 0 ? '+' : ''}${change.toFixed(0)}%`;
	}
</script>

<PageHeader title="Vista mensal" subtitle={formatMonth(month, 'long')}>
	{#snippet actions()}
		<button class="btn-secondary" onclick={() => (month = addMonths(month, -1))} aria-label="Mês anterior">←</button>
		<input type="month" value={month.slice(0, 7)} onchange={(e) => (month = `${e.currentTarget.value}-01`)} class="input w-40" />
		<button class="btn-secondary" onclick={() => (month = addMonths(month, 1))} aria-label="Mês seguinte">→</button>
		<button class="btn-ghost" onclick={() => (month = monthStart(today()))}>Hoje</button>
	{/snippet}
</PageHeader>

{#await request}
	<States state="loading" />
{:then { summary, planned }}
	<div class="mb-6 grid gap-4 md:grid-cols-3">
		<StatCard
			privateHint
			label="Receitas realizadas"
			value={formatMoney(summary.income)}
			hint="Previstas: {formatMoney(summary.pendingIncome)} · Total esperado: {formatMoney(summary.expectedIncome)}"
		/>
		<StatCard
			privateHint
			label="Despesas realizadas"
			value={formatMoney(summary.outcome)}
			hint="Previstas: {formatMoney(summary.pendingOutcome)} · Total esperado: {formatMoney(summary.expectedOutcome)}"
		/>
		<StatCard
			privateHint
			label="Resultado"
			value={formatMoney(summary.net)}
			tone={summary.net >= 0 ? 'positive' : 'negative'}
			hint="Com o previsto: {formatMoney(summary.expectedNet)}"
		/>
	</div>

	<section class="card mb-6">
		<h2 class="mb-3 font-medium">Comparação</h2>
		<div class="overflow-x-auto">
		<table class="table-base">
			<thead>
				<tr><th></th><th class="text-right">Este mês</th><th class="text-right">Mês anterior</th><th class="text-right">Média 6 meses</th></tr>
			</thead>
			<tbody>
				<tr>
					<td>Receitas</td>
					<td class="text-right tabular-nums"><Money value={summary.income} /></td>
					<td class="text-right tabular-nums"><Money value={summary.previousIncome} /> <span class="text-xs text-stone-500">{delta(summary.income, summary.previousIncome)}</span></td>
					<td class="text-right tabular-nums"><Money value={summary.averageIncome} /> <span class="text-xs text-stone-500">{delta(summary.income, summary.averageIncome)}</span></td>
				</tr>
				<tr>
					<td>Despesas</td>
					<td class="text-right tabular-nums"><Money value={summary.outcome} /></td>
					<td class="text-right tabular-nums"><Money value={summary.previousOutcome} /> <span class="text-xs text-stone-500">{delta(summary.outcome, summary.previousOutcome)}</span></td>
					<td class="text-right tabular-nums"><Money value={summary.averageOutcome} /> <span class="text-xs text-stone-500">{delta(summary.outcome, summary.averageOutcome)}</span></td>
				</tr>
			</tbody>
		</table>
		</div>
	</section>

	<div class="mb-6 grid gap-4 xl:grid-cols-2">
		<section class="card">
			<h2 class="mb-3 font-medium">Categorias com maior gasto</h2>
			{#if summary.categories.filter((c) => c.current > 0).length === 0}
				<p class="muted">Sem despesas neste mês.</p>
			{:else}
				<HBarChart
					items={summary.categories.filter((c) => c.current > 0).map((c) => ({ label: c.name, value: c.current }))}
					format={(v) => formatMoney(v)}
				/>
			{/if}
		</section>
		<section class="card">
			<h2 class="mb-3 font-medium">Despesas e receitas previstas</h2>
			{#if planned.length === 0}
				<p class="muted">Nada previsto para o resto do mês.</p>
			{:else}
				<ul class="divide-y divide-stone-100 text-sm dark:divide-stone-800">
					{#each planned as item (item.key)}
						<li class="flex justify-between gap-3 py-1.5">
							<span><span class="text-stone-500">{formatDate(item.date)}</span> · {item.description}</span>
							<Amount value={item.amount} />
						</li>
					{/each}
				</ul>
			{/if}
		</section>
	</div>

	<section class="card">
		<h2 class="mb-3 font-medium">Por categoria</h2>
		{#if summary.categories.length === 0}
			<p class="muted">Sem despesas no período.</p>
		{:else}
			<div class="overflow-x-auto">
			<table class="table-base">
				<thead>
					<tr>
						<th>Categoria</th>
						<th class="text-right">Este mês</th>
						<th class="text-right">Mês anterior</th>
						<th class="text-right">Média</th>
						<th class="text-right">Diferença vs média</th>
					</tr>
				</thead>
				<tbody>
					{#each summary.categories as category (category.categoryId ?? category.name)}
						<tr>
							<td>{category.name}</td>
							<td class="text-right tabular-nums"><Money value={category.current} /></td>
							<td class="text-right tabular-nums"><Money value={category.previous} /></td>
							<td class="text-right tabular-nums"><Money value={category.average} /></td>
							<td class="text-right"><Amount value={category.average - category.current} /></td>
						</tr>
					{/each}
				</tbody>
			</table>
			</div>
			<p class="mt-2 text-xs text-stone-500">Diferença positiva significa que gastaste menos do que a média.</p>
		{/if}
	</section>
{:catch error}
	<States state="error" {error} />
{/await}
