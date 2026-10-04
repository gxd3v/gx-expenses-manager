<script lang="ts">
	import { untrack } from 'svelte';
	import { saveCredit, type Credit } from '#lib/api/credits.ts';
	import { frequencyPresets } from '#lib/api/recurrences.ts';
	import AccountSelect from '#lib/components/AccountSelect.svelte';
	import MoneyInput from '#lib/components/MoneyInput.svelte';
	import Toggle from '#lib/components/Toggle.svelte';
	import { today } from '#lib/format.ts';
	import { errorMessage } from '#lib/graphql.ts';
	import { notify } from '#lib/toasts.svelte.ts';

	let { credit, onsaved, oncancel }: { credit: Credit | null; onsaved: () => void; oncancel: () => void } = $props();

	let form = $state(
		untrack(() => ({
			name: credit?.name ?? '',
			institution: credit?.institution ?? '',
			principal: credit?.principal ?? null as number | null,
			openingBalance: credit?.openingBalance ?? null as number | null,
			annualRate: credit?.annualRate ?? 0,
			installment: credit?.installment ?? null as number | null,
			frequency: String(Math.max(0, frequencyPresets.findIndex((p) => p.unit === (credit?.unit ?? 'MONTH') && p.interval === (credit?.interval ?? 1)))),
			startDate: credit?.startDate ?? today(),
			endDate: credit?.endDate ?? '',
			installments: credit?.installments ?? null as number | null,
			accountId: credit?.accountId ?? null as string | null
		}))
	);
	let createRecurrence = $state(true);
	let error = $state('');

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		if (!form.principal || !form.installment) {
			error = 'Capital inicial e prestação obrigatórios.';
			return;
		}
		const preset = frequencyPresets[Number(form.frequency)];
		try {
			await saveCredit(
				credit?.id ?? null,
				{
					name: form.name,
					institution: form.institution || null,
					principal: form.principal,
					openingBalance: form.openingBalance ?? form.principal,
					annualRate: Number(form.annualRate),
					installment: form.installment,
					unit: preset.unit,
					interval: preset.interval,
					startDate: form.startDate,
					endDate: form.endDate || null,
					installments: form.installments || null,
					accountId: form.accountId
				},
				!credit && createRecurrence && form.accountId !== null
			);
			notify('Crédito guardado');
			onsaved();
		} catch (e) {
			error = errorMessage(e);
		}
	}
</script>

<form onsubmit={submit} class="grid grid-cols-2 gap-4">
	<label class="label">
		Nome
		<input bind:value={form.name} class="input" required placeholder="Ex.: Crédito habitação" />
	</label>
	<label class="label">
		Entidade
		<input bind:value={form.institution} class="input" placeholder="Banco" />
	</label>
	<label class="label">
		Capital inicial
		<MoneyInput bind:value={form.principal} required />
	</label>
	<label class="label">
		Capital em dívida hoje
		<MoneyInput bind:value={form.openingBalance} placeholder="Igual ao inicial" />
	</label>
	<label class="label">
		Taxa de juro anual (%)
		<input type="number" step="0.001" min="0" max="100" bind:value={form.annualRate} class="input" />
	</label>
	<label class="label">
		Prestação
		<MoneyInput bind:value={form.installment} required />
	</label>
	<label class="label">
		Periodicidade
		<select bind:value={form.frequency} class="input">
			{#each frequencyPresets as option, index (option.label)}
				<option value={String(index)}>{option.label}</option>
			{/each}
		</select>
	</label>
	<label class="label">
		Número de prestações
		<input type="number" min="1" bind:value={form.installments} class="input" />
	</label>
	<label class="label">
		Data inicial
		<input type="date" bind:value={form.startDate} class="input" required />
	</label>
	<label class="label">
		Data prevista de fim
		<input type="date" bind:value={form.endDate} class="input" />
	</label>
	<label class="label col-span-2">
		Conta de pagamento
		<AccountSelect bind:value={form.accountId} allowEmpty emptyLabel="Nenhuma" />
	</label>
	{#if !credit}
		<div class="col-span-2">
			<Toggle
				bind:checked={createRecurrence}
				disabled={!form.accountId}
				label="Criar recorrência para a prestação (regista os pagamentos automaticamente)"
			/>
		</div>
	{/if}

	{#if error}<p class="col-span-2 text-sm text-red-600" role="alert">{error}</p>{/if}

	<div class="col-span-2 flex justify-end gap-2">
		<button type="button" onclick={oncancel} class="btn-secondary">Cancelar</button>
		<button type="submit" class="btn-primary">Guardar</button>
	</div>
</form>
