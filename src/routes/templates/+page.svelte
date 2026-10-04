<script lang="ts">
	import { untrack } from 'svelte';
	import { deleteTemplate, listTemplates, saveTemplate, type Template, type TemplateInput } from '#lib/api/templates.ts';
	import AccountSelect from '#lib/components/AccountSelect.svelte';
	import CategorySelect from '#lib/components/CategorySelect.svelte';
	import Modal from '#lib/components/Modal.svelte';
	import MoneyInput from '#lib/components/MoneyInput.svelte';
	import PageHeader from '#lib/components/PageHeader.svelte';
	import States from '#lib/components/States.svelte';
	import { confirmAction } from '#lib/dialogs.ts';
	import { formatMoney } from '#lib/format.ts';
	import { errorMessage } from '#lib/graphql.ts';
	import { dataChanged, refs } from '#lib/refs.svelte.ts';
	import { notify, notifyError } from '#lib/toasts.svelte.ts';
	import { openQuickAdd } from '#lib/ui.svelte.ts';

	let editing = $state<{ id: string | null; form: TemplateInput } | null>(null);
	let formError = $state('');
	let request = $state<Promise<Template[]>>(new Promise(() => {}));

	$effect(() => {
		refs.version;
		untrack(() => (request = listTemplates()));
	});

	function open(template: Template | null) {
		formError = '';
		editing = {
			id: template?.id ?? null,
			form: {
				name: template?.name ?? '',
				accountId: template?.accountId ?? refs.accounts[0]?.id ?? '',
				categoryId: template?.categoryId ?? null,
				kind: template?.kind ?? 'OUTCOME',
				amount: template?.amount ?? null,
				description: template?.description ?? ''
			}
		};
	}

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		if (!editing) return;
		try {
			await saveTemplate(editing.id, editing.form);
			notify('Template guardado');
			editing = null;
			await dataChanged();
		} catch (e) {
			formError = errorMessage(e);
		}
	}

	async function remove(template: Template) {
		if (!(await confirmAction(`Eliminar o template "${template.name}"?`))) return;
		await deleteTemplate(template.id).catch(notifyError);
		await dataChanged();
	}

	function use(template: Template) {
		openQuickAdd('transaction', {
			accountId: template.accountId,
			categoryId: template.categoryId,
			kind: template.kind,
			amount: template.amount ?? 0,
			description: template.description
		});
	}
</script>

<PageHeader title="Templates" subtitle="Atalhos para movimentos frequentes. Também aparecem na adição rápida (tecla N).">
	{#snippet actions()}
		<button class="btn-primary" onclick={() => open(null)}>Novo template</button>
	{/snippet}
</PageHeader>

{#await request}
	<States state="loading" />
{:then templates}
	{#if templates.length === 0}
		<States state="empty" message="Ex.: “Supermercado” → Alimentação / Supermercado / Conta principal.">
			<button class="btn-primary" onclick={() => open(null)}>Criar template</button>
		</States>
	{:else}
		<div class="card overflow-x-auto p-0">
			<table class="table-base">
				<thead><tr><th>Nome</th><th>Tipo</th><th>Conta</th><th>Categoria</th><th>Descrição</th><th class="text-right">Valor</th><th></th></tr></thead>
				<tbody>
					{#each templates as template (template.id)}
						<tr>
							<td class="font-medium">{template.name}</td>
							<td>{template.kind === 'INCOME' ? 'Receita' : 'Despesa'}</td>
							<td>{template.accountName}</td>
							<td>{template.categoryName ?? '—'}</td>
							<td>{template.description || '—'}</td>
							<td class="text-right tabular-nums">{template.amount ? formatMoney(template.amount) : '—'}</td>
							<td class="text-right whitespace-nowrap">
								<button class="btn-ghost" onclick={() => use(template)}>Usar</button>
								<button class="btn-ghost" onclick={() => open(template)}>Editar</button>
								<button class="btn-ghost text-red-600" onclick={() => remove(template)}>Eliminar</button>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
{:catch error}
	<States state="error" {error} />
{/await}

{#if editing}
	<Modal title={editing.id ? 'Editar template' : 'Novo template'} onclose={() => (editing = null)}>
		<form onsubmit={submit} class="grid grid-cols-2 gap-4">
			<label class="label">
				Nome
				<input bind:value={editing.form.name} class="input" required />
			</label>
			<label class="label">
				Tipo
				<select bind:value={editing.form.kind} class="input">
					<option value="OUTCOME">Despesa</option>
					<option value="INCOME">Receita</option>
				</select>
			</label>
			<label class="label">
				Conta
				<AccountSelect bind:value={editing.form.accountId} />
			</label>
			<label class="label">
				Categoria
				<CategorySelect bind:value={editing.form.categoryId} kind={editing.form.kind} />
			</label>
			<label class="label">
				Descrição
				<input bind:value={editing.form.description} class="input" />
			</label>
			<label class="label">
				Valor (opcional)
				<MoneyInput bind:value={editing.form.amount} placeholder="" />
			</label>
			{#if formError}<p class="col-span-2 text-sm text-red-600" role="alert">{formError}</p>{/if}
			<div class="col-span-2 flex justify-end gap-2">
				<button type="button" class="btn-secondary" onclick={() => (editing = null)}>Cancelar</button>
				<button type="submit" class="btn-primary">Guardar</button>
			</div>
		</form>
	</Modal>
{/if}
