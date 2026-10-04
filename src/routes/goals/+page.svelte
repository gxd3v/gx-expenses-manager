<script lang="ts">
	import { untrack } from 'svelte';
	import GoalProgress from './GoalProgress.svelte';
	import { deleteGoal, listGoals, saveGoal, setGoalArchived, type Goal } from '#lib/api/goals.ts';
	import AccountSelect from '#lib/components/AccountSelect.svelte';
	import Modal from '#lib/components/Modal.svelte';
	import MoneyInput from '#lib/components/MoneyInput.svelte';
	import PageHeader from '#lib/components/PageHeader.svelte';
	import ProgressBar from '#lib/components/ProgressBar.svelte';
	import States from '#lib/components/States.svelte';
	import Toggle from '#lib/components/Toggle.svelte';
	import { confirmAction } from '#lib/dialogs.ts';
	import { formatDate, formatMoney, formatMonth, formatPercent } from '#lib/format.ts';
	import { errorMessage } from '#lib/graphql.ts';
	import { dataChanged, refs } from '#lib/refs.svelte.ts';
	import { notify, notifyError } from '#lib/toasts.svelte.ts';

	type Form = { id: string | null; name: string; accountId: string; targetAmount: number | null; targetDate: string };

	let showArchived = $state(false);
	let expanded = $state<string | null>(null);
	let editing = $state<Form | null>(null);
	let formError = $state('');
	let request = $state<Promise<Goal[]>>(new Promise(() => {}));

	function load() {
		request = listGoals(showArchived);
	}

	$effect(() => {
		refs.version;
		untrack(load);
	});

	function open(goal: Goal | null) {
		formError = '';
		editing = {
			id: goal?.id ?? null,
			name: goal?.name ?? '',
			accountId: goal?.accountId ?? refs.accounts[0]?.id ?? '',
			targetAmount: goal?.targetAmount ?? null,
			targetDate: goal?.targetDate ?? ''
		};
	}

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		if (!editing?.targetAmount) {
			formError = 'Indica o valor objetivo.';
			return;
		}
		try {
			await saveGoal(editing.id, {
				name: editing.name,
				accountId: editing.accountId,
				targetAmount: editing.targetAmount,
				targetDate: editing.targetDate || null
			});
			notify('Objetivo guardado');
			editing = null;
			await dataChanged();
		} catch (e) {
			formError = errorMessage(e);
		}
	}

	async function toggleArchive(goal: Goal) {
		await setGoalArchived(goal.id, !goal.archivedAt).catch(notifyError);
		await dataChanged();
	}

	async function remove(goal: Goal) {
		if (!(await confirmAction(`Eliminar o objetivo "${goal.name}"?`))) return;
		await deleteGoal(goal.id).catch(notifyError);
		await dataChanged();
	}
</script>

<PageHeader title="Objetivos" subtitle="Metas de poupança associadas a uma conta.">
	{#snippet actions()}
		<Toggle bind:checked={showArchived} onchange={load} label="Mostrar arquivados" />
		<button class="btn-primary" onclick={() => open(null)}>Novo objetivo</button>
	{/snippet}
</PageHeader>

{#await request}
	<States state="loading" />
{:then goals}
	{#if goals.length === 0}
		<States state="empty" message="Sem objetivos. Ex.: 10.000 € até dezembro de 2027.">
			<button class="btn-primary" onclick={() => open(null)}>Criar objetivo</button>
		</States>
	{:else}
		<div class="grid gap-4 xl:grid-cols-2">
			{#each goals as goal (goal.id)}
				<article class="card space-y-3" class:opacity-60={goal.archivedAt}>
					<header class="flex items-start justify-between">
						<div>
							<h2 class="font-medium">{goal.name}</h2>
							<p class="text-xs text-stone-500">{goal.accountName}{goal.targetDate ? ` · até ${formatDate(goal.targetDate)}` : ''}</p>
						</div>
						<span class="text-lg font-semibold">{formatPercent(goal.progress)}</span>
					</header>
					<ProgressBar value={goal.progress} label={goal.name} />
					<dl class="grid grid-cols-2 gap-2 text-sm md:grid-cols-4">
						<div><dt class="text-xs text-stone-500">Atual</dt><dd class="tabular-nums">{formatMoney(goal.currentAmount)}</dd></div>
						<div><dt class="text-xs text-stone-500">Objetivo</dt><dd class="tabular-nums">{formatMoney(goal.targetAmount)}</dd></div>
						<div><dt class="text-xs text-stone-500">Em falta</dt><dd class="tabular-nums">{formatMoney(goal.remaining)}</dd></div>
						<div>
							<dt class="text-xs text-stone-500">Por mês</dt>
							<dd class="tabular-nums">{goal.monthlyNeeded !== null ? formatMoney(goal.monthlyNeeded) : '—'}</dd>
						</div>
					</dl>
					<p class="text-sm">
						<span class="muted">Previsão de conclusão:</span>
						{goal.remaining === 0 ? 'Atingido 🎉' : goal.projectedDate ? formatMonth(goal.projectedDate, 'long') : 'não atingido nos próximos 10 anos'}
					</p>
					{#if expanded === goal.id}
						<GoalProgress {goal} />
					{/if}
					<div class="flex flex-wrap gap-1">
						<button class="btn-ghost" onclick={() => (expanded = expanded === goal.id ? null : goal.id)} aria-expanded={expanded === goal.id}>
							{expanded === goal.id ? 'Esconder evolução' : 'Ver evolução'}
						</button>
						<button class="btn-ghost" onclick={() => open(goal)}>Editar</button>
						<button class="btn-ghost" onclick={() => toggleArchive(goal)}>{goal.archivedAt ? 'Reativar' : 'Arquivar'}</button>
						<button class="btn-ghost text-red-600" onclick={() => remove(goal)}>Eliminar</button>
					</div>
				</article>
			{/each}
		</div>
	{/if}
{:catch error}
	<States state="error" {error} />
{/await}

{#if editing}
	<Modal title={editing.id ? 'Editar objetivo' : 'Novo objetivo'} onclose={() => (editing = null)}>
		<form onsubmit={submit} class="grid grid-cols-2 gap-4">
			<label class="label col-span-2">
				Nome
				<input bind:value={editing.name} class="input" required placeholder="Fundo de emergência" />
			</label>
			<label class="label col-span-2">
				Conta associada
				<AccountSelect bind:value={editing.accountId} />
			</label>
			<label class="label">
				Valor objetivo
				<MoneyInput bind:value={editing.targetAmount} required />
			</label>
			<label class="label">
				Data objetivo
				<input type="date" bind:value={editing.targetDate} class="input" />
			</label>
			{#if formError}<p class="col-span-2 text-sm text-red-600" role="alert">{formError}</p>{/if}
			<div class="col-span-2 flex justify-end gap-2">
				<button type="button" class="btn-secondary" onclick={() => (editing = null)}>Cancelar</button>
				<button type="submit" class="btn-primary">Guardar</button>
			</div>
		</form>
	</Modal>
{/if}
