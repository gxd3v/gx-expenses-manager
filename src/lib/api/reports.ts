import { gql } from '../graphql.ts';
import type { EntryKind } from './transactions.ts';

export type MonthlyTotal = { month: string; income: number; outcome: number; net: number };
export type CategoryAmount = { categoryId: string | null; name: string; color: string | null; amount: number };
export type BalancePoint = { month: string; balance: number; debt: number; netWorth: number };

export type CategoryComparison = {
	categoryId: string | null;
	name: string;
	color: string | null;
	current: number;
	previous: number;
	average: number;
};

export type MonthSummary = {
	month: string;
	income: number;
	outcome: number;
	net: number;
	pendingIncome: number;
	pendingOutcome: number;
	expectedIncome: number;
	expectedOutcome: number;
	expectedNet: number;
	previousIncome: number;
	previousOutcome: number;
	averageIncome: number;
	averageOutcome: number;
	categories: CategoryComparison[];
};

export type RecordPeriod = 'ALL_TIME' | 'YEAR' | 'MONTH' | 'WEEK';
export type BalanceMark = { date: string; balance: number };
export type BalanceRecord = { period: RecordPeriod; high: BalanceMark; low: BalanceMark };

export type BalanceSummary = { total: number; available: number; projected: number; debt: number; netWorth: number };

export type MonthComparison = {
	categoryId: string | null;
	name: string;
	first: number;
	second: number;
	difference: number;
};

export type Alert = { key: string; kind: string; title: string; message: string; date: string | null };

export async function monthlyTotals(from: string, to: string, accountId: string | null = null): Promise<MonthlyTotal[]> {
	const data = await gql<{ monthlyTotals: MonthlyTotal[] }>(
		`query ($from: NaiveDate!, $to: NaiveDate!, $accountId: UUID) {
			monthlyTotals(from: $from, to: $to, accountId: $accountId) { month income outcome net }
		}`,
		{ from, to, accountId }
	);
	return data.monthlyTotals;
}

export async function categoryBreakdown(
	kind: EntryKind,
	from: string,
	to: string,
	grouping: 'PARENT' | 'LEAF' = 'PARENT',
	accountId: string | null = null
): Promise<CategoryAmount[]> {
	const data = await gql<{ categoryBreakdown: CategoryAmount[] }>(
		`query ($kind: EntryKind!, $from: NaiveDate!, $to: NaiveDate!, $grouping: CategoryGrouping!, $accountId: UUID) {
			categoryBreakdown(kind: $kind, from: $from, to: $to, grouping: $grouping, accountId: $accountId) {
				categoryId name color amount
			}
		}`,
		{ kind, from, to, grouping, accountId }
	);
	return data.categoryBreakdown;
}

export async function balanceHistory(months: number, accountId: string | null = null): Promise<BalancePoint[]> {
	const data = await gql<{ balanceHistory: BalancePoint[] }>(
		`query ($months: Int!, $accountId: UUID) { balanceHistory(months: $months, accountId: $accountId) { month balance debt netWorth } }`,
		{ months, accountId }
	);
	return data.balanceHistory;
}

export async function balanceRecords(accountId: string | null = null): Promise<BalanceRecord[]> {
	const data = await gql<{ balanceRecords: BalanceRecord[] }>(
		`query ($accountId: UUID) { balanceRecords(accountId: $accountId) { period high { date balance } low { date balance } } }`,
		{ accountId }
	);
	return data.balanceRecords;
}

export async function monthSummary(month: string): Promise<MonthSummary> {
	const data = await gql<{ monthSummary: MonthSummary }>(
		`query ($month: NaiveDate!) {
			monthSummary(month: $month) {
				month income outcome net pendingIncome pendingOutcome expectedIncome expectedOutcome expectedNet previousIncome previousOutcome averageIncome averageOutcome
				categories { categoryId name color current previous average }
			}
		}`,
		{ month }
	);
	return data.monthSummary;
}

export async function dismissAlert(key: string): Promise<void> {
	await gql(`mutation ($key: String!) { dismissAlert(key: $key) }`, { key });
}

export async function listAlerts(): Promise<Alert[]> {
	const data = await gql<{ alerts: Alert[] }>(`{ alerts { key kind title message date } }`);
	return data.alerts;
}

export async function balanceSummary(): Promise<BalanceSummary> {
	const data = await gql<{ balanceSummary: BalanceSummary }>(`{ balanceSummary { total available projected debt netWorth } }`);
	return data.balanceSummary;
}

export async function categoryAverages(kind: EntryKind, months: number): Promise<CategoryAmount[]> {
	const data = await gql<{ categoryAverages: CategoryAmount[] }>(
		`query ($kind: EntryKind!, $months: Int!) { categoryAverages(kind: $kind, months: $months) { categoryId name color amount } }`,
		{ kind, months }
	);
	return data.categoryAverages;
}

export async function compareMonths(first: string, second: string): Promise<MonthComparison[]> {
	const data = await gql<{ compareMonths: MonthComparison[] }>(
		`query ($first: NaiveDate!, $second: NaiveDate!) { compareMonths(first: $first, second: $second) { categoryId name first second difference } }`,
		{ first, second }
	);
	return data.compareMonths;
}
