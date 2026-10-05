<script lang="ts">
	import { untrack } from 'svelte';
	import { accountKinds, interestPeriods, saveAccount, type Account } from '#lib/api/accounts.ts';
	import IconPicker from '#lib/components/IconPicker.svelte';
	import MoneyInput from '#lib/components/MoneyInput.svelte';
	import Toggle from '#lib/components/Toggle.svelte';
	import { errorMessage } from '#lib/graphql.ts';
	import { app } from '#lib/settings.svelte.ts';
	import { notify } from '#lib/toasts.svelte.ts';

	let { account, onsaved, oncancel }: { account: Account | null; onsaved: () => void; oncancel: () => void } = $props();

	let form = $state(
		untrack(() => ({
			name: account?.name ?? '',
			kind: account?.kind ?? 'BANK',
			currency: account?.currency ?? app.settings?.currency ?? 'EUR',
			initialBalance: account?.initialBalance ?? 0 as number | null,
			color: account?.color ?? '#2a78d6',
			icon: account?.icon ?? '',
			overdraftLimit: (account?.overdraftLimit ?? 0) as number | null
		}))
	);
	let earnsInterest = $state(untrack(() => account?.interest !== null && account?.interest !== undefined));
	let periodMonths = $state(untrack(() => account?.interest?.periodMonths ?? 12));
	let tiers = $state(
		untrack(() =>
			(account?.interest?.tiers ?? [{ minBalance: 0, rate: 0 }]).map((t) => ({
				minBalance: t.minBalance as number | null,
				rate: String(t.rate).replace('.', ',')
			}))
		)
	);
	let error = $state('');
	let busy = $state(false);

	function interest() {
		if (!earnsInterest) return null;
		return {
			periodMonths,
			tiers: tiers.map((t) => ({ minBalance: t.minBalance ?? 0, rate: Number(t.rate.replace(',', '.')) }))
		};
	}

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		busy = true;
		try {
			await saveAccount(account?.id ?? null, {
				...form,
				initialBalance: form.initialBalance ?? 0,
				icon: form.icon || null,
				overdraftLimit: form.overdraftLimit ?? 0,
				interest: interest()
			});
			notify(account ? 'Conta atualizada' : 'Conta criada');
			onsaved();
		} catch (e) {
			error = errorMessage(e);
		} finally {
			busy = false;
		}
	}
</script>

<form onsubmit={submit} class="grid grid-cols-2 gap-4">
	<label class="label col-span-2">
		Nome
		<input bind:value={form.name} class="input" required />
	</label>
	<label class="label">
		Tipo
		<select bind:value={form.kind} class="input">
			{#each Object.entries(accountKinds) as [value, label] (value)}
				<option {value}>{label}</option>
			{/each}
		</select>
	</label>
	<label class="label">
		Moeda
		<input bind:value={form.currency} maxlength="3" class="input uppercase" required />
	</label>
	<label class="label">
		Saldo inicial
		<MoneyInput bind:value={form.initialBalance} />
	</label>
	<label class="label">
		{form.kind === 'CREDIT_CARD' ? 'Plafond' : 'Descoberto autorizado'}
		<MoneyInput bind:value={form.overdraftLimit} />
	</label>
	{#if form.kind === 'CREDIT_CARD'}
		<p class="col-span-2 muted">O saldo dos cartões de crédito não conta para o saldo total; o valor em dívida entra no património líquido.</p>
	{:else if form.kind === 'MEAL'}
		<p class="col-span-2 muted">O saldo do cartão refeição não conta para o saldo total, e os seus movimentos ficam fora das previsões e das estimativas.</p>
	{/if}
	<div class="grid grid-cols-2 gap-2">
		<label class="label">
			Cor
			<input type="color" bind:value={form.color} class="h-9 w-full rounded-md" />
		</label>
		<div class="label">
			Ícone
			<IconPicker bind:value={form.icon} />
		</div>
	</div>

	<div class="col-span-2">
		<Toggle bind:checked={earnsInterest} label="Rende juros" />
	</div>
	{#if earnsInterest}
		<div class="col-span-2 space-y-3 rounded-lg border border-stone-200 p-3 dark:border-stone-800">
			<label class="label">
				Vencimento dos juros
				<select bind:value={periodMonths} class="input">
					{#each Object.entries(interestPeriods) as [months, label] (months)}
						<option value={Number(months)}>{label}</option>
					{/each}
				</select>
			</label>
			<div class="space-y-2">
				<p class="text-sm text-stone-600 dark:text-stone-400">Escalões (TANB)</p>
				{#each tiers as tier, index (index)}
					<div class="flex items-center gap-2 text-sm">
						<span class="text-stone-500">A partir de</span>
						<span class="w-32"><MoneyInput bind:value={tier.minBalance} /></span>
						<span class="w-24"><input bind:value={tier.rate} inputmode="decimal" class="input text-right" aria-label="Taxa" /></span>
						<span>%</span>
						{#if tiers.length > 1}
							<button type="button" class="btn-ghost" onclick={() => tiers.splice(index, 1)} aria-label="Remover escalão">✕</button>
						{/if}
					</div>
				{/each}
				<button type="button" class="btn-ghost" onclick={() => tiers.push({ minBalance: null, rate: '' })}>+ Escalão</button>
			</div>
			<p class="muted">
				Taxa fixa: um único escalão a partir de 0 €. O saldo inteiro rende à taxa do escalão correspondente. As previsões mostram os juros
				líquidos (retenção de 28%).
			</p>
		</div>
	{/if}

	{#if error}
		<p class="col-span-2 text-sm text-red-600" role="alert">{error}</p>
	{/if}

	<div class="col-span-2 flex justify-end gap-2">
		<button type="button" onclick={oncancel} class="btn-secondary">Cancelar</button>
		<button type="submit" disabled={busy} class="btn-primary">Guardar</button>
	</div>
</form>
