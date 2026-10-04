<script lang="ts">
	import { balanceRecords, type BalanceMark, type RecordPeriod } from '#lib/api/reports.ts';
	import AccountSelect from '#lib/components/AccountSelect.svelte';
	import Money from '#lib/components/Money.svelte';
	import States from '#lib/components/States.svelte';
	import { formatDate } from '#lib/format.ts';
	import { refs } from '#lib/refs.svelte.ts';

	const periods: [RecordPeriod, string][] = [
		['ALL_TIME', 'Sempre'],
		['YEAR', 'Ano'],
		['MONTH', 'Mês'],
		['WEEK', 'Semana']
	];

	let {
		accountId = null,
		currency,
		selectable = false
	}: { accountId?: string | null; currency?: string; selectable?: boolean } = $props();

	let selected = $state<string | null>(null);
	let period = $state<RecordPeriod>('ALL_TIME');

	$effect.pre(() => {
		selected = accountId;
	});

	const request = $derived.by(() => {
		refs.version;
		return balanceRecords(selected);
	});
</script>

{#snippet mark(label: string, icon: string, tone: string, value: BalanceMark)}
	<div class="flex items-baseline justify-between gap-3">
		<dt class="flex items-center gap-2 text-stone-600 dark:text-stone-400"><span class={tone}>{icon}</span>{label}</dt>
		<dd class="text-right">
			<span class="text-lg font-semibold"><Money value={value.balance} {currency} /></span>
			<span class="block text-xs text-stone-500">{formatDate(value.date)}</span>
		</dd>
	</div>
{/snippet}

<section class="card">
	<div class="mb-3 flex flex-wrap items-center justify-between gap-2">
		<h2 class="font-medium">Máximos e mínimos de saldo</h2>
		{#if selectable}
			<div class="w-44">
				<AccountSelect bind:value={selected} allowEmpty emptyLabel="Saldo total" />
			</div>
		{/if}
	</div>
	<div class="mb-4 inline-flex rounded-lg bg-stone-100 p-0.5 text-xs dark:bg-stone-800" role="group" aria-label="Período">
		{#each periods as [key, label] (key)}
			<button
				class="rounded-md px-3 py-1 transition-colors {period === key
					? 'bg-white font-medium shadow-sm dark:bg-stone-700'
					: 'text-stone-600 hover:text-stone-900 dark:text-stone-400 dark:hover:text-stone-100'}"
				aria-pressed={period === key}
				onclick={() => (period = key)}>{label}</button
			>
		{/each}
	</div>
	{#await request}
		<States state="loading" />
	{:then records}
		{@const record = records.find((r) => r.period === period)}
		{#if record}
			<dl class="space-y-3 text-sm">
				{@render mark('Máximo', '▲', 'text-emerald-600 dark:text-emerald-400', record.high)}
				{@render mark('Mínimo', '▼', 'text-red-600 dark:text-red-400', record.low)}
			</dl>
		{/if}
	{:catch error}
		<States state="error" {error} />
	{/await}
</section>
