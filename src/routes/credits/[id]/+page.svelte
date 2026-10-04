<script lang="ts">
	import { untrack } from 'svelte';
	import { page } from '$app/state';
	import {
		createCreditRecurrence,
		creditHistory,
		deletePayment,
		getCredit,
		listPayments,
		registerPayment,
		simulateCredit,
		type AmortizationMode,
		type Simulation
	} from '#lib/api/credits.ts';
	import { listTransactions } from '#lib/api/transactions.ts';
	import LineChart from '#lib/charts/LineChart.svelte';
	import AccountSelect from '#lib/components/AccountSelect.svelte';
	import MoneyInput from '#lib/components/MoneyInput.svelte';
	import PageHeader from '#lib/components/PageHeader.svelte';
	import StatCard from '#lib/components/StatCard.svelte';
	import States from '#lib/components/States.svelte';
	import { confirmAction } from '#lib/dialogs.ts';
	import { addMonths, formatDate, formatMoney, formatMonth, today } from '#lib/format.ts';
	import { errorMessage } from '#lib/graphql.ts';
	import { dataChanged, refs } from '#lib/refs.svelte.ts';
	import { notify, notifyError } from '#lib/toasts.svelte.ts';

	const id = $derived(page.params.id ?? '');

	async function load(creditId: string) {
		const credit = await getCredit(creditId);
		const [payments, history, candidates] = await Promise.all([
			listPayments(creditId),
			creditHistory(creditId),
			credit.accountId
				? listTransactions({ accountId: credit.accountId, kind: 'OUTCOME', dateFrom: addMonths(today(), -3) }, 30)
				: Promise.resolve(null)
		]);
		return { credit, payments, history, candidates: candidates?.items ?? [] };
	}

	let request = $state<ReturnType<typeof load>>(new Promise(() => {}));
	let payment = $state({ amount: null as number | null, date: today(), accountId: null as string | null, transactionId: '' });
	let simulation = $state({ extra: null as number | null, mode: 'REDUCE_TERM' as AmortizationMode, installment: null as number | null });
	let simulationResult = $state<Simulation | null>(null);
	let showSchedule = $state(false);

	$effect(() => {
		refs.version;
		const creditId = id;
		untrack(() => (request = load(creditId)));
	});

	async function pay(event: SubmitEvent, creditId: string) {
		event.preventDefault();
		try {
			await registerPayment({
				creditId,
				date: payment.date,
				amount: payment.amount ?? 0,
				accountId: payment.accountId,
				transactionId: payment.transactionId || null,
				principal: null,
				interest: null
			});
			notify('Pagamento registado');
			payment = { amount: null, date: today(), accountId: null, transactionId: '' };
			await dataChanged();
		} catch (e) {
			notifyError(e);
		}
	}

	async function removePayment(paymentId: string) {
		if (!(await confirmAction('Eliminar este pagamento? O movimento associado mantém-se.'))) return;
		await deletePayment(paymentId).catch(notifyError);
		await dataChanged();
	}

	async function linkRecurrence(creditId: string) {
		try {
			await createCreditRecurrence(creditId);
			notify('Recorrência da prestação criada');
			await dataChanged();
		} catch (e) {
			notifyError(e);
		}
	}

	async function simulate(event: SubmitEvent, creditId: string) {
		event.preventDefault();
		try {
			simulationResult = await simulateCredit(creditId, simulation.extra ?? 0, simulation.mode, simulation.installment);
		} catch (e) {
			notify(errorMessage(e), 'error');
		}
	}
</script>

{#await request}
	<States state="loading" />
{:then { credit, payments, history, candidates }}
	<PageHeader title={credit.name} subtitle="{credit.institution ?? ''} · taxa {credit.annualRate.toLocaleString('pt-PT')} % · prestação {formatMoney(credit.installment)}">
		{#snippet actions()}
			{#if credit.recurrenceId}
				<a class="btn-secondary" href="/recurrences">Prestação automática ativa</a>
			{:else}
				<button class="btn-secondary" onclick={() => linkRecurrence(credit.id)} disabled={!credit.accountId} title={credit.accountId ? '' : 'Associa uma conta de pagamento ao crédito'}>
					Criar recorrência da prestação
				</button>
			{/if}
		{/snippet}
	</PageHeader>

	<div class="mb-6 grid gap-4 md:grid-cols-3 xl:grid-cols-6">
		<StatCard label="Capital inicial" value={formatMoney(credit.principal)} />
		<StatCard label="Capital em dívida" value={formatMoney(credit.remaining)} />
		<StatCard label="Capital pago" value={formatMoney(credit.principalPaid)} />
		<StatCard label="Juros pagos" value={formatMoney(credit.interestPaid)} hint="Dos pagamentos registados" />
		<StatCard label="Prestações restantes" value={String(credit.remainingInstallments)} />
		<StatCard label="Fim previsto" value={formatDate(credit.projectedEndDate)} />
	</div>

	<section class="card mb-6">
		<h2 class="mb-3 font-medium">Evolução do capital em dívida</h2>
		<LineChart
			labels={history.map((p) => (p.projected ? formatMonth(p.date) : formatDate(p.date)))}
			series={[{ name: 'Capital em dívida', color: 'var(--series-8)', values: history.map((p) => p.balance) }]}
			format={(v) => formatMoney(v)}
		/>
	</section>

	<div class="mb-6 grid gap-4 xl:grid-cols-2">
		<section class="card">
			<h2 class="mb-3 font-medium">Registar pagamento</h2>
			<form onsubmit={(e) => pay(e, credit.id)} class="grid grid-cols-2 gap-3">
				<label class="label col-span-2">
					Associar a um movimento existente
					<select bind:value={payment.transactionId} class="input">
						<option value="">Não — criar um movimento novo</option>
						{#each candidates as candidate (candidate.id)}
							<option value={candidate.id}>{formatDate(candidate.date)} · {candidate.description} · {formatMoney(-candidate.amount)}</option>
						{/each}
					</select>
				</label>
				{#if !payment.transactionId}
					<label class="label">
						Valor
						<MoneyInput bind:value={payment.amount} required />
					</label>
					<label class="label">
						Data
						<input type="date" bind:value={payment.date} class="input" />
					</label>
					<label class="label col-span-2">
						Conta
						<AccountSelect bind:value={payment.accountId} allowEmpty emptyLabel="Conta do crédito" />
					</label>
				{/if}
				<p class="col-span-2 muted">Capital e juros são calculados a partir do capital em dívida e da taxa.</p>
				<div class="col-span-2 flex justify-end"><button class="btn-primary">Registar</button></div>
			</form>
		</section>

		<section class="card">
			<h2 class="mb-3 font-medium">Simular amortização</h2>
			<form onsubmit={(e) => simulate(e, credit.id)} class="grid grid-cols-2 gap-3">
				<label class="label">
					Amortização antecipada
					<MoneyInput bind:value={simulation.extra} />
				</label>
				<label class="label">
					Efeito
					<select bind:value={simulation.mode} class="input">
						<option value="REDUCE_TERM">Reduzir prazo</option>
						<option value="REDUCE_INSTALLMENT">Reduzir prestação</option>
					</select>
				</label>
				<label class="label col-span-2">
					Nova prestação (opcional, para simular aumento/redução)
					<MoneyInput bind:value={simulation.installment} placeholder="Manter" />
				</label>
				<div class="col-span-2 flex justify-end"><button class="btn-secondary">Simular</button></div>
			</form>
			{#if simulationResult}
				<div class="overflow-x-auto">
				<table class="table-base mt-4">
					<thead><tr><th></th><th class="text-right">Atual</th><th class="text-right">Cenário</th></tr></thead>
					<tbody>
						<tr><td>Prestação</td><td class="text-right">{formatMoney(simulationResult.baseline.installment)}</td><td class="text-right">{formatMoney(simulationResult.scenario.installment)}</td></tr>
						<tr><td>Prestações</td><td class="text-right">{simulationResult.baseline.periods}</td><td class="text-right">{simulationResult.scenario.periods}</td></tr>
						<tr><td>Juros totais</td><td class="text-right">{formatMoney(simulationResult.baseline.totalInterest)}</td><td class="text-right">{formatMoney(simulationResult.scenario.totalInterest)}</td></tr>
						<tr><td>Fim</td><td class="text-right">{formatDate(simulationResult.baseline.endDate)}</td><td class="text-right">{formatDate(simulationResult.scenario.endDate)}</td></tr>
					</tbody>
				</table>
				</div>
				<p class="mt-2 text-sm">
					Poupança de juros: <strong>{formatMoney(simulationResult.interestSaved)}</strong> · {simulationResult.periodsSaved} prestações a menos
				</p>
			{/if}
		</section>
	</div>

	<section class="card mb-6">
		<h2 class="mb-3 font-medium">Histórico de pagamentos</h2>
		{#if payments.length === 0}
			<p class="muted">Ainda não há pagamentos registados.</p>
		{:else}
			<div class="overflow-x-auto">
			<table class="table-base">
				<thead><tr><th>Data</th><th class="text-right">Total</th><th class="text-right">Capital</th><th class="text-right">Juros</th><th></th></tr></thead>
				<tbody>
					{#each payments as item (item.id)}
						<tr>
							<td>{formatDate(item.date)}</td>
							<td class="text-right tabular-nums">{formatMoney(item.amount)}</td>
							<td class="text-right tabular-nums">{formatMoney(item.principal)}</td>
							<td class="text-right tabular-nums">{formatMoney(item.interest)}</td>
							<td class="text-right"><button class="btn-ghost text-red-600" onclick={() => removePayment(item.id)}>Eliminar</button></td>
						</tr>
					{/each}
				</tbody>
			</table>
			</div>
		{/if}
	</section>

	<section class="card">
		<button class="font-medium" onclick={() => (showSchedule = !showSchedule)} aria-expanded={showSchedule}>
			{showSchedule ? '▾' : '▸'} Plano de pagamentos previsto ({credit.schedule.length})
		</button>
		{#if showSchedule}
			<div class="mt-3 max-h-96 overflow-y-auto">
				<div class="overflow-x-auto">
				<table class="table-base">
					<thead><tr><th>#</th><th>Data</th><th class="text-right">Prestação</th><th class="text-right">Capital</th><th class="text-right">Juros</th><th class="text-right">Em dívida</th></tr></thead>
					<tbody>
						{#each credit.schedule as entry (entry.number)}
							<tr>
								<td>{entry.number}</td>
								<td>{formatDate(entry.date)}</td>
								<td class="text-right tabular-nums">{formatMoney(entry.installment)}</td>
								<td class="text-right tabular-nums">{formatMoney(entry.principal)}</td>
								<td class="text-right tabular-nums">{formatMoney(entry.interest)}</td>
								<td class="text-right tabular-nums">{formatMoney(entry.balance)}</td>
							</tr>
						{/each}
					</tbody>
				</table>
				</div>
			</div>
		{/if}
	</section>
{:catch error}
	<States state="error" {error} />
{/await}
