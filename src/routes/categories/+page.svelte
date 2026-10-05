<script lang="ts">
	import { untrack } from 'svelte';
	import {
		categoryKinds,
		deleteCategory,
		listCategories,
		saveCategory,
		setCategoryArchived,
		type Category,
		type CategoryInput
	} from '#lib/api/categories.ts';
	import CategorySelect from '#lib/components/CategorySelect.svelte';
	import IconPicker from '#lib/components/IconPicker.svelte';
	import Modal from '#lib/components/Modal.svelte';
	import PageHeader from '#lib/components/PageHeader.svelte';
	import States from '#lib/components/States.svelte';
	import TransactionsModal from '#lib/components/TransactionsModal.svelte';
	import Toggle from '#lib/components/Toggle.svelte';
	import { confirmAction } from '#lib/dialogs.ts';
	import { errorMessage } from '#lib/graphql.ts';
	import { dataChanged, refs } from '#lib/refs.svelte.ts';
	import { notify, notifyError } from '#lib/toasts.svelte.ts';

	let showArchived = $state(false);
	let viewing = $state<Category | null>(null);
	let categories = $state<Category[]>([]);
	let status = $state<'loading' | 'ready' | 'error'>('loading');
	let loadError = $state<unknown>(null);
	let editing = $state<{ id: string | null; form: CategoryInput } | null>(null);
	let removing = $state<{ category: Category; reassignTo: string | null } | null>(null);
	let formError = $state('');

	async function load() {
		try {
			categories = await listCategories(showArchived);
			status = 'ready';
		} catch (e) {
			loadError = e;
			status = 'error';
		}
	}

	$effect(() => {
		refs.version;
		untrack(load);
	});

	const parents = $derived(categories.filter((c) => !c.parentId));
	const childrenOf = (id: string) => categories.filter((c) => c.parentId === id);

	function create(parent: Category | null = null) {
		formError = '';
		editing = {
			id: null,
			form: { parentId: parent?.id ?? null, name: '', kind: parent?.kind ?? 'OUTCOME', icon: null, color: parent?.color ?? null }
		};
	}

	function edit(category: Category) {
		formError = '';
		const { parentId, name, kind, icon, color } = category;
		editing = { id: category.id, form: { parentId, name, kind, icon, color } };
	}

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		if (!editing) return;
		try {
			await saveCategory(editing.id, { ...editing.form, icon: editing.form.icon || null });
			notify('Categoria guardada');
			editing = null;
			await dataChanged();
		} catch (e) {
			formError = errorMessage(e);
		}
	}

	async function toggleArchive(category: Category) {
		await setCategoryArchived(category.id, !category.archivedAt).catch(notifyError);
		await dataChanged();
	}

	async function remove() {
		if (!removing) return;
		const { category, reassignTo } = removing;
		if (!(await confirmAction(`Eliminar a categoria "${category.name}"?`))) return;
		try {
			await deleteCategory(category.id, reassignTo);
			notify('Categoria eliminada');
			removing = null;
			await dataChanged();
		} catch (e) {
			notifyError(e);
		}
	}
</script>

{#snippet row(category: Category, nested: boolean)}
	<li
		class="flex flex-wrap items-center gap-3 px-4 py-2 transition-colors duration-100 hover:bg-stone-100/70 md:px-5 dark:hover:bg-stone-800/50 {nested ? 'pl-12 md:pl-13' : ''}"
		class:opacity-60={category.archivedAt}
	>
		<span class="size-3 rounded-full" style:background-color={category.color ?? 'transparent'}></span>
		<button class="flex-1 text-left hover:underline {nested ? '' : 'font-medium'}" onclick={() => (viewing = category)}>
			{category.icon ?? ''} {category.name}
		</button>
		<span class="badge">{categoryKinds[category.kind]}</span>
		<span class="w-24 text-right text-xs text-stone-500">{category.transactionCount} movimentos</span>
		<div class="flex flex-wrap">
			{#if !nested}<button class="btn-ghost" onclick={() => create(category)}>+ Sub</button>{/if}
			<button class="btn-ghost" onclick={() => edit(category)}>Editar</button>
			<button class="btn-ghost" onclick={() => toggleArchive(category)}>{category.archivedAt ? 'Reativar' : 'Arquivar'}</button>
			<button class="btn-ghost text-red-600" onclick={() => (removing = { category, reassignTo: null })}>Eliminar</button>
		</div>
	</li>
{/snippet}

<PageHeader title="Categorias" subtitle="Receitas e despesas organizadas em categorias e subcategorias.">
	{#snippet actions()}
		<Toggle bind:checked={showArchived} onchange={load} label="Mostrar arquivadas" />
		<button class="btn-primary" onclick={() => create()}>Nova categoria</button>
	{/snippet}
</PageHeader>

{#if status === 'loading'}
	<States state="loading" />
{:else if status === 'error'}
	<States state="error" error={loadError} />
{:else if categories.length === 0}
	<States state="empty" message="Sem categorias." />
{:else}
	<ul class="card divide-y divide-stone-100 overflow-hidden px-0 py-1 md:px-0 dark:divide-stone-800">
		{#each parents as parent (parent.id)}
			{@render row(parent, false)}
			{#each childrenOf(parent.id) as child (child.id)}
				{@render row(child, true)}
			{/each}
		{/each}
	</ul>
{/if}

{#if editing}
	<Modal title={editing.id ? 'Editar categoria' : 'Nova categoria'} onclose={() => (editing = null)}>
		<form onsubmit={submit} class="grid grid-cols-2 gap-4">
			<label class="label col-span-2">
				Nome
				<input bind:value={editing.form.name} class="input" required />
			</label>
			<div class="label">
				Categoria principal
				<CategorySelect bind:value={editing.form.parentId} parentsOnly exclude={editing.id} emptyLabel="Nenhuma" />
			</div>
			<label class="label">
				Tipo
				<select bind:value={editing.form.kind} class="input">
					{#each Object.entries(categoryKinds) as [value, label] (value)}
						<option {value}>{label}</option>
					{/each}
				</select>
			</label>
			<label class="label">
				Cor
				<input type="color" value={editing.form.color ?? '#2a78d6'} oninput={(e) => editing && (editing.form.color = e.currentTarget.value)} class="h-9 w-full rounded-md" />
			</label>
			<div class="label">
				Ícone
				<IconPicker bind:value={editing.form.icon} />
			</div>
			{#if formError}<p class="col-span-2 text-sm text-red-600" role="alert">{formError}</p>{/if}
			<div class="col-span-2 flex justify-end gap-2">
				<button type="button" class="btn-secondary" onclick={() => (editing = null)}>Cancelar</button>
				<button type="submit" class="btn-primary">Guardar</button>
			</div>
		</form>
	</Modal>
{/if}

{#if removing}
	<Modal title="Eliminar categoria" onclose={() => (removing = null)}>
		<div class="space-y-4">
			<p class="text-sm">
				<strong>{removing.category.name}</strong> tem {removing.category.transactionCount} movimentos. Para eliminar uma categoria em uso,
				é necessária uma categoria de destino para os movimentos, recorrências e templates.
			</p>
			<div class="label">
				Mover para
				<CategorySelect bind:value={removing.reassignTo} exclude={removing.category.id} emptyLabel="Não mover (só se não estiver em uso)" />
			</div>
			<div class="flex justify-end gap-2">
				<button class="btn-secondary" onclick={() => (removing = null)}>Cancelar</button>
				<button class="btn-danger" onclick={remove}>Eliminar</button>
			</div>
		</div>
	</Modal>
{/if}

{#if viewing}
	<TransactionsModal title={viewing.name} filter={{ categoryId: viewing.id }} onclose={() => (viewing = null)} />
{/if}
