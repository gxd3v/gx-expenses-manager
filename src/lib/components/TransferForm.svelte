<script lang="ts">
	import { untrack } from 'svelte';
	import { saveTransfer, type TransferInput } from '#lib/api/transactions.ts';
	import { errorMessage } from '#lib/graphql.ts';
	import { notify } from '#lib/toasts.svelte.ts';
	import AccountSelect from './AccountSelect.svelte';
	import MoneyInput from './MoneyInput.svelte';

	let {
		id = null,
		initial,
		onsaved,
		oncancel
	}: { id?: string | null; initial: TransferInput; onsaved: () => void; oncancel: () => void } = $props();

	let form = $state(untrack(() => ({ ...initial, amount: initial.amount || null })));
	let error = $state('');
	let busy = $state(false);

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		if (!form.amount || form.amount <= 0) {
			error = 'Indica um valor maior que zero.';
			return;
		}

		busy = true;
		try {
			await saveTransfer(id, { ...form, amount: form.amount });
			notify(id ? 'Transferência atualizada' : 'Transferência criada');
			onsaved();
		} catch (e) {
			error = errorMessage(e);
		} finally {
			busy = false;
		}
	}
</script>

<form onsubmit={submit} class="grid grid-cols-2 gap-4">
	<label class="label">
		De
		<AccountSelect bind:value={form.fromAccountId} />
	</label>
	<label class="label">
		Para
		<AccountSelect bind:value={form.toAccountId} exclude={form.fromAccountId} />
	</label>
	<label class="label">
		Valor
		<MoneyInput bind:value={form.amount} required />
	</label>
	<label class="label">
		Data
		<input type="date" bind:value={form.date} class="input" required />
	</label>
	<label class="label col-span-2">
		Descrição
		<input bind:value={form.description} class="input" placeholder="Opcional" />
	</label>

	<p class="col-span-2 muted">Transferências não contam como receita nem despesa.</p>

	{#if error}
		<p class="col-span-2 text-sm text-red-600" role="alert">{error}</p>
	{/if}

	<div class="col-span-2 flex justify-end gap-2">
		<button type="button" onclick={oncancel} class="btn-secondary">Cancelar</button>
		<button type="submit" disabled={busy} class="btn-primary">Guardar</button>
	</div>
</form>
