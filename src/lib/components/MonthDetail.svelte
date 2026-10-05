<script lang="ts" module>
	export type Totals = { realized: number; planned: number; expected: number };
</script>

<script lang="ts">
	import { listTransactions } from '#lib/api/transactions.ts';
	import Amount from '#lib/components/Amount.svelte';
	import Modal from '#lib/components/Modal.svelte';
	import States from '#lib/components/States.svelte';
	import { formatDate, today } from '#lib/format.ts';
	import { dismissPlanned, type Planned } from '#lib/planned.ts';

	const LIMIT = 500;

	let {
		title,
		sign,
		start,
		end,
		planned,
		totals,
		onclose
	}: {
		title: string;
		sign: 1 | -1 | 0;
		start: string;
		end: string;
		planned: Planned[];
		totals: Totals;
		onclose: () => void;
	} = $props();

	const matches = (amount: number) => sign === 0 || Math.sign(amount) === sign;
	const realizedEnd = $derived(end < today() ? end : today());

	const realized = $derived.by(async () => {
		if (start > realizedEnd) return [];
		const page = await listTransactions({ dateFrom: start, dateTo: realizedEnd }, LIMIT);
		return page.items.filter((t) => t.kind !== 'TRANSFER' && matches(t.amount));
	});

	const upcoming = $derived(planned.filter((p) => matches(p.amount)));
</script>

{#snippet section(
	name: string,
	total: number,
	items: { key: string; date: string; description: string; amount: number; detail?: string; planned?: Planned }[],
	empty: string
)}
	<section>
		<h3 class="mb-1 flex justify-between text-sm font-medium">
			<span>{name}</span>
			<Amount value={total} />
		</h3>
		{#if items.length === 0}
			<p class="muted">{empty}</p>
		{:else}
			<ul class="max-h-[30vh] divide-y divide-stone-100 overflow-y-auto text-sm dark:divide-stone-800">
				{#each items as item (item.key)}
					<li class="flex items-center justify-between gap-3 py-1.5">
						<span class="min-w-0">
							<span class="block truncate">{item.description || '—'}</span>
							<span class="text-xs text-stone-500">{formatDate(item.date)}{item.detail ? ` · ${item.detail}` : ''}</span>
						</span>
						<span class="flex shrink-0 items-center gap-1">
							<Amount value={item.amount} />
							{#if item.planned}
								{@const planned = item.planned}
								<button
									class="rounded p-1 text-xs leading-none text-stone-400 hover:bg-stone-100 hover:text-red-600 dark:hover:bg-stone-800"
									onclick={() => dismissPlanned(planned)}
									title="Não vai acontecer"
									aria-label="Remover movimento previsto">✕</button
								>
							{/if}
						</span>
					</li>
				{/each}
			</ul>
		{/if}
	</section>
{/snippet}

<Modal {title} {onclose} wide>
	<div class="space-y-5">
		{#await realized}
			<States state="loading" />
		{:then items}
			{@render section(
				'Realizado',
				totals.realized,
				items.map((t) => ({ key: t.id, date: t.date, description: t.description, amount: t.amount, detail: t.categoryName ?? 'Sem categoria' })),
				'Sem movimentos realizados.'
			)}
		{:catch error}
			<States state="error" {error} />
		{/await}
		{@render section(
			'Previsto até ao fim do mês',
			totals.planned,
			upcoming.map((p) => ({ ...p, planned: p })),
			'Nada previsto.'
		)}
		<p class="flex justify-between border-t border-stone-200 pt-2 text-sm font-semibold dark:border-stone-800">
			<span>Total esperado</span>
			<Amount value={totals.expected} />
		</p>
		<p class="text-xs text-stone-500">
			O previsto inclui as ocorrências das recorrências e os movimentos já agendados até ao fim do mês. Transferências entre contas não
			contam.
		</p>
	</div>
</Modal>
