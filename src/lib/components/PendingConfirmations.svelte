<script lang="ts">
	import { pendingConfirmations, saveTransaction, setConfirmed, type Transaction } from '#lib/api/transactions.ts';
	import type { EntryKind } from '#lib/api/transactions.ts';
	import Amount from '#lib/components/Amount.svelte';
	import MoneyInput from '#lib/components/MoneyInput.svelte';
	import { formatDate } from '#lib/format.ts';
	import { dataChanged, refs } from '#lib/refs.svelte.ts';
	import { notify, notifyError } from '#lib/toasts.svelte.ts';

	let editing = $state<string | null>(null);
	let amount = $state<number | null>(null);

	const request = $derived.by(() => {
		refs.version;
		return pendingConfirmations();
	});

	async function confirm(transaction: Transaction) {
		try {
			await setConfirmed([transaction.id], true);
			notify('Valor confirmado');
			await dataChanged();
		} catch (e) {
			notifyError(e);
		}
	}

	function edit(transaction: Transaction) {
		editing = transaction.id;
		amount = Math.abs(transaction.amount);
	}

	async function save(transaction: Transaction) {
		if (!amount || amount <= 0) return;
		try {
			await saveTransaction(transaction.id, {
				accountId: transaction.accountId,
				categoryId: transaction.categoryId,
				kind: transaction.kind as EntryKind,
				amount,
				date: transaction.date,
				description: transaction.description,
				notes: transaction.notes,
				confirmed: true,
				oneOff: transaction.oneOff
			});
			editing = null;
			notify('Valor atualizado');
			await dataChanged();
		} catch (e) {
			notifyError(e);
		}
	}
</script>

{#await request then items}
	{#if items.length}
		<section class="mb-6 rounded-xl border border-indigo-200 bg-indigo-50 p-4 dark:border-indigo-900 dark:bg-indigo-950" aria-label="Ações pendentes">
			<h2 class="mb-2 text-sm font-medium text-indigo-900 dark:text-indigo-200">Ações pendentes · confirmação de valor</h2>
			<ul class="divide-y divide-indigo-100 text-sm dark:divide-indigo-900">
				{#each items as transaction (transaction.id)}
					<li class="flex flex-wrap items-center justify-between gap-3 py-2">
						<span>
							<strong>{transaction.description}</strong>
							<span class="text-stone-500">· {formatDate(transaction.date)} · {transaction.accountName}</span>
						</span>
						{#if editing === transaction.id}
							<span class="flex items-center gap-2">
								<span class="w-32"><MoneyInput bind:value={amount} /></span>
								<button class="btn-primary" onclick={() => save(transaction)}>Guardar</button>
								<button class="btn-ghost" onclick={() => (editing = null)}>Cancelar</button>
							</span>
						{:else}
							<span class="flex items-center gap-2">
								<Amount value={transaction.amount} />
								<button class="btn-primary" onclick={() => confirm(transaction)}>Confirmar</button>
								<button class="btn-secondary" onclick={() => edit(transaction)}>Outro valor</button>
							</span>
						{/if}
					</li>
				{/each}
			</ul>
		</section>
	{/if}
{/await}
