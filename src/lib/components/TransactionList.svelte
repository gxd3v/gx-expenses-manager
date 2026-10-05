<script lang="ts">
	import { untrack } from 'svelte';
	import { goto } from '$app/navigation';
	import { exportCsv } from '#lib/api/backups.ts';
	import {
		deleteSavedFilter,
		deleteTransaction,
		getTransfer,
		listSavedFilters,
		listTransactions,
		saveFilter,
		saveTransaction,
		saveTransfer,
		setConfirmed,
		toInput,
		transactionKinds,
		type SavedFilter,
		type Transaction,
		type TransactionFilter,
		type Transfer
	} from '#lib/api/transactions.ts';
	import { confirmAction, pickSavePath } from '#lib/dialogs.ts';
	import { formatDate, today } from '#lib/format.ts';
	import { dataChanged, refs } from '#lib/refs.svelte.ts';
	import { openQuickAdd } from '#lib/ui.svelte.ts';
	import { notify, notifyError } from '#lib/toasts.svelte.ts';
	import AccountSelect from './AccountSelect.svelte';
	import Amount from './Amount.svelte';
	import ContextMenu, { type MenuItem } from './ContextMenu.svelte';
	import CategorySelect from './CategorySelect.svelte';
	import Modal from './Modal.svelte';
	import MoneyInput from './MoneyInput.svelte';
	import States from './States.svelte';
	import TransactionForm from './TransactionForm.svelte';
	import TransferForm from './TransferForm.svelte';

	const PAGE_SIZE = 50;

	let { initialFilter = {}, fixedAccountId = null }: { initialFilter?: TransactionFilter; fixedAccountId?: string | null } =
		$props();

	let filter = $state<TransactionFilter>(untrack(() => ({ ...initialFilter, accountId: fixedAccountId ?? initialFilter.accountId })));
	let items = $state<Transaction[]>([]);
	let totals = $state({ totalCount: 0, income: 0, outcome: 0, net: 0 });
	let status = $state<'loading' | 'ready' | 'error'>('loading');
	let error = $state<unknown>(null);
	let selected = $state<string[]>([]);
	let editing = $state<Transaction | null>(null);
	let editingTransfer = $state<Transfer | null>(null);
	let savedFilters = $state<SavedFilter[]>([]);

	async function load(append = false) {
		if (!append) status = 'loading';
		try {
			const page = await listTransactions(filter, PAGE_SIZE, append ? items.length : 0);
			items = append ? [...items, ...page.items] : page.items;
			totals = page;
			selected = [];
			status = 'ready';
		} catch (e) {
			error = e;
			status = 'error';
		}
	}

	async function loadSavedFilters() {
		savedFilters = await listSavedFilters().catch(() => []);
	}

	$effect(() => {
		refs.version;
		untrack(() => {
			load();
			loadSavedFilters();
		});
	});

	let filtersOpen = $state(false);
	const activeFilters = $derived(
		Object.entries(filter).filter(([key, value]) => value !== null && value !== undefined && value !== '' && !(key === 'accountId' && fixedAccountId))
			.length
	);
	const categoryOptions = $derived(
		refs.categories
			.filter((c) => !c.parentId)
			.flatMap((parent) => [
				{ id: parent.id, label: parent.name },
				...refs.categories.filter((c) => c.parentId === parent.id).map((child) => ({ id: child.id, label: `${parent.name} / ${child.name}` }))
			])
	);

	function headerFilter(key: 'categoryId' | 'accountId', value: string) {
		filter[key] = value || null;
		load();
	}

	function clearFilters() {
		filter = { accountId: fixedAccountId };
		load();
	}

	async function applySaved(event: Event) {
		const saved = savedFilters.find((f) => f.id === (event.target as HTMLSelectElement).value);
		if (!saved) return;
		filter = { ...JSON.parse(saved.filter), accountId: fixedAccountId ?? JSON.parse(saved.filter).accountId };
		await load();
	}

	async function storeFilter() {
		const name = window.prompt('Nome do filtro');
		if (!name) return;
		await saveFilter(name, filter).catch(notifyError);
		await loadSavedFilters();
		notify('Filtro guardado');
	}

	async function removeSaved(id: string) {
		await deleteSavedFilter(id).catch(notifyError);
		await loadSavedFilters();
	}

	async function edit(transaction: Transaction) {
		if (!transaction.transferId) {
			editing = transaction;
			return;
		}
		editingTransfer = await getTransfer(transaction.transferId).catch((e) => {
			notifyError(e);
			return null;
		});
	}

	async function remove(transaction: Transaction) {
		if (!(await confirmAction(`Eliminar "${transaction.description || 'movimento'}"?`))) return;
		try {
			const transfer = transaction.transferId ? await getTransfer(transaction.transferId) : null;
			await deleteTransaction(transaction.id);
			await dataChanged();
			notify(transfer ? 'Transferência eliminada' : 'Movimento eliminado', 'success', {
				label: 'Desfazer',
				run: () => restore(transaction, transfer)
			});
		} catch (e) {
			notifyError(e);
		}
	}

	async function restore(transaction: Transaction, transfer: Transfer | null) {
		try {
			if (transfer) await saveTransfer(null, transfer);
			else await saveTransaction(null, toInput(transaction));
			await dataChanged();
		} catch (e) {
			notifyError(e);
		}
	}

	async function toggleConfirmed(transaction: Transaction) {
		await setConfirmed([transaction.id], !transaction.confirmed).catch(notifyError);
		transaction.confirmed = !transaction.confirmed;
	}

	let menu = $state<{ x: number; y: number; transaction: Transaction } | null>(null);

	function openMenu(event: MouseEvent, transaction: Transaction) {
		event.preventDefault();
		menu = { x: event.clientX, y: event.clientY, transaction };
	}

	function menuItems(transaction: Transaction): MenuItem[] {
		const entry: MenuItem[] = transaction.transferId
			? []
			: [
					{ label: transaction.oneOff ? 'Remover marcação de pontual' : 'Marcar como pontual', run: () => toggleOneOff(transaction) },
					{ label: 'Duplicar', run: () => openQuickAdd('transaction', { ...toInput(transaction), date: today(), confirmed: false }) }
				];
		const account: MenuItem[] = fixedAccountId ? [] : [{ label: 'Abrir conta', run: () => goto(`/accounts/${transaction.accountId}`) }];
		return [
			{ label: 'Editar', run: () => edit(transaction) },
			{ label: transaction.confirmed ? 'Marcar como por confirmar' : 'Marcar como confirmado', run: () => toggleConfirmed(transaction) },
			...entry,
			...account,
			{ label: 'Eliminar', run: () => remove(transaction), danger: true }
		];
	}

	async function toggleOneOff(transaction: Transaction) {
		try {
			await saveTransaction(transaction.id, { ...toInput(transaction), oneOff: !transaction.oneOff });
			notify(transaction.oneOff ? 'Marcação de pontual removida' : 'Marcado como pontual');
			await dataChanged();
		} catch (e) {
			notifyError(e);
		}
	}

	async function confirmSelected() {
		await setConfirmed(selected, true).catch(notifyError);
		notify(`${selected.length} movimentos confirmados`);
		await load();
	}

	async function exportFiltered() {
		const path = await pickSavePath(`movimentos-${today()}.csv`, 'csv');
		if (!path) return;
		try {
			const count = await exportCsv(path, filter);
			notify(`${count} movimentos exportados`);
		} catch (e) {
			notifyError(e);
		}
	}

	async function closeEditors() {
		editing = null;
		editingTransfer = null;
		await dataChanged();
	}

	const allSelected = $derived(items.length > 0 && selected.length === items.length);
</script>

<form class="mb-3 flex flex-wrap items-center gap-2" onsubmit={(e) => (e.preventDefault(), load())}>
	<input bind:value={filter.search} class="input max-w-sm" placeholder="Pesquisar descrição ou notas" aria-label="Pesquisa" />
	<button type="button" class="btn-secondary" onclick={() => (filtersOpen = !filtersOpen)} aria-expanded={filtersOpen}>
		{filtersOpen ? '▾' : '▸'} Filtros{activeFilters ? ` (${activeFilters})` : ''}
	</button>
	{#if activeFilters}<button type="button" class="btn-ghost" onclick={clearFilters}>Limpar</button>{/if}
</form>

{#if filtersOpen}
<form class="card mb-4 grid grid-cols-2 gap-3 md:grid-cols-4" onsubmit={(e) => (e.preventDefault(), load())}>
	{#if !fixedAccountId}
		<label class="label">
			Conta
			<AccountSelect bind:value={filter.accountId as string | null} allowEmpty />
		</label>
	{/if}
	<label class="label">
		Categoria
		<CategorySelect bind:value={filter.categoryId as string | null} emptyLabel="Todas" />
	</label>
	<label class="label">
		Tipo
		<select bind:value={filter.kind} class="input">
			<option value={null}>Todos</option>
			{#each Object.entries(transactionKinds) as [value, label] (value)}
				<option {value}>{label}</option>
			{/each}
		</select>
	</label>
	<label class="label">
		De
		<input type="date" bind:value={filter.dateFrom} class="input" />
	</label>
	<label class="label">
		Até
		<input type="date" bind:value={filter.dateTo} class="input" />
	</label>
	<label class="label">
		Valor mínimo
		<MoneyInput bind:value={filter.minAmount as number | null} placeholder="" />
	</label>
	<label class="label">
		Valor máximo
		<MoneyInput bind:value={filter.maxAmount as number | null} placeholder="" />
	</label>
	<label class="label">
		Estado
		<select bind:value={filter.confirmed} class="input">
			<option value={null}>Todos</option>
			<option value={true}>Confirmados</option>
			<option value={false}>Por confirmar</option>
		</select>
	</label>
	<div class="col-span-2 flex flex-wrap items-end gap-2 md:col-span-4">
		<button type="submit" class="btn-primary">Filtrar</button>
		<button type="button" class="btn-secondary" onclick={clearFilters}>Limpar</button>
		<button type="button" class="btn-secondary" onclick={storeFilter}>Guardar filtro</button>
		{#if savedFilters.length}
			<select class="input w-48" onchange={applySaved} aria-label="Filtros guardados">
				<option value="">Filtros guardados…</option>
				{#each savedFilters as saved (saved.id)}
					<option value={saved.id}>{saved.name}</option>
				{/each}
			</select>
			<details class="relative">
				<summary class="btn-ghost cursor-pointer">Gerir</summary>
				<ul class="card absolute z-10 mt-1 w-56 p-2">
					{#each savedFilters as saved (saved.id)}
						<li class="flex items-center justify-between text-sm">
							{saved.name}
							<button type="button" class="btn-ghost" onclick={() => removeSaved(saved.id)} aria-label="Eliminar {saved.name}">✕</button>
						</li>
					{/each}
				</ul>
			</details>
		{/if}
		<button type="button" class="btn-secondary ml-auto" onclick={exportFiltered}>Exportar CSV</button>
	</div>
</form>
{/if}

<div class="mb-4 flex flex-wrap items-center gap-6 text-sm">
	<span><span class="muted">Movimentos:</span> {totals.totalCount}</span>
	<span><span class="muted">Receitas:</span> <Amount value={totals.income} /></span>
	<span><span class="muted">Despesas:</span> <Amount value={-totals.outcome} /></span>
	<span><span class="muted">Resultado:</span> <Amount value={totals.net} /></span>
	{#if selected.length}
		<button class="btn-secondary ml-auto" onclick={confirmSelected}>Confirmar {selected.length} selecionados</button>
	{/if}
</div>

{#if status === 'loading'}
	<States state="loading" />
{:else if status === 'error'}
	<States state="error" {error} />
{:else if items.length === 0}
	<States state="empty" message="Não há movimentos para estes filtros." />
{:else}
	<div class="card overflow-x-auto p-0">
		<table class="table-base">
			<thead>
				<tr>
					<th class="w-8">
						<input
							type="checkbox"
							checked={allSelected}
							onchange={() => (selected = allSelected ? [] : items.map((i) => i.id))}
							aria-label="Selecionar todos"
						/>
					</th>
					<th>Data</th>
					<th>Descrição</th>
					<th>
						<select
							class="header-filter"
							value={filter.categoryId ?? ''}
							onchange={(e) => headerFilter('categoryId', e.currentTarget.value)}
							aria-label="Filtrar por categoria"
						>
							<option value="">Categoria</option>
							{#each categoryOptions as option (option.id)}
								<option value={option.id}>{option.label}</option>
							{/each}
						</select>
					</th>
					{#if !fixedAccountId}
						<th>
							<select
								class="header-filter"
								value={filter.accountId ?? ''}
								onchange={(e) => headerFilter('accountId', e.currentTarget.value)}
								aria-label="Filtrar por conta"
							>
								<option value="">Conta</option>
								{#each refs.accounts as account (account.id)}
									<option value={account.id}>{account.name}</option>
								{/each}
							</select>
						</th>
					{/if}
					<th class="text-right">Valor</th>
					<th class="text-center">Confirmado</th>
					<th></th>
				</tr>
			</thead>
			<tbody>
				{#each items as transaction (transaction.id)}
					<tr class={transaction.date > today() ? 'text-stone-400' : ''} oncontextmenu={(e) => openMenu(e, transaction)}>
						<td><input type="checkbox" bind:group={selected} value={transaction.id} aria-label="Selecionar" /></td>
						<td class="whitespace-nowrap">{formatDate(transaction.date)}</td>
						<td>
							{transaction.description || '—'}
							{#if transaction.counterpartAccountId}
								<a class="badge ml-1 hover:underline" href="/accounts/{transaction.counterpartAccountId}"
									>Transferência {transaction.amount < 0 ? '→' : '←'} {transaction.counterpartAccountName}</a
								>
							{/if}
							{#if transaction.recurrenceId}<span class="badge ml-1">Recorrente</span>{/if}
							{#if transaction.oneOff}<span class="badge ml-1">Pontual</span>{/if}
							{#if transaction.date > today()}<span class="badge ml-1">Futuro</span>{/if}
						</td>
						<td class="text-stone-500">{transaction.categoryName ?? '—'}</td>
						{#if !fixedAccountId}<td class="text-stone-500">{transaction.accountName}</td>{/if}
						<td class="text-right"><Amount value={transaction.amount} currency={transaction.currency} /></td>
						<td class="text-center">
							<input
								type="checkbox"
								checked={transaction.confirmed}
								onchange={() => toggleConfirmed(transaction)}
								aria-label="Confirmado"
							/>
						</td>
						<td class="text-right whitespace-nowrap">
							<button class="btn-ghost" onclick={() => edit(transaction)}>Editar</button>
							<button class="btn-ghost text-red-600" onclick={() => remove(transaction)}>Eliminar</button>
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
	{#if items.length < totals.totalCount}
		<div class="mt-4 flex justify-center">
			<button class="btn-secondary" onclick={() => load(true)}>Carregar mais ({totals.totalCount - items.length})</button>
		</div>
	{/if}
{/if}

{#if editing}
	<Modal title="Editar movimento" onclose={() => (editing = null)}>
		<TransactionForm id={editing.id} initial={toInput(editing)} onsaved={closeEditors} oncancel={() => (editing = null)} />
	</Modal>
{/if}

{#if editingTransfer}
	<Modal title="Editar transferência" onclose={() => (editingTransfer = null)}>
		<TransferForm id={editingTransfer.id} initial={editingTransfer} onsaved={closeEditors} oncancel={() => (editingTransfer = null)} />
	</Modal>
{/if}

{#if refs.accounts.length === 0}
	<p class="mt-4 muted">É necessária pelo menos uma conta para registar movimentos.</p>
{/if}

{#if menu}
	<ContextMenu x={menu.x} y={menu.y} items={menuItems(menu.transaction)} onclose={() => (menu = null)} />
{/if}
