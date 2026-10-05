<script lang="ts">
	import Filterable from '#lib/components/Filterable.svelte';
	import { untrack } from 'svelte';
	import {
		deleteWishlistItem,
		listWishlist,
		priorities,
		purchasePlan,
		saveWishlistItem,
		setWishlistItemPurchased,
		type PurchasePlan,
		type WishlistInput,
		type WishlistItem
	} from '#lib/api/purchases.ts';
	import AccountSelect from '#lib/components/AccountSelect.svelte';
	import Modal from '#lib/components/Modal.svelte';
	import Money from '#lib/components/Money.svelte';
	import MoneyInput from '#lib/components/MoneyInput.svelte';
	import PageHeader from '#lib/components/PageHeader.svelte';
	import StatCard from '#lib/components/StatCard.svelte';
	import States from '#lib/components/States.svelte';
	import Toggle from '#lib/components/Toggle.svelte';
	import { confirmAction } from '#lib/dialogs.ts';
	import { formatDate, formatMoney, formatMonth, today } from '#lib/format.ts';
	import { errorMessage } from '#lib/graphql.ts';
	import { dataChanged, refs } from '#lib/refs.svelte.ts';
	import { notify, notifyError } from '#lib/toasts.svelte.ts';
	import { openQuickAdd } from '#lib/ui.svelte.ts';

	const HORIZONS = [6, 12, 24, 36];

	let margin = $state<number | null>(0);
	let allowOverdraft = $state(false);
	let simulation = $state(untrack(() => ({ amount: null as number | null, accountId: refs.accounts[0]?.id ?? '', months: 12 })));
	let plan = $state<{ amount: number; result: PurchasePlan } | null>(null);
	let planError = $state('');
	let editing = $state<{ id: string | null; form: Omit<WishlistInput, 'amount'> & { amount: number | null } } | null>(null);
	let formError = $state('');

	const options = $derived({ margin: margin ?? 0, allowOverdraft });

	const wishlist = $derived.by(() => {
		refs.version;
		return listWishlist(options);
	});

	async function simulate(event?: SubmitEvent) {
		event?.preventDefault();
		planError = '';
		if (!simulation.amount || simulation.amount <= 0) {
			planError = 'O valor tem de ser maior que zero.';
			return;
		}
		try {
			const amount = simulation.amount;
			plan = { amount, result: await purchasePlan({ ...simulation, amount, ...options }) };
		} catch (e) {
			planError = errorMessage(e);
		}
	}

	function simulateItem(item: WishlistItem) {
		simulation = { ...simulation, amount: item.amount, accountId: item.accountId };
		simulate();
		window.scrollTo({ top: 0 });
	}

	function create() {
		formError = '';
		editing = { id: null, form: { name: '', amount: null, accountId: refs.accounts[0]?.id ?? '', priority: 2, notes: null } };
	}

	function edit(item: WishlistItem) {
		formError = '';
		editing = { id: item.id, form: { name: item.name, amount: item.amount, accountId: item.accountId, priority: item.priority, notes: item.notes } };
	}

	async function save(event: SubmitEvent) {
		event.preventDefault();
		if (!editing) return;
		if (!editing.form.amount || editing.form.amount <= 0) {
			formError = 'O valor tem de ser maior que zero.';
			return;
		}
		try {
			await saveWishlistItem(editing.id, { ...editing.form, amount: editing.form.amount, notes: editing.form.notes || null });
			notify('Artigo guardado');
			editing = null;
			await dataChanged();
		} catch (e) {
			formError = errorMessage(e);
		}
	}

	async function markPurchased(item: WishlistItem, purchased: boolean) {
		try {
			await setWishlistItemPurchased(item.id, purchased);
			await dataChanged();
			if (purchased) {
				openQuickAdd('transaction', { kind: 'OUTCOME', amount: item.amount, description: item.name, accountId: item.accountId, date: today() });
			}
		} catch (e) {
			notifyError(e);
		}
	}

	async function remove(item: WishlistItem) {
		if (!(await confirmAction(`Eliminar "${item.name}" da lista de desejos?`))) return;
		try {
			await deleteWishlistItem(item.id);
			notify('Artigo eliminado');
			await dataChanged();
		} catch (e) {
			notifyError(e);
		}
	}
</script>

<PageHeader title="Compras planeadas" subtitle="Simulação de compras e lista de desejos, com base nas recorrências e nos movimentos agendados.">
	{#snippet actions()}
		<button class="btn-primary" onclick={create}>Novo artigo</button>
	{/snippet}
</PageHeader>

<section class="card mb-6 flex flex-wrap items-end gap-6">
	<div class="label w-48">
		Margem de segurança
		<MoneyInput bind:value={margin} />
	</div>
	<Toggle bind:checked={allowOverdraft} label="Permitir usar o descoberto autorizado" />
	<p class="muted max-w-xl">
		Uma compra só é considerada possível numa data se o saldo da conta não ficar abaixo da margem em nenhum dia seguinte, tendo em conta
		todas as recorrências e movimentos agendados.
	</p>
</section>

<section class="card mb-6">
	<h2 class="mb-3 font-medium">Simular compra</h2>
	<form onsubmit={simulate} class="grid grid-cols-2 gap-4 md:grid-cols-4">
		<div class="label">
			Valor
			<MoneyInput bind:value={simulation.amount} required />
		</div>
		<label class="label">
			Conta
			<AccountSelect bind:value={simulation.accountId} />
		</label>
		<label class="label">
			Horizonte
			<select bind:value={simulation.months} class="input">
				{#each HORIZONS as months (months)}
					<option value={months}>{months} meses</option>
				{/each}
			</select>
		</label>
		<div class="flex items-end"><button class="btn-primary w-full">Simular</button></div>
	</form>
	{#if planError}<p class="mt-3 text-sm text-red-600" role="alert">{planError}</p>{/if}

	{#if plan}
		{@const result = plan.result}
		<div class="mt-6 grid gap-4 md:grid-cols-3">
			<StatCard label="Saldo atual" value={formatMoney(result.balance)} />
			<StatCard
				plain
				label="Data mais cedo para comprar"
				value={result.earliestDate ? formatDate(result.earliestDate) : 'Fora do horizonte'}
				tone={result.earliestDate ? 'positive' : 'negative'}
				hint="Saldo mínimo exigido: {formatMoney(result.floor)}"
			/>
			<StatCard
				label="Saldo mais baixo se comprar hoje"
				value={formatMoney(result.lowestIfToday)}
				tone={result.lowestIfToday >= result.floor ? 'neutral' : 'negative'}
				hint={result.lowestIfToday >= result.floor ? 'Compra possível hoje' : 'Abaixo do mínimo exigido'}
			/>
		</div>
		<Filterable>
		<div class="mt-6 overflow-x-auto">
			<table class="table-base">
				<thead>
					<tr>
						<th>Mês</th>
						<th class="text-right">Saldo mais baixo previsto</th>
						<th class="text-right">Saldo mais baixo após a compra</th>
						<th>Estado</th>
					</tr>
				</thead>
				<tbody>
					{#each result.months as month (month.month)}
						<tr>
							<td>{formatMonth(month.month, 'long')}</td>
							<td class="text-right tabular-nums"><Money value={month.lowest} /></td>
							<td class="text-right tabular-nums {month.lowestAfter < result.floor ? 'text-red-700 dark:text-red-400' : ''}">
								<Money value={month.lowestAfter} />
							</td>
							<td>
								<span class="badge">{month.lowestAfter >= result.floor ? 'Possível' : 'Sem margem'}</span>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
		</Filterable>
		<p class="mt-2 text-xs text-stone-500">
			"Após a compra" considera a compra feita no início de cada mês (no mês atual, hoje) e o saldo mais baixo daí até ao fim do horizonte.
		</p>
	{/if}
</section>

<section class="card">
	<h2 class="mb-3 font-medium">Lista de desejos</h2>
	{#await wishlist}
		<States state="loading" />
	{:then items}
		{#if items.length === 0}
			<p class="muted">Sem artigos na lista de desejos.</p>
		{:else}
			<Filterable>
			<div class="overflow-x-auto">
				<table class="table-base">
					<thead>
						<tr>
							<th>Artigo</th>
							<th>Conta</th>
							<th>Prioridade</th>
							<th class="text-right">Valor</th>
							<th>Data prevista de compra</th>
							<th></th>
						</tr>
					</thead>
					<tbody>
						{#each items as item (item.id)}
							<tr class:opacity-60={item.purchasedAt}>
								<td>
									<p>{item.name}</p>
									{#if item.notes}<p class="text-xs text-stone-500">{item.notes}</p>{/if}
								</td>
								<td class="text-stone-500">{item.accountName}</td>
								<td><span class="badge">{priorities[item.priority]}</span></td>
								<td class="text-right tabular-nums"><Money value={item.amount} /></td>
								<td>
									{#if item.purchasedAt}
										Comprado em {formatDate(item.purchasedAt)}
									{:else if item.plannedDate}
										{formatDate(item.plannedDate)}
									{:else}
										<span class="text-red-700 dark:text-red-400">Fora do horizonte (24 meses)</span>
									{/if}
								</td>
								<td class="text-right whitespace-nowrap">
									{#if item.purchasedAt}
										<button class="btn-ghost" onclick={() => markPurchased(item, false)}>Repor</button>
									{:else}
										<button class="btn-ghost" onclick={() => simulateItem(item)}>Simular</button>
										<button class="btn-ghost" onclick={() => markPurchased(item, true)}>Comprado</button>
										<button class="btn-ghost" onclick={() => edit(item)}>Editar</button>
									{/if}
									<button class="btn-ghost text-red-600" onclick={() => remove(item)}>Eliminar</button>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
			</Filterable>
			<p class="mt-2 text-xs text-stone-500">
				As datas previstas seguem a ordem de prioridade: cada artigo conta com as compras dos artigos anteriores.
			</p>
		{/if}
	{:catch error}
		<States state="error" {error} />
	{/await}
</section>

{#if editing}
	<Modal title={editing.id ? 'Editar artigo' : 'Novo artigo'} onclose={() => (editing = null)}>
		<form onsubmit={save} class="grid grid-cols-2 gap-4">
			<label class="label col-span-2">
				Artigo
				<input bind:value={editing.form.name} class="input" required />
			</label>
			<div class="label">
				Valor
				<MoneyInput bind:value={editing.form.amount} required />
			</div>
			<label class="label">
				Conta
				<AccountSelect bind:value={editing.form.accountId} />
			</label>
			<label class="label">
				Prioridade
				<select bind:value={editing.form.priority} class="input">
					{#each Object.entries(priorities) as [value, label] (value)}
						<option value={Number(value)}>{label}</option>
					{/each}
				</select>
			</label>
			<label class="label col-span-2">
				Notas
				<textarea bind:value={editing.form.notes} rows="2" class="input"></textarea>
			</label>
			{#if formError}<p class="col-span-2 text-sm text-red-600" role="alert">{formError}</p>{/if}
			<div class="col-span-2 flex justify-end gap-2">
				<button type="button" class="btn-secondary" onclick={() => (editing = null)}>Cancelar</button>
				<button class="btn-primary">Guardar</button>
			</div>
		</form>
	</Modal>
{/if}
