<script lang="ts">
	import type { Goal } from '#lib/api/goals.ts';
	import { balanceHistory } from '#lib/api/reports.ts';
	import LineChart from '#lib/charts/LineChart.svelte';
	import States from '#lib/components/States.svelte';
	import { formatMoney, formatMonth } from '#lib/format.ts';

	const MONTHS = 12;

	let { goal }: { goal: Goal } = $props();

	const request = $derived(balanceHistory(MONTHS, goal.accountId));
</script>

{#await request}
	<States state="loading" />
{:then history}
	<LineChart
		labels={history.map((p) => formatMonth(p.month))}
		series={[
			{ name: 'Valor atual', color: 'var(--series-1)', values: history.map((p) => p.balance) },
			{ name: 'Objetivo', color: 'var(--series-3)', values: history.map(() => goal.targetAmount) }
		]}
		format={(v) => formatMoney(v)}
		height={180}
	/>
{:catch error}
	<States state="error" {error} />
{/await}
