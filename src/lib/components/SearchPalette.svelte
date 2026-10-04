<script lang="ts">
	import { goto } from '$app/navigation';
	import { search, type SearchResult } from '#lib/api/search.ts';
	import { formatDate, formatMoney, formatPercent } from '#lib/format.ts';
	import Modal from './Modal.svelte';

	let { onclose }: { onclose: () => void } = $props();

	let text = $state('');
	let result = $state<SearchResult | null>(null);
	let timer: ReturnType<typeof setTimeout>;

	function input() {
		clearTimeout(timer);
		timer = setTimeout(async () => {
			result = text.trim() ? await search(text) : null;
		}, 200);
	}

	function open(path: string) {
		onclose();
		goto(path);
	}

	const total = $derived(
		result
			? result.accounts.length +
					result.categories.length +
					result.transactions.length +
					result.credits.length +
					result.goals.length
			: 0
	);
</script>

<Modal title="Pesquisa" {onclose} wide>
	<input bind:value={text} oninput={input} class="input" placeholder="Contas, movimentos, categorias, créditos, objetivos…" />

	{#if result && total === 0}
		<p class="mt-4 muted">Sem resultados.</p>
	{:else if result}
		<div class="mt-4 flex max-h-[60vh] flex-col gap-4 overflow-y-auto">
			{#if result.accounts.length}
				<section>
					<h3 class="mb-1 text-xs font-medium text-stone-500 uppercase">Contas</h3>
					{#each result.accounts as account (account.id)}
						<button class="flex w-full justify-between rounded px-2 py-1.5 text-left hover:bg-stone-100 dark:hover:bg-stone-800" onclick={() => open(`/accounts/${account.id}`)}>
							<span>{account.name}</span><span class="tabular-nums">{formatMoney(account.balance, account.currency)}</span>
						</button>
					{/each}
				</section>
			{/if}
			{#if result.transactions.length}
				<section>
					<h3 class="mb-1 text-xs font-medium text-stone-500 uppercase">Movimentos</h3>
					{#each result.transactions as transaction (transaction.id)}
						<button
							class="flex w-full justify-between gap-4 rounded px-2 py-1.5 text-left hover:bg-stone-100 dark:hover:bg-stone-800"
							onclick={() => open(`/transactions?search=${encodeURIComponent(transaction.description)}`)}
						>
							<span class="truncate">{formatDate(transaction.date)} · {transaction.description || '—'} · {transaction.accountName}</span>
							<span class="tabular-nums">{formatMoney(transaction.amount, transaction.currency)}</span>
						</button>
					{/each}
				</section>
			{/if}
			{#if result.categories.length}
				<section>
					<h3 class="mb-1 text-xs font-medium text-stone-500 uppercase">Categorias</h3>
					{#each result.categories as category (category.id)}
						<button class="w-full rounded px-2 py-1.5 text-left hover:bg-stone-100 dark:hover:bg-stone-800" onclick={() => open(`/transactions?category=${category.id}`)}>
							{category.name}
						</button>
					{/each}
				</section>
			{/if}
			{#if result.credits.length}
				<section>
					<h3 class="mb-1 text-xs font-medium text-stone-500 uppercase">Créditos</h3>
					{#each result.credits as credit (credit.id)}
						<button class="flex w-full justify-between rounded px-2 py-1.5 text-left hover:bg-stone-100 dark:hover:bg-stone-800" onclick={() => open(`/credits/${credit.id}`)}>
							<span>{credit.name}</span><span class="tabular-nums">{formatMoney(credit.remaining)}</span>
						</button>
					{/each}
				</section>
			{/if}
			{#if result.goals.length}
				<section>
					<h3 class="mb-1 text-xs font-medium text-stone-500 uppercase">Objetivos</h3>
					{#each result.goals as goal (goal.id)}
						<button class="flex w-full justify-between rounded px-2 py-1.5 text-left hover:bg-stone-100 dark:hover:bg-stone-800" onclick={() => open('/goals')}>
							<span>{goal.name}</span><span>{formatPercent(goal.progress)}</span>
						</button>
					{/each}
				</section>
			{/if}
		</div>
	{/if}
</Modal>
