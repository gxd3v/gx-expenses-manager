<script lang="ts">
	import { listTransactions, type TransactionFilter } from '#lib/api/transactions.ts';
	import Amount from '#lib/components/Amount.svelte';
	import Modal from '#lib/components/Modal.svelte';
	import States from '#lib/components/States.svelte';
	import { formatDate } from '#lib/format.ts';

	const LIMIT = 500;

	let { title, filter, onclose }: { title: string; filter: TransactionFilter; onclose: () => void } = $props();

	const request = $derived(listTransactions(filter, LIMIT));
</script>

<Modal {title} {onclose} wide>
	{#await request}
		<States state="loading" />
	{:then page}
		<div class="mb-3 flex flex-wrap gap-x-6 gap-y-1 text-sm">
			<span><span class="muted">Entradas:</span> <Amount value={page.income} /></span>
			<span><span class="muted">Saídas:</span> <Amount value={-page.outcome} /></span>
			<span><span class="muted">Total:</span> <Amount value={page.net} /></span>
			<span class="muted">{page.totalCount} movimentos</span>
		</div>
		{#if page.items.length === 0}
			<p class="muted">Sem movimentos.</p>
		{:else}
			<ul class="max-h-[60vh] divide-y divide-stone-100 overflow-y-auto text-sm dark:divide-stone-800">
				{#each page.items as item (item.id)}
					<li class="flex items-center justify-between gap-3 py-1.5">
						<span class="min-w-0">
							<span class="block truncate">{item.description || '—'}</span>
							<span class="text-xs text-stone-500">{formatDate(item.date)} · {item.categoryName ?? 'Sem categoria'} · {item.accountName}</span>
						</span>
						<Amount value={item.amount} currency={item.currency} />
					</li>
				{/each}
			</ul>
			{#if page.totalCount > page.items.length}
				<p class="mt-2 text-xs text-stone-500">A mostrar os {page.items.length} mais recentes.</p>
			{/if}
		{/if}
	{:catch error}
		<States state="error" {error} />
	{/await}
</Modal>
