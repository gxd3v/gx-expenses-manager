<script lang="ts">
	import { listOccurrences, type FrequencyUnit, type Recurrence } from '#lib/api/recurrences.ts';
	import Amount from '#lib/components/Amount.svelte';
	import { formatDate, today } from '#lib/format.ts';
	import { refs } from '#lib/refs.svelte.ts';

	type Totals = { income: number; outcome: number; transfers: number };
	type Entry = { kind: string; amount: number; accountId: string; toAccountId: string | null };

	const ROWS: { label: string; key: keyof Totals }[] = [
		{ label: 'Entradas', key: 'income' },
		{ label: 'Saídas', key: 'outcome' },
		{ label: 'Transferências entre contas', key: 'transfers' }
	];
	const PER_MONTH: Record<FrequencyUnit, number> = { DAY: 365 / 12, WEEK: 52 / 12, MONTH: 1, YEAR: 0 };

	let { recurrences }: { recurrences: Recurrence[] } = $props();

	const yearEnd = `${today().slice(0, 4)}-12-31`;
	const ignored = $derived(new Set(refs.accounts.filter((a) => a.kind === 'MEAL').map((a) => a.id)));

	function sum(entries: Entry[]): Totals {
		const totals = { income: 0, outcome: 0, transfers: 0 };
		for (const entry of entries) {
			if (ignored.has(entry.accountId)) continue;
			if (entry.toAccountId) totals.transfers += entry.amount;
			else if (entry.kind === 'INCOME') totals.income += entry.amount;
			else totals.outcome += entry.amount;
		}
		return totals;
	}

	const active = (r: Recurrence) => !r.pausedAt && (!r.endDate || r.endDate >= today());

	const month = $derived(
		sum(recurrences.filter(active).map((r) => ({ ...r, amount: Math.round((r.amount * PER_MONTH[r.unit]) / r.interval) })))
	);

	let year = $state<Totals | null>(null);

	$effect(() => {
		void recurrences;
		listOccurrences(today(), yearEnd)
			.then((occurrences) => (year = sum(occurrences)))
			.catch(() => (year = null));
	});

	const columns = $derived([
		{ label: 'Mês típico', totals: month },
		{ label: `Até ${formatDate(yearEnd)}`, totals: year }
	]);
</script>

<div class="card mt-4 overflow-x-auto p-0 md:p-0">
	<table class="table-base">
		<thead>
			<tr>
				<th></th>
				{#each columns as column (column.label)}<th class="text-right">{column.label}</th>{/each}
			</tr>
		</thead>
		<tbody>
			{#each ROWS as row (row.key)}
				<tr>
					<td>{row.label}</td>
					{#each columns as column (column.label)}
						<td class="text-right">
							{#if column.totals}<Amount value={row.key === 'income' ? column.totals.income : -column.totals[row.key]} />{:else}—{/if}
						</td>
					{/each}
				</tr>
			{/each}
			<tr class="font-semibold">
				<td>Sobra</td>
				{#each columns as column (column.label)}
					<td class="text-right">
						{#if column.totals}<Amount value={column.totals.income - column.totals.outcome - column.totals.transfers} />{:else}—{/if}
					</td>
				{/each}
			</tr>
		</tbody>
	</table>
	<p class="px-3 py-2 text-xs text-stone-500">
		O mês típico converte as recorrências ativas para o valor por mês, sem as anuais. O total até ao fim do ano conta as ocorrências
		por registar a partir de hoje, incluindo anuais e alterações. Contas de refeição ficam de fora.
	</p>
</div>
