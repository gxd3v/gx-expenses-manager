<script lang="ts">
	import { untrack } from 'svelte';
	import CreditForm from './CreditForm.svelte';
	import { deleteCredit, listCredits, setCreditArchived, type Credit } from '#lib/api/credits.ts';
	import Modal from '#lib/components/Modal.svelte';
	import PageHeader from '#lib/components/PageHeader.svelte';
	import ProgressBar from '#lib/components/ProgressBar.svelte';
	import States from '#lib/components/States.svelte';
	import Toggle from '#lib/components/Toggle.svelte';
	import { confirmAction } from '#lib/dialogs.ts';
	import { formatDate, formatMoney } from '#lib/format.ts';
	import { dataChanged, refs } from '#lib/refs.svelte.ts';
	import { notify, notifyError } from '#lib/toasts.svelte.ts';

	let showArchived = $state(false);
	let editing = $state<Credit | null | undefined>(undefined);
	let request = $state<Promise<Credit[]>>(new Promise(() => {}));

	function load() {
		request = listCredits(showArchived);
	}

	$effect(() => {
		refs.version;
		untrack(load);
	});

	async function toggleArchive(credit: Credit) {
		await setCreditArchived(credit.id, !credit.archivedAt).catch(notifyError);
		await dataChanged();
	}

	async function remove(credit: Credit) {
		if (!(await confirmAction(`Eliminar o crédito "${credit.name}" e o histórico de pagamentos?`))) return;
		await deleteCredit(credit.id).catch(notifyError);
		notify('Crédito eliminado');
		await dataChanged();
	}
</script>

<PageHeader title="Créditos" subtitle="Empréstimos, capital em dívida, juros e prestações.">
	{#snippet actions()}
		<Toggle bind:checked={showArchived} onchange={load} label="Mostrar arquivados" />
		<button class="btn-primary" onclick={() => (editing = null)}>Novo crédito</button>
	{/snippet}
</PageHeader>

{#await request}
	<States state="loading" />
{:then credits}
	{#if credits.length === 0}
		<States state="empty" message="Sem créditos registados.">
			<button class="btn-primary" onclick={() => (editing = null)}>Registar crédito</button>
		</States>
	{:else}
		<div class="grid gap-4 xl:grid-cols-2">
			{#each credits as credit (credit.id)}
				<article class="card space-y-3" class:opacity-60={credit.archivedAt}>
					<header class="flex items-start justify-between">
						<div>
							<a href="/credits/{credit.id}" class="font-medium hover:underline">{credit.name}</a>
							<p class="text-xs text-stone-500">{credit.institution ?? '—'} · {credit.annualRate.toLocaleString('pt-PT')} % · prestação {formatMoney(credit.installment)}</p>
						</div>
						<p class="text-right">
							<span class="block text-xs text-stone-500">Em dívida</span>
							<span class="font-semibold tabular-nums">{formatMoney(credit.remaining)}</span>
						</p>
					</header>
					<ProgressBar value={credit.principalPaid / credit.principal} label="Capital pago" />
					<dl class="grid grid-cols-3 gap-2 text-sm">
						<div><dt class="text-xs text-stone-500">Próximo pagamento</dt><dd>{formatDate(credit.nextPaymentDate)}</dd></div>
						<div><dt class="text-xs text-stone-500">Prestações restantes</dt><dd>{credit.remainingInstallments}</dd></div>
						<div><dt class="text-xs text-stone-500">Fim previsto</dt><dd>{formatDate(credit.projectedEndDate)}</dd></div>
					</dl>
					<div class="flex gap-1">
						<a class="btn-ghost" href="/credits/{credit.id}">Detalhe</a>
						<button class="btn-ghost" onclick={() => (editing = credit)}>Editar</button>
						<button class="btn-ghost" onclick={() => toggleArchive(credit)}>{credit.archivedAt ? 'Reativar' : 'Arquivar'}</button>
						<button class="btn-ghost text-red-600" onclick={() => remove(credit)}>Eliminar</button>
					</div>
				</article>
			{/each}
		</div>
	{/if}
{:catch error}
	<States state="error" {error} />
{/await}

{#if editing !== undefined}
	<Modal title={editing ? 'Editar crédito' : 'Novo crédito'} onclose={() => (editing = undefined)} wide>
		<CreditForm
			credit={editing}
			onsaved={async () => {
				editing = undefined;
				await dataChanged();
			}}
			oncancel={() => (editing = undefined)}
		/>
	</Modal>
{/if}
