<script lang="ts">
	import { untrack } from 'svelte';
	import { listAccounts } from '#lib/api/accounts.ts';
	import { listCredits } from '#lib/api/credits.ts';
	import { forecast } from '#lib/api/forecasts.ts';
	import { listGoals } from '#lib/api/goals.ts';
	import { balanceHistory, balanceSummary, categoryBreakdown, dismissAlert, listAlerts, monthSummary } from '#lib/api/reports.ts';
	import { listTransactions, type TransactionFilter } from '#lib/api/transactions.ts';
	import { describe } from '#lib/background.ts';
	import HBarChart from '#lib/charts/HBarChart.svelte';
	import LineChart from '#lib/charts/LineChart.svelte';
	import Amount from '#lib/components/Amount.svelte';
	import BalanceRecords from '#lib/components/BalanceRecords.svelte';
	import Money from '#lib/components/Money.svelte';
	import PendingConfirmations from '#lib/components/PendingConfirmations.svelte';
	import PrivacyToggle from '#lib/components/PrivacyToggle.svelte';
	import ProgressBar from '#lib/components/ProgressBar.svelte';
	import StatCard from '#lib/components/StatCard.svelte';
	import States from '#lib/components/States.svelte';
	import MonthDetail, { type Totals } from '#lib/components/MonthDetail.svelte';
	import TransactionsModal from '#lib/components/TransactionsModal.svelte';
	import { listPlanned, type Planned } from '#lib/planned.ts';
	import { addDays, formatDate, formatMoney, formatMonth, formatPercent, monthEnd, monthStart, today, weekStart } from '#lib/format.ts';
	import { dataChanged, refs } from '#lib/refs.svelte.ts';
	import { app } from '#lib/settings.svelte.ts';
	import { notifyError } from '#lib/toasts.svelte.ts';
	import { openQuickAdd } from '#lib/ui.svelte.ts';

	const UPCOMING_DAYS = 30;


	async function load() {
		const now = today();
		const until = addDays(now, UPCOMING_DAYS);
		const [accounts, summary, categories, upcoming, goals, projection, history, alerts, credits, balances, week] = await Promise.all([
			listAccounts(),
			monthSummary(now),
			categoryBreakdown('OUTCOME', monthStart(now), now),
			listPlanned(addDays(now, 1), until),
			listGoals(),
			forecast({ months: app.settings?.forecastHorizonMonths ?? 6 }),
			balanceHistory(12),
			listAlerts(),
			listCredits(),
			balanceSummary(),
			listTransactions({ kind: 'OUTCOME', dateFrom: weekStart(now, app.settings?.firstDayOfWeek ?? 1), dateTo: now }, 1)
		]);

		return { accounts, summary, categories, upcoming, goals, projection, history, alerts, credits, weekOutcome: week.outcome, balances };
	}

	let request = $state<ReturnType<typeof load>>(new Promise(() => {}));
	let detail = $state<{ title: string; filter: TransactionFilter } | null>(null);
	let monthDetail = $state<{ title: string; sign: 1 | -1 | 0; totals: Totals } | null>(null);

	async function dismiss(key: string) {
		try {
			await dismissAlert(key);
			await dataChanged();
		} catch (e) {
			notifyError(e);
		}
	}

	function showDetail(title: string, kind: 'INCOME' | 'OUTCOME', dateFrom: string) {
		detail = { title, filter: { kind, dateFrom, dateTo: today() } };
	}

	$effect(() => {
		refs.version;
		untrack(() => (request = load()));
	});
</script>

{#snippet cardTitle(title: string, total: number, signed: boolean)}
	<h2 class="mb-3 flex items-baseline justify-between gap-3 font-medium">
		<span>{title}</span>
		<span class="text-sm">
			{#if signed}<Amount value={total} />{:else}<Money value={total} />{/if}
		</span>
	</h2>
{/snippet}

{#snippet upcomingCard(title: string, items: Planned[])}
	<section class="card">
		{@render cardTitle(title, items.reduce((sum, item) => sum + item.amount, 0), true)}
		{#if items.length === 0}
			<p class="muted">Nada nos próximos 30 dias.</p>
		{:else}
			<ul class="max-h-64 divide-y divide-stone-100 overflow-y-auto text-sm dark:divide-stone-800">
				{#each items as item (item.key)}
					<li class="flex justify-between gap-3 py-1.5">
						<span class="truncate"><span class="text-stone-500">{formatDate(item.date)}</span> · {item.description}</span>
						<Amount value={item.amount} />
					</li>
				{/each}
			</ul>
		{/if}
	</section>
{/snippet}

{#await request}
	<States state="loading" />
{:then data}
	<div class="mb-6 flex flex-wrap items-end justify-between gap-4">
		<div>
			<p class="muted flex items-center gap-1">Saldo total <PrivacyToggle /></p>
			<p class="text-5xl font-semibold tabular-nums"><Money value={data.balances.total} /></p>
			<p class="muted">Património líquido <Money value={data.balances.netWorth} /> · dívida em créditos <Money value={data.balances.debt} /></p>
		</div>
		<button class="btn-primary" onclick={() => openQuickAdd()}>+ Adicionar movimento</button>
	</div>

	<PendingConfirmations />

	{#if data.alerts.length}
		<section class="mb-6 rounded-xl border border-amber-200 bg-amber-50 p-4 dark:border-amber-900 dark:bg-amber-950" aria-label="Alertas">
			<h2 class="mb-2 text-sm font-medium text-amber-900 dark:text-amber-200">Avisos</h2>
			<ul class="divide-y divide-amber-200/60 text-sm dark:divide-amber-900/60">
				{#each data.alerts as alert (alert.key)}
					<li class="flex items-center justify-between gap-3 py-1">
						<span class="min-w-0">⚠ <strong>{alert.title}:</strong> {describe(alert)}</span>
						<button
							class="shrink-0 rounded p-1 text-xs leading-none text-amber-900 hover:bg-amber-200/60 dark:text-amber-200 dark:hover:bg-amber-900"
							onclick={() => dismiss(alert.key)}
							aria-label="Dispensar aviso"
							title="Dispensar">✕</button
						>
					</li>
				{/each}
			</ul>
		</section>
	{/if}

	<div class="mb-6 grid gap-4 md:grid-cols-2 xl:grid-cols-4">
		<StatCard
			label="Despesas desta semana"
			value={formatMoney(data.weekOutcome)}
			ondetail={() => showDetail('Despesas desta semana', 'OUTCOME', weekStart(today(), app.settings?.firstDayOfWeek ?? 1))}
		/>
		<StatCard
			privateHint
			label="Receitas do mês"
			value={formatMoney(data.summary.income)}
			hint="Previsto ainda: {formatMoney(data.summary.pendingIncome)}"
			ondetail={() =>
				(monthDetail = {
					title: 'Receitas do mês',
					sign: 1,
					totals: { realized: data.summary.income, planned: data.summary.pendingIncome, expected: data.summary.expectedIncome }
				})}
		/>
		<StatCard
			privateHint
			label="Despesas do mês"
			value={formatMoney(data.summary.outcome)}
			hint="Previsto ainda: {formatMoney(data.summary.pendingOutcome)}"
			ondetail={() =>
				(monthDetail = {
					title: 'Despesas do mês',
					sign: -1,
					totals: { realized: -data.summary.outcome, planned: -data.summary.pendingOutcome, expected: -data.summary.expectedOutcome }
				})}
		/>
		<StatCard
			label="Resultado do mês"
			value={formatMoney(data.summary.net)}
			tone={data.summary.net >= 0 ? 'positive' : 'negative'}
			privateHint
			hint="Com o previsto: {formatMoney(data.summary.expectedNet)}"
			ondetail={() =>
				(monthDetail = {
					title: 'Resultado do mês',
					sign: 0,
					totals: {
						realized: data.summary.net,
						planned: data.summary.pendingIncome - data.summary.pendingOutcome,
						expected: data.summary.expectedNet
					}
				})}
		/>
	</div>

	<div class="mb-6 grid gap-4 lg:grid-cols-2 xl:grid-cols-3">
		<section class="card">
			{@render cardTitle('Saldo por conta', data.balances.total, false)}
			{#if data.accounts.length === 0}
				<p class="muted">Sem contas. <a class="underline" href="/accounts">Criar conta</a></p>
			{:else}
				<ul class="divide-y divide-stone-100 text-sm dark:divide-stone-800">
					{#each data.accounts as account (account.id)}
						<li class="flex items-center justify-between py-1.5">
							<a href="/accounts/{account.id}" class="flex items-center gap-2 hover:underline">
								<span class="size-2.5 rounded-full" style:background-color={account.color ?? 'gray'}></span>
								{account.name}
								{#if account.kind === 'CREDIT_CARD' || account.kind === 'MEAL'}<span class="badge">fora do total</span>{/if}
							</a>
							<span class="tabular-nums"><Money value={account.balance} currency={account.currency} /></span>
						</li>
					{/each}
				</ul>
			{/if}
		</section>
		<section class="card">
			{@render cardTitle('Gastos por categoria (este mês)', data.categories.reduce((sum, c) => sum + c.amount, 0), false)}
			{#if data.categories.length === 0}
				<p class="muted">Sem despesas este mês.</p>
			{:else}
				<HBarChart
					items={data.categories.map((c) => ({ label: c.name, value: c.amount, color: c.color }))}
					format={(v) => formatMoney(v)}
					limit={data.categories.length}
				/>
			{/if}
		</section>
		<BalanceRecords selectable />
	</div>

	<div class="mb-6 grid gap-4 xl:grid-cols-3">
		{@render upcomingCard('Próximas despesas', data.upcoming.filter((u) => u.amount < 0 && !u.credit))}
		{@render upcomingCard('Próximos rendimentos', data.upcoming.filter((u) => u.amount > 0))}
		<section class="card">
			{@render cardTitle('Próximos pagamentos de créditos', -data.credits.reduce((sum, c) => sum + c.installment, 0), true)}
			{#if data.credits.length === 0}
				<p class="muted">Sem créditos ativos.</p>
			{:else}
				<ul class="divide-y divide-stone-100 text-sm dark:divide-stone-800">
					{#each data.credits as credit (credit.id)}
						<li class="flex justify-between py-1.5">
							<a href="/credits/{credit.id}" class="hover:underline">{credit.name}</a>
							<span class="text-stone-500">{formatDate(credit.nextPaymentDate)} · <Money value={credit.installment} /></span>
						</li>
					{/each}
				</ul>
			{/if}
		</section>
	</div>

	<div class="mb-6 grid gap-4 xl:grid-cols-2">
		<section class="card">
			<h2 class="mb-3 font-medium">Objetivos em progresso</h2>
			{#if data.goals.length === 0}
				<p class="muted">Sem objetivos. <a class="underline" href="/goals">Criar objetivo</a></p>
			{:else}
				<ul class="space-y-3">
					{#each data.goals as goal (goal.id)}
						<li class="text-sm">
							<div class="mb-1 flex justify-between">
								<span>{goal.name}</span>
								<span class="tabular-nums">{formatPercent(goal.progress)} · <Money value={goal.currentAmount} /> / <Money value={goal.targetAmount} /></span>
							</div>
							<ProgressBar value={goal.progress} label={goal.name} />
						</li>
					{/each}
				</ul>
			{/if}
		</section>
		<section class="card">
			<h2 class="mb-3 font-medium">Previsão de saldo</h2>
			<LineChart
				labels={data.projection.months.map((m) => formatMonth(m.month))}
				series={[{ name: 'Saldo total previsto', color: 'var(--series-1)', values: data.projection.months.map((m) => m.total) }]}
				format={(v) => formatMoney(v)}
				height={200}
			/>
		</section>
	</div>

	<section class="card">
		<h2 class="mb-3 font-medium">Evolução financeira (12 meses)</h2>
		<LineChart
			labels={data.history.map((p) => formatMonth(p.month))}
			series={[
				{ name: 'Saldo das contas', color: 'var(--series-1)', values: data.history.map((p) => p.balance) },
				{ name: 'Património líquido', color: 'var(--series-2)', values: data.history.map((p) => p.netWorth) }
			]}
			format={(v) => formatMoney(v)}
		/>
	</section>
	{#if monthDetail}
		<MonthDetail
			title={monthDetail.title}
			sign={monthDetail.sign}
			start={monthStart(today())}
			end={monthEnd(today())}
			planned={data.upcoming.filter((u) => u.date <= monthEnd(today()))}
			totals={monthDetail.totals}
			onclose={() => (monthDetail = null)}
		/>
	{/if}
{:catch error}
	<States state="error" {error} />
{/await}

{#if detail}
	<TransactionsModal title={detail.title} filter={detail.filter} onclose={() => (detail = null)} />
{/if}
