<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import { forecast, type Adjustment, type Forecast, type RecurrenceChange } from '#lib/api/forecasts.ts';
	import { listRecurrences, type Recurrence } from '#lib/api/recurrences.ts';
	import LineChart from '#lib/charts/LineChart.svelte';
	import AccountSelect from '#lib/components/AccountSelect.svelte';
	import Amount from '#lib/components/Amount.svelte';
	import MoneyInput from '#lib/components/MoneyInput.svelte';
	import PageHeader from '#lib/components/PageHeader.svelte';
	import StatCard from '#lib/components/StatCard.svelte';
	import States from '#lib/components/States.svelte';
	import { addMonths, formatMoney, formatMonth, monthStart, today } from '#lib/format.ts';
	import { refs } from '#lib/refs.svelte.ts';
	import { app, type ForecastMethod } from '#lib/settings.svelte.ts';

	type AdjustmentKind = 'expense' | 'income' | 'saving';
	type Draft = { kind: AdjustmentKind; accountId: string; toAccountId: string | null; amount: number | null; date: string; repeatMonths: number };

	const horizons = [1, 2, 4, 6, 12];

	let months = $state(untrack(() => app.settings?.forecastHorizonMonths ?? 6));
	let method = $state<ForecastMethod>(untrack(() => app.settings?.forecastMethod ?? 'HISTORY'));
	let historyMonths = $state(untrack(() => app.settings?.forecastHistoryMonths ?? 6));
	let adjustments = $state<(Adjustment & { label: string })[]>([]);
	let changes = $state<RecurrenceChange[]>([]);
	let recurrences = $state<Recurrence[]>([]);
	let draft = $state<Draft>(blankDraft());
	let request = $state<Promise<{ base: Forecast; scenario: Forecast | null }>>(new Promise(() => {}));

	function blankDraft(): Draft {
		return { kind: 'expense', accountId: refs.accounts[0]?.id ?? '', toAccountId: null, amount: null, date: addMonths(today(), 1), repeatMonths: 1 };
	}

	const hasScenario = $derived(adjustments.length > 0 || changes.length > 0);

	async function run() {
		const input = { months, method, historyMonths };
		const [base, scenario] = await Promise.all([
			forecast(input),
			hasScenario ? forecast({ ...input, adjustments: adjustments.map(({ label: _, ...a }) => a), recurrenceChanges: changes }) : null
		]);
		return { base, scenario };
	}

	$effect(() => {
		refs.version;
		void [months, method, historyMonths, adjustments.length, changes.length];
		untrack(() => (request = run()));
	});

	onMount(async () => {
		recurrences = await listRecurrences().catch(() => []);
	});

	function addAdjustment(event: SubmitEvent) {
		event.preventDefault();
		if (!draft.amount) return;
		const saving = draft.kind === 'saving';
		const sign = draft.kind === 'income' ? 1 : -1;
		const labels = { expense: 'Despesa', income: 'Receita', saving: 'Poupança' };
		adjustments.push({
			accountId: draft.accountId,
			toAccountId: saving ? draft.toAccountId : null,
			amount: saving ? draft.amount : sign * draft.amount,
			date: draft.date,
			repeatMonths: draft.repeatMonths,
			label: `${labels[draft.kind]} de ${formatMoney(draft.amount)} ${draft.repeatMonths > 1 ? `× ${draft.repeatMonths} meses` : ''} a partir de ${formatMonth(draft.date)}`
		});
		draft = blankDraft();
	}

	function changeRecurrence(recurrence: Recurrence, value: string) {
		changes = changes.filter((c) => c.recurrenceId !== recurrence.id);
		if (value === 'keep') return;
		const amount = value === 'remove' ? null : Math.round(Number(value.replace(',', '.')) * 100);
		changes.push({ recurrenceId: recurrence.id, amount });
	}

	const accountName = (id: string) => refs.accounts.find((a) => a.id === id)?.name ?? '—';
</script>

<PageHeader title="Previsões" subtitle="Evolução prevista com base no saldo atual, movimentos futuros, recorrências e histórico.">
	{#snippet actions()}
		{#each horizons as option (option)}
			<button class={months === option ? 'btn-primary' : 'btn-secondary'} onclick={() => (months = option)}>+{option} {option === 1 ? 'mês' : 'meses'}</button>
		{/each}
		<label class="flex items-center gap-1 text-sm">
			Custom
			<input type="number" min="1" max="120" bind:value={months} class="input w-20" />
		</label>
	{/snippet}
</PageHeader>

<div class="card mb-6 flex flex-wrap items-end gap-4">
	<label class="label">
		Método
		<select bind:value={method} class="input">
			<option value="HISTORY">Recorrências + média do histórico</option>
			<option value="RECURRING">Só recorrências e movimentos futuros</option>
		</select>
	</label>
	{#if method === 'HISTORY'}
		<label class="label">
			Meses de histórico
			<input type="number" min="1" max="24" bind:value={historyMonths} class="input w-24" />
		</label>
	{/if}
	<p class="muted max-w-md">
		Despesas fixas vêm das recorrências; as variáveis são estimadas pela média dos meses anteriores (movimentos sem recorrência).
	</p>
</div>

{#await request}
	<States state="loading" />
{:then { base, scenario }}
	{@const last = base.months[base.months.length - 1]}
	{@const income = base.months.reduce((s, m) => s + m.income, 0)}
	{@const outcome = base.months.reduce((s, m) => s + m.outcome, 0)}
	<div class="mb-6 grid gap-4 md:grid-cols-4">
		<StatCard label="Saldo total previsto" value={formatMoney(last?.total ?? 0)} hint={last ? formatMonth(last.month, 'long') : ''} />
		<StatCard label="Receitas previstas" value={formatMoney(income)} />
		<StatCard label="Despesas previstas" value={formatMoney(outcome)} />
		<StatCard label="Resultado previsto" value={formatMoney(income - outcome)} tone={income >= outcome ? 'positive' : 'negative'} />
	</div>

	<section class="card mb-6">
		<h2 class="mb-3 font-medium">Saldo total {scenario ? '— atual vs cenário' : ''}</h2>
		<LineChart
			labels={base.months.map((m) => formatMonth(m.month))}
			series={[
				{ name: 'Atual', color: 'var(--series-1)', values: base.months.map((m) => m.total) },
				...(scenario ? [{ name: 'Cenário', color: 'var(--series-2)', values: scenario.months.map((m) => m.total) }] : [])
			]}
			format={(v) => formatMoney(v)}
		/>
		{#if scenario && last}
			<p class="mt-2 text-sm">
				Diferença no fim do período: <Amount value={scenario.months[scenario.months.length - 1].total - last.total} />
			</p>
		{/if}
	</section>

	<div class="mb-6 grid gap-4 xl:grid-cols-3">
		<section class="card">
			<h2 class="mb-3 font-medium">Saldo previsto por conta</h2>
			<ul class="divide-y divide-stone-100 text-sm dark:divide-stone-800">
				{#each last?.balances ?? [] as balance (balance.accountId)}
					<li class="flex justify-between py-1.5"><span>{accountName(balance.accountId)}</span><span class="tabular-nums">{formatMoney(balance.balance)}</span></li>
				{/each}
			</ul>
		</section>
		<section class="card">
			<h2 class="mb-3 font-medium">Objetivos atingidos</h2>
			{#if base.goalsReached.length === 0}
				<p class="muted">Nenhum objetivo atingido neste horizonte.</p>
			{:else}
				<ul class="text-sm">
					{#each base.goalsReached as goal (goal.id)}<li>{goal.name} · {formatMonth(goal.date, 'long')}</li>{/each}
				</ul>
			{/if}
		</section>
		<section class="card">
			<h2 class="mb-3 font-medium">Créditos pagos</h2>
			{#if base.creditsPaid.length === 0}
				<p class="muted">Nenhum crédito termina neste horizonte.</p>
			{:else}
				<ul class="text-sm">
					{#each base.creditsPaid as credit (credit.id)}<li>{credit.name} · {formatMonth(credit.date, 'long')}</li>{/each}
				</ul>
			{/if}
		</section>
	</div>

	<section class="card mb-6 overflow-x-auto">
		<h2 class="mb-3 font-medium">Mês a mês</h2>
		<table class="table-base">
			<thead><tr><th>Mês</th><th class="text-right">Receitas</th><th class="text-right">Despesas</th><th class="text-right">Resultado</th><th class="text-right">Saldo total</th></tr></thead>
			<tbody>
				{#each base.months as month (month.month)}
					<tr>
						<td>{formatMonth(month.month, 'long')}</td>
						<td class="text-right tabular-nums">{formatMoney(month.income)}</td>
						<td class="text-right tabular-nums">{formatMoney(month.outcome)}</td>
						<td class="text-right"><Amount value={month.net} /></td>
						<td class="text-right tabular-nums">{formatMoney(month.total)}</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</section>
{:catch error}
	<States state="error" {error} />
{/await}

<section class="card">
	<h2 class="mb-1 font-medium">Cenário hipotético</h2>
	<p class="mb-4 muted">Adiciona despesas, receitas ou poupança e altera recorrências para comparar com a previsão atual.</p>

	<form onsubmit={addAdjustment} class="mb-4 grid grid-cols-2 gap-3 md:grid-cols-6">
		<label class="label">
			Tipo
			<select bind:value={draft.kind} class="input">
				<option value="expense">Despesa</option>
				<option value="income">Receita</option>
				<option value="saving">Poupança (transferência)</option>
			</select>
		</label>
		<label class="label">
			{draft.kind === 'saving' ? 'De' : 'Conta'}
			<AccountSelect bind:value={draft.accountId} />
		</label>
		{#if draft.kind === 'saving'}
			<label class="label">
				Para
				<AccountSelect bind:value={draft.toAccountId} exclude={draft.accountId} />
			</label>
		{/if}
		<label class="label">
			Valor
			<MoneyInput bind:value={draft.amount} required />
		</label>
		<label class="label">
			A partir de
			<input type="date" bind:value={draft.date} min={monthStart(today())} class="input" />
		</label>
		<label class="label">
			Repetir (meses)
			<input type="number" min="1" max="120" bind:value={draft.repeatMonths} class="input" />
		</label>
		<div class="flex items-end"><button class="btn-secondary w-full">Adicionar</button></div>
	</form>

	{#if adjustments.length}
		<ul class="mb-4 space-y-1 text-sm">
			{#each adjustments as adjustment, index (index)}
				<li class="flex items-center gap-2">
					<span class="flex-1">{adjustment.label}</span>
					<button class="btn-ghost" onclick={() => adjustments.splice(index, 1)} aria-label="Remover">✕</button>
				</li>
			{/each}
		</ul>
	{/if}

	{#if recurrences.length}
		<details>
			<summary class="cursor-pointer text-sm font-medium">Alterar recorrências</summary>
			<table class="table-base mt-2">
				<thead><tr><th>Recorrência</th><th class="text-right">Valor atual</th><th>No cenário</th></tr></thead>
				<tbody>
					{#each recurrences as recurrence (recurrence.id)}
						<tr>
							<td>{recurrence.description}</td>
							<td class="text-right tabular-nums">{formatMoney(recurrence.amount)}</td>
							<td>
								<select class="input" onchange={(e) => changeRecurrence(recurrence, e.currentTarget.value === 'custom' ? (window.prompt('Novo valor', '0,00') ?? 'keep') : e.currentTarget.value)}>
									<option value="keep">Manter</option>
									<option value="remove">Remover</option>
									<option value="custom">Outro valor…</option>
								</select>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</details>
	{/if}
</section>
