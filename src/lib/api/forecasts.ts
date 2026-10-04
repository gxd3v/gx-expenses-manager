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
	fixedIncome: number;
	fixedOutcome: number;
	variableIncome: number;
	variableOutcome: number;
	total: number;
	balances: { accountId: string; balance: number }[];
};

export type Milestone = { id: string; name: string; date: string };

export type Forecast = {
	totalIncome: number;
	totalOutcome: number;
	totalNet: number;
	months: ForecastMonth[];
	goalsReached: Milestone[];
	creditsPaid: Milestone[];
};

const forecastFields = `totalIncome totalOutcome totalNet
	months { month income outcome net fixedIncome fixedOutcome variableIncome variableOutcome total balances { accountId balance } }
	goalsReached { id name date }
	creditsPaid { id name date }`;

export type ScenarioComparison = { base: Forecast; scenario: Forecast; endDifference: number };

export async function forecast(input: ForecastRequest): Promise<Forecast> {
	const data = await gql<{ forecast: Forecast }>(`query ($input: ForecastInput!) { forecast(input: $input) { ${forecastFields} } }`, {
		input
	});
	return data.forecast;
}

export async function forecastScenario(input: ForecastRequest): Promise<ScenarioComparison> {
	const data = await gql<{ forecastScenario: ScenarioComparison }>(
		`query ($input: ForecastInput!) {
			forecastScenario(input: $input) { base { ${forecastFields} } scenario { ${forecastFields} } endDifference }
		}`,
		{ input }
	);
	return data.forecastScenario;
}
