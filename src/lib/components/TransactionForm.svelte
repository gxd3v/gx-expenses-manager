<script lang="ts">
	import { untrack } from 'svelte';
	import { saveTransaction, type TransactionInput } from '#lib/api/transactions.ts';
	import { errorMessage } from '#lib/graphql.ts';
	import { notify } from '#lib/toasts.svelte.ts';
	import AccountSelect from './AccountSelect.svelte';
	import CategorySelect from './CategorySelect.svelte';
	import MoneyInput from './MoneyInput.svelte';
	import Toggle from './Toggle.svelte';

	let {
		id = null,
		initial,
		onsaved,
		oncancel
	}: { id?: string | null; initial: TransactionInput; onsaved: () => void; oncancel: () => void } = $props();

	let form = $state(untrack(() => ({ ...initial, amount: initial.amount || null })));
	let error = $state('');
	let busy = $state(false);

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		if (!form.amount || form.amount <= 0) {
			error = 'Indica um valor maior que zero.';
			return;
		}
		if (!form.accountId) {
			error = 'Escolhe uma conta.';
			return;
		}

		busy = true;
		try {
			await saveTransaction(id, { ...form, amount: form.amount, notes: form.notes || null });
			notify(id ? 'Movimento atualizado' : 'Movimento criado');
			onsaved();
		} catch (e) {
			error = errorMessage(e);
		} finally {
			busy = false;
		}
	}

	function setKind(kind: TransactionInput['kind']) {
		form.kind = kind;
		form.categoryId = null;
	}
</script>

<form onsubmit={submit} class="grid grid-cols-2 gap-4">
	<div class="col-span-2 flex rounded-md border border-stone-300 p-0.5 dark:border-stone-700" role="radiogroup" aria-label="Tipo">
		{#each [['OUTCOME', 'Despesa'], ['INCOME', 'Receita']] as const as [kind, label] (kind)}
			<button
				type="button"
				role="radio"
				aria-checked={form.kind === kind}
				class="flex-1 rounded py-1 text-sm {form.kind === kind ? 'bg-indigo-600 text-white' : ''}"
				onclick={() => setKind(kind)}
			>
				{label}
			</button>
		{/each}
	</div>

	<label class="label">
		Valor
		<MoneyInput bind:value={form.amount} required />
	</label>
	<label class="label">
		Data
		<input type="date" bind:value={form.date} class="input" required />
	</label>
	<label class="label">
		Conta
		<AccountSelect bind:value={form.accountId} />
	</label>
	<label class="label">
		Categoria
		<CategorySelect bind:value={form.categoryId} kind={form.kind} />
	</label>
	<label class="label col-span-2">
		Descrição
		<input bind:value={form.description} class="input" placeholder="Ex.: Supermercado" />
	</label>
	<label class="label col-span-2">
		Notas
		<textarea bind:value={form.notes} rows="2" class="input"></textarea>
	</label>
	<div class="col-span-2">
		<Toggle bind:checked={form.confirmed} label="Confirmado no banco" />
	</div>
	<div class="col-span-2">
		<Toggle bind:checked={form.oneOff} label="Pontual — não entra nas médias nem nas previsões" />
	</div>

	{#if error}
		<p class="col-span-2 text-sm text-red-600" role="alert">{error}</p>
	{/if}

	<div class="col-span-2 flex justify-end gap-2">
		<button type="button" onclick={oncancel} class="btn-secondary">Cancelar</button>
		<button type="submit" disabled={busy} class="btn-primary">Guardar</button>
	</div>
</form>
