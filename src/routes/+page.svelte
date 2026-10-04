<script lang="ts">
	import { untrack } from 'svelte';
	import { listAccounts } from '#lib/api/accounts.ts';
	import { listCredits } from '#lib/api/credits.ts';
	import { forecast } from '#lib/api/forecasts.ts';
	import { listGoals } from '#lib/api/goals.ts';
	import { listOccurrences } from '#lib/api/recurrences.ts';
	import { balanceHistory, categoryBreakdown, listAlerts, monthSummary } from '#lib/api/reports.ts';
	import { listTransactions } from '#lib/api/transactions.ts';
	import { describe } from '#lib/background.ts';
	import HBarChart from '#lib/charts/HBarChart.svelte';
	import LineChart from '#lib/charts/LineChart.svelte';
	import Amount from '#lib/components/Amount.svelte';
	import ProgressBar from '#lib/components/ProgressBar.svelte';
	import StatCard from '#lib/components/StatCard.svelte';
	import States from '#lib/components/States.svelte';
	import { addDays, formatDate, formatMoney, formatMonth, formatPercent, monthStart, today, weekStart } from '#lib/format.ts';
	import { refs } from '#lib/refs.svelte.ts';
	import { app } from '#lib/settings.svelte.ts';
	import { openQuickAdd } from '#lib/ui.svelte.ts';

	const UPCOMING_DAYS = 30;

	type Upcoming = { key: string; date: string; description: string; amount: number; credit: boolean };

	async function load() {
		const now = today();
		const until = addDays(now, UPCOMING_DAYS);
		const [accounts, summary, categories, occurrences, future, goals, projection, history, alerts, credits, week] = await Promise.all([
			listAccounts(),
			monthSummary(now),
			categoryBreakdown('OUTCOME', monthStart(now), now),
			listOccurrences(addDays(now, 1), until),
			listTransactions({ dateFrom: addDays(now, 1), dateTo: until }, 100),
			listGoals(),
			forecast({ months: app.settings?.forecastHorizonMonths ?? 6 }),
			balanceHistory(12),
			listAlerts(),
			listCredits(),
			listTransactions({ kind: 'OUTCOME', dateFrom: weekStart(now, app.settings?.firstDayOfWeek ?? 1), dateTo: now }, 1)
		]);

		const upcoming: Upcoming[] = [
			...occurrences.map((o) => ({
				key: `${o.recurrenceId}-${o.occurrenceDate}`,
				date: o.date,
				description: o.description,
				amount: o.kind === 'INCOME' ? o.amount : -o.amount,
				credit: o.creditId !== null
			})),
			...future.items
				.filter((t) => t.kind !== 'TRANSFER')
				.map((t) => ({ key: t.id, date: t.date, description: t.description, amount: t.amount, credit: false }))
		].sort((a, b) => a.date.localeCompare(b.date));

		return { accounts, summary, categories, upcoming, goals, projection, history, alerts, credits, weekOutcome: week.outcome };
	}

	let request = $state<ReturnType<typeof load>>(new Promise(() => {}));

	$effect(() => {
		refs.version;
		untrack(() => (request = load()));
	});
</script>

{#snippet upcomingList(items: Upcoming[], empty: string)}
	{#if items.length === 0}
		<p class="muted">{empty}</p>
	{:else}
		<ul class="divide-y divide-stone-100 text-sm dark:divide-stone-800">
			{#each items.slice(0, 6) as item (item.key)}
				<li class="flex justify-between gap-3 py-1.5">
					<span class="truncate"><span class="text-stone-500">{formatDate(item.date)}</span> · {item.description}</span>
					<Amount value={item.amount} />
				</li>
			{/each}
		</ul>
	{/if}
{/snippet}

{#await request}
	<States state="loading" />
{:then data}
	{@const total = data.accounts.reduce((sum, a) => sum + a.balance, 0)}
	<div class="mb-6 flex flex-wrap items-end justify-between gap-4">
		<div>
			<p class="muted">Saldo total</p>
			<p class="text-5xl font-semibold tabular-nums">{formatMoney(total)}</p>
		</div>
		<button class="btn-primary" onclick={() => openQuickAdd()}>+ Adicionar movimento</button>
	</div>

	{#if data.alerts.length}
		<section class="mb-6 rounded-xl border border-amber-200 bg-amber-50 p-4 dark:border-amber-900 dark:bg-amber-950" aria-label="Alertas">
			<h2 class="mb-2 text-sm font-medium text-amber-900 dark:text-amber-200">Avisos</h2>
			<ul class="grid gap-1 text-sm md:grid-cols-2">
				{#each data.alerts as alert (alert.key)}
					<li>⚠ <strong>{alert.title}:</strong> {describe(alert)}</li>
				{/each}
			</ul>
		</section>
	{/if}

	<div class="mb-6 grid gap-4 md:grid-cols-2 xl:grid-cols-4">
		<StatCard label="Despesas desta semana" value={formatMoney(data.weekOutcome)} />
		<StatCard label="Receitas do mês" value={formatMoney(data.summary.income)} hint="Previsto ainda: {formatMoney(data.summary.pendingIncome)}" />
		<StatCard label="Despesas do mês" value={formatMoney(data.summary.outcome)} hint="Previsto ainda: {formatMoney(data.summary.pendingOutcome)}" />
		<StatCard
			label="Resultado do mês"
			value={formatMoney(data.summary.net)}
			tone={data.summary.net >= 0 ? 'positive' : 'negative'}
		/>
	</div>

	<div class="mb-6 grid gap-4 xl:grid-cols-2">
		<section class="card">
			<h2 class="mb-3 font-medium">Saldo por conta</h2>
			{#if data.accounts.length === 0}
				<p class="muted">Ainda não tens contas. <a class="underline" href="/accounts">Criar conta</a></p>
			{:else}
				<ul class="divide-y divide-stone-100 text-sm dark:divide-stone-800">
					{#each data.accounts as account (account.id)}
						<li class="flex items-center justify-between py-1.5">
							<a href="/accounts/{account.id}" class="flex items-center gap-2 hover:underline">
								<span class="size-2.5 rounded-full" style:background-color={account.color ?? 'gray'}></span>
								{account.name}
							</a>
							<span class="tabular-nums">{formatMoney(account.balance, account.currency)}</span>
						</li>
					{/each}
				</ul>
			{/if}
		</section>
		<section class="card">
			<h2 class="mb-3 font-medium">Gastos por categoria (este mês)</h2>
			{#if data.categories.length === 0}
				<p class="muted">Sem despesas este mês.</p>
			{:else}
				<HBarChart items={data.categories.map((c) => ({ label: c.name, value: c.amount }))} format={(v) => formatMoney(v)} />
			{/if}
		</section>
	</div>

	<div class="mb-6 grid gap-4 xl:grid-cols-3">
		<section class="card">
			<h2 class="mb-3 font-medium">Próximas despesas</h2>
			{@render upcomingList(data.upcoming.filter((u) => u.amount < 0 && !u.credit), 'Nada nos próximos 30 dias.')}
		</section>
		<section class="card">
			<h2 class="mb-3 font-medium">Próximos rendimentos</h2>
			{@render upcomingList(data.upcoming.filter((u) => u.amount > 0), 'Nada nos próximos 30 dias.')}
		</section>
		<section class="card">
			<h2 class="mb-3 font-medium">Próximos pagamentos de créditos</h2>
			{#if data.credits.length === 0}
				<p class="muted">Sem créditos ativos.</p>
			{:else}
				<ul class="divide-y divide-stone-100 text-sm dark:divide-stone-800">
					{#each data.credits as credit (credit.id)}
						<li class="flex justify-between py-1.5">
							<a href="/credits/{credit.id}" class="hover:underline">{credit.name}</a>
							<span class="text-stone-500">{formatDate(credit.nextPaymentDate)} · {formatMoney(credit.installment)}</span>
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
								<span class="tabular-nums">{formatPercent(goal.progress)} · {formatMoney(goal.currentAmount)} / {formatMoney(goal.targetAmount)}</span>
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
{:catch error}
	<States state="error" {error} />
{/await}
