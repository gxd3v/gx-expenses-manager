<script lang="ts">
	import { untrack } from 'svelte';
	import { frequencyPresets, saveRecurrence, unitLabels, type Recurrence, type RecurrenceInput } from '#lib/api/recurrences.ts';
	import AccountSelect from '#lib/components/AccountSelect.svelte';
	import CategorySelect from '#lib/components/CategorySelect.svelte';
	import MoneyInput from '#lib/components/MoneyInput.svelte';
	import Toggle from '#lib/components/Toggle.svelte';
	import { today } from '#lib/format.ts';
	import { errorMessage } from '#lib/graphql.ts';
	import { refs } from '#lib/refs.svelte.ts';
	import { notify } from '#lib/toasts.svelte.ts';

	let { recurrence, onsaved, oncancel }: { recurrence: Recurrence | null; onsaved: () => void; oncancel: () => void } = $props();

	let form = $state<Omit<RecurrenceInput, 'amount'> & { amount: number | null }>(
		untrack(() => ({
			accountId: recurrence?.accountId ?? refs.accounts[0]?.id ?? '',
			categoryId: recurrence?.categoryId ?? null,
			kind: recurrence?.kind ?? 'OUTCOME',
			amount: recurrence?.amount ?? null,
			description: recurrence?.description ?? '',
			startDate: recurrence?.startDate ?? today(),
			endDate: recurrence?.endDate ?? null,
			unit: recurrence?.unit ?? 'MONTH',
			interval: recurrence?.interval ?? 1,
			toAccountId: recurrence?.toAccountId ?? null,
			variableAmount: recurrence?.variableAmount ?? false
		}))
	);
	let type = $state<'OUTCOME' | 'INCOME' | 'TRANSFER'>(untrack(() => (recurrence?.toAccountId ? 'TRANSFER' : (recurrence?.kind ?? 'OUTCOME'))));
	const transfer = $derived(type === 'TRANSFER');

	function chooseType() {
		form.categoryId = null;
		form.kind = type === 'INCOME' ? 'INCOME' : 'OUTCOME';
		if (!transfer) form.toAccountId = null;
		if (transfer) form.variableAmount = false;
	}
	let preset = $state(
		untrack(() => {
			const index = frequencyPresets.findIndex((p) => p.unit === form.unit && p.interval === form.interval);
			return index >= 0 ? String(index) : 'custom';
		})
	);
	let error = $state('');

	function choosePreset() {
		if (preset === 'custom') return;
		const chosen = frequencyPresets[Number(preset)];
		form.unit = chosen.unit;
		form.interval = chosen.interval;
	}

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		if (!form.amount || form.amount <= 0) {
			error = 'Indica um valor maior que zero.';
			return;
		}
		if (transfer && !form.toAccountId) {
			error = 'Escolhe a conta de destino.';
			return;
		}
		try {
			await saveRecurrence(recurrence?.id ?? null, { ...form, amount: form.amount, endDate: form.endDate || null });
			notify('Recorrência guardada');
			onsaved();
		} catch (e) {
			error = errorMessage(e);
		}
	}
</script>

<form onsubmit={submit} class="grid grid-cols-2 gap-4">
	<label class="label">
		Tipo
		<select bind:value={type} class="input" onchange={chooseType}>
			<option value="OUTCOME">Despesa</option>
			<option value="INCOME">Receita</option>
			<option value="TRANSFER">Transferência</option>
		</select>
	</label>
	<label class="label">
		Valor
		<MoneyInput bind:value={form.amount} required />
	</label>
	<label class="label col-span-2">
		Descrição
		<input bind:value={form.description} class="input" required placeholder="Ex.: Renda" />
	</label>
	<label class="label">
		{transfer ? 'Conta de origem' : 'Conta'}
		<AccountSelect bind:value={form.accountId} />
	</label>
	{#if transfer}
		<label class="label">
			Conta de destino
			<AccountSelect bind:value={form.toAccountId} exclude={form.accountId} allowEmpty emptyLabel="Escolher…" />
		</label>
	{:else}
		<label class="label">
			Categoria
			<CategorySelect bind:value={form.categoryId} kind={form.kind} />
		</label>
	{/if}
	<label class="label">
		Periodicidade
		<select bind:value={preset} onchange={choosePreset} class="input">
			{#each frequencyPresets as option, index (option.label)}
				<option value={String(index)}>{option.label}</option>
			{/each}
			<option value="custom">Personalizada</option>
		</select>
	</label>
	{#if preset === 'custom'}
		<div class="grid grid-cols-2 gap-2">
			<label class="label">
				A cada
				<input type="number" min="1" bind:value={form.interval} class="input" />
			</label>
			<label class="label">
				Unidade
				<select bind:value={form.unit} class="input">
					{#each Object.entries(unitLabels) as [value, label] (value)}
						<option {value}>{label}</option>
					{/each}
				</select>
			</label>
		</div>
	{:else}
		<div></div>
	{/if}
	<label class="label">
		Data inicial
		<input type="date" bind:value={form.startDate} class="input" required />
	</label>
	<label class="label">
		Data final (opcional)
		<input type="date" bind:value={form.endDate} class="input" />
	</label>
	{#if !transfer}
		<div class="col-span-2">
			<Toggle bind:checked={form.variableAmount} label="Valor variável — pedir para confirmar o valor real quando acontecer" />
		</div>
	{/if}
	<p class="col-span-2 muted">
		As ocorrências passam a movimentos reais quando chega a data (só a partir do dia em que a recorrência é criada). As futuras
		entram nas previsões.
	</p>

	{#if error}<p class="col-span-2 text-sm text-red-600" role="alert">{error}</p>{/if}

	<div class="col-span-2 flex justify-end gap-2">
		<button type="button" onclick={oncancel} class="btn-secondary">Cancelar</button>
		<button type="submit" class="btn-primary">Guardar</button>
	</div>
</form>
