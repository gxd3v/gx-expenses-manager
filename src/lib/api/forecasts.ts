import { gql } from '../graphql.ts';
import type { ForecastMethod } from '../settings.svelte.ts';

export type Adjustment = {
	accountId: string;
	toAccountId: string | null;
	amount: number;
	date: string;
	repeatMonths: number;
};

export type RecurrenceChange = { recurrenceId: string; amount: number | null };

export type ForecastRequest = {
	months: number;
	method?: ForecastMethod;
	historyMonths?: number;
	adjustments?: Adjustment[];
	recurrenceChanges?: RecurrenceChange[];
};

export type ForecastMonth = {
	month: string;
	income: number;
	outcome: number;
	net: number;
	total: number;
	balances: { accountId: string; balance: number }[];
};

export type Milestone = { id: string; name: string; date: string };

export type Forecast = {
	months: ForecastMonth[];
	goalsReached: Milestone[];
	creditsPaid: Milestone[];
};

export async function forecast(input: ForecastRequest): Promise<Forecast> {
	const data = await gql<{ forecast: Forecast }>(
		`query ($input: ForecastInput!) {
			forecast(input: $input) {
				months { month income outcome net total balances { accountId balance } }
				goalsReached { id name date }
				creditsPaid { id name date }
			}
		}`,
		{ input }
	);
	return data.forecast;
}
