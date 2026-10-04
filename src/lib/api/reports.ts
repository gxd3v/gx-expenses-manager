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
	previousIncome: number;
	previousOutcome: number;
	averageIncome: number;
	averageOutcome: number;
	categories: CategoryComparison[];
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

export async function monthSummary(month: string): Promise<MonthSummary> {
	const data = await gql<{ monthSummary: MonthSummary }>(
		`query ($month: NaiveDate!) {
			monthSummary(month: $month) {
				month income outcome net pendingIncome pendingOutcome previousIncome previousOutcome averageIncome averageOutcome
				categories { categoryId name color current previous average }
			}
		}`,
		{ month }
	);
	return data.monthSummary;
}

export async function listAlerts(): Promise<Alert[]> {
	const data = await gql<{ alerts: Alert[] }>(`{ alerts { key kind title message date } }`);
	return data.alerts;
}
