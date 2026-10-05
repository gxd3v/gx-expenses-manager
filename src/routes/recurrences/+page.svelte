<script lang="ts">
	import Filterable from '#lib/components/Filterable.svelte';
	import { untrack } from 'svelte';
	import Money from '#lib/components/Money.svelte';
	import RecurrenceForm from './RecurrenceForm.svelte';
	import {
		endRecurrence,
		frequencyLabel,
		listOccurrences,
		listRecurrences,
		modifyOccurrence,
		recurrenceAction,
		resetOccurrence,
		skipOccurrence,
		type Occurrence,
		type Recurrence
	} from '#lib/api/recurrences.ts';
	import Amount from '#lib/components/Amount.svelte';
	import Modal from '#lib/components/Modal.svelte';
	import MoneyInput from '#lib/components/MoneyInput.svelte';
	import PageHeader from '#lib/components/PageHeader.svelte';
	import States from '#lib/components/States.svelte';
	import { confirmAction } from '#lib/dialogs.ts';
	import { addMonths, formatDate, today } from '#lib/format.ts';
	import { dataChanged, refs } from '#lib/refs.svelte.ts';
	import { notify, notifyError } from '#lib/toasts.svelte.ts';

	const PREVIEW_COUNT = 6;

	let editing = $state<Recurrence | null | undefined>(undefined);
	let expanded = $state<string | null>(null);
	let occurrences = $state<Occurrence[]>([]);
	let changing = $state<{ occurrence: Occurrence; amount: number | null; date: string } | null>(null);
	let request = $state<Promise<Recurrence[]>>(new Promise(() => {}));

	$effect(() => {
		refs.version;
		untrack(() => {
			request = listRecurrences();
			if (expanded) loadOccurrences(expanded);
		});
	});

	const ended = (r: Recurrence) => r.endDate !== null && r.endDate < today();
	const statusLabel = (r: Recurrence) => (ended(r) ? 'Terminada' : r.pausedAt ? 'Em pausa' : 'Ativa');

	async function loadOccurrences(id: string) {
		const all = await listOccurrences(today(), addMonths(today(), 24), id).catch(() => []);
		occurrences = all.slice(0, PREVIEW_COUNT);
	}

	async function toggle(recurrence: Recurrence) {
		expanded = expanded === recurrence.id ? null : recurrence.id;
		if (expanded) await loadOccurrences(expanded);
	}

	async function run(action: () => Promise<unknown>, message: string) {
		try {
			await action();
			notify(message);
			await dataChanged();
		} catch (e) {
			notifyError(e);
		}
	}

	async function end(recurrence: Recurrence) {
		const date = window.prompt('Data de fim (AAAA-MM-DD)', today());
		if (date) await run(() => endRecurrence(recurrence.id, date), 'Recorrência terminada');
	}

	async function remove(recurrence: Recurrence) {
		const message = `Eliminar "${recurrence.description}"? Os movimentos já registados mantêm-se.`;
		if (await confirmAction(message)) await run(() => recurrenceAction('deleteRecurrence', recurrence.id), 'Recorrência eliminada');
	}

	async function saveChange(event: SubmitEvent) {
		event.preventDefault();
		if (!changing) return;
		const { occurrence, amount, date } = changing;
		changing = null;
		await run(
			() => modifyOccurrence(occurrence.recurrenceId, occurrence.occurrenceDate, amount, date === occurrence.occurrenceDate ? null : date),
			'Ocorrência alterada'
		);
	}
</script>

<PageHeader title="Recorrências" subtitle="Despesas e receitas que se repetem. Alimentam as previsões.">
	{#snippet actions()}
		<button class="btn-primary" onclick={() => (editing = null)}>Nova recorrência</button>
	{/snippet}
</PageHeader>

{#await request}
	<States state="loading" />
{:then recurrences}
	{#if recurrences.length === 0}
		<States state="empty" message="Sem recorrências. Ex.: renda, salário, subscrições.">
			<button class="btn-primary" onclick={() => (editing = null)}>Criar recorrência</button>
		</States>
	{:else}
		<Filterable>
		<div class="card overflow-x-auto p-0">
			<table class="table-base">
				<thead>
					<tr>
						<th>Descrição</th>
						<th>Periodicidade</th>
						<th>Conta</th>
						<th>Próxima</th>
						<th>Estado</th>
						<th class="text-right">Valor</th>
						<th></th>
					</tr>
				</thead>
				<tbody>
					{#each recurrences as recurrence (recurrence.id)}
						<tr class:opacity-60={ended(recurrence) || recurrence.pausedAt}>
							<td>
								<button class="text-left hover:underline" onclick={() => toggle(recurrence)} aria-expanded={expanded === recurrence.id}>
									{expanded === recurrence.id ? '▾' : '▸'} {recurrence.description}
								</button>
								{#if recurrence.creditId}<span class="badge ml-1">Crédito</span>{/if}
								{#if recurrence.toAccountId}
									<a class="text-xs text-indigo-600 hover:underline dark:text-indigo-400" href="/accounts/{recurrence.toAccountId}"
										>Transferência → {recurrence.toAccountName}</a
									>
								{:else}
									<p class="text-xs text-stone-500">
										{recurrence.categoryName ?? 'Sem categoria'}{recurrence.variableAmount ? ' · valor variável' : ''}
									</p>
								{/if}
							</td>
							<td>{frequencyLabel(recurrence.unit, recurrence.interval)}</td>
							<td><a class="hover:underline" href="/accounts/{recurrence.accountId}">{recurrence.accountName}</a></td>
							<td>{formatDate(recurrence.nextDate)}</td>
							<td><span class="badge">{statusLabel(recurrence)}</span></td>
							<td class="text-right">
								<Amount value={recurrence.kind === 'INCOME' ? recurrence.amount : -recurrence.amount} signed={!recurrence.toAccountId} />
							</td>
							<td class="text-right whitespace-nowrap">
								<button class="btn-ghost" onclick={() => (editing = recurrence)}>Editar</button>
								{#if recurrence.pausedAt}
									<button class="btn-ghost" onclick={() => run(() => recurrenceAction('resumeRecurrence', recurrence.id), 'Recorrência retomada')}>Retomar</button>
								{:else if !ended(recurrence)}
									<button class="btn-ghost" onclick={() => run(() => recurrenceAction('pauseRecurrence', recurrence.id), 'Recorrência em pausa')}>Pausar</button>
									<button class="btn-ghost" onclick={() => end(recurrence)}>Terminar</button>
								{/if}
								<button class="btn-ghost text-red-600" onclick={() => remove(recurrence)}>Eliminar</button>
							</td>
						</tr>
						{#if expanded === recurrence.id}
							<tr>
								<td colspan="7" class="bg-stone-50 dark:bg-stone-950">
									<p class="mb-2 text-xs font-medium text-stone-500 uppercase">Próximas ocorrências</p>
									{#if occurrences.length === 0}
										<p class="muted">Sem ocorrências futuras.</p>
									{/if}
									<ul class="space-y-1 text-sm">
										{#each occurrences as occurrence (occurrence.occurrenceDate)}
											<li class="flex items-center gap-3">
												<span class="w-28">{formatDate(occurrence.date)}</span>
												<span class="w-28 tabular-nums"><Money value={occurrence.amount} /></span>
												{#if occurrence.modified}<span class="badge">Alterada</span>{/if}
												<button class="btn-ghost" onclick={() => (changing = { occurrence, amount: occurrence.amount, date: occurrence.date })}>Alterar</button>
												<button class="btn-ghost" onclick={() => run(() => skipOccurrence(occurrence.recurrenceId, occurrence.occurrenceDate), 'Ocorrência saltada')}>Saltar</button>
												{#if occurrence.modified}
													<button class="btn-ghost" onclick={() => run(() => resetOccurrence(occurrence.recurrenceId, occurrence.occurrenceDate), 'Ocorrência reposta')}>Repor</button>
												{/if}
											</li>
										{/each}
									</ul>
								</td>
							</tr>
						{/if}
					{/each}
				</tbody>
			</table>
		</div>
		</Filterable>
	{/if}
{:catch error}
	<States state="error" {error} />
{/await}

{#if editing !== undefined}
	<Modal title={editing ? 'Editar recorrência' : 'Nova recorrência'} onclose={() => (editing = undefined)}>
		<RecurrenceForm
			recurrence={editing}
			onsaved={async () => {
				editing = undefined;
				await dataChanged();
			}}
			oncancel={() => (editing = undefined)}
		/>
	</Modal>
{/if}

{#if changing}
	<Modal title="Alterar ocorrência de {formatDate(changing.occurrence.occurrenceDate)}" onclose={() => (changing = null)}>
		<form onsubmit={saveChange} class="grid grid-cols-2 gap-4">
			<label class="label">
				Valor
				<MoneyInput bind:value={changing.amount} />
			</label>
			<label class="label">
				Data
				<input type="date" bind:value={changing.date} class="input" />
			</label>
			<p class="col-span-2 muted">Altera apenas esta ocorrência, que pode ser movida até 31 dias.</p>
			<div class="col-span-2 flex justify-end gap-2">
				<button type="button" class="btn-secondary" onclick={() => (changing = null)}>Cancelar</button>
				<button type="submit" class="btn-primary">Guardar</button>
			</div>
		</form>
	</Modal>
{/if}
