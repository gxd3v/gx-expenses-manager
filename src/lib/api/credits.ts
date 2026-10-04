import { gql } from '../graphql.ts';
import type { FrequencyUnit } from './recurrences.ts';

export type ScheduleEntry = {
	number: number;
	date: string;
	installment: number;
	principal: number;
	interest: number;
	balance: number;
};

export type Credit = {
	id: string;
	name: string;
	institution: string | null;
	principal: number;
	openingBalance: number;
	annualRate: number;
	installment: number;
	unit: FrequencyUnit;
	interval: number;
	startDate: string;
	endDate: string | null;
	installments: number | null;
	accountId: string | null;
	archivedAt: string | null;
	paymentsCount: number;
	remaining: number;
	principalPaid: number;
	interestPaid: number;
	nextPaymentDate: string | null;
	remainingInstallments: number;
	projectedEndDate: string | null;
	schedule: ScheduleEntry[];
};

export type CreditInput = Pick<
	Credit,
	| 'name'
	| 'institution'
	| 'principal'
	| 'openingBalance'
	| 'annualRate'
	| 'installment'
	| 'unit'
	| 'interval'
	| 'startDate'
	| 'endDate'
	| 'installments'
	| 'accountId'
>;

export type CreditPayment = {
	id: string;
	creditId: string;
	transactionId: string | null;
	date: string;
	principal: number;
	interest: number;
	amount: number;
};

export type PaymentInput = {
	creditId: string;
	date: string;
	amount: number;
	accountId: string | null;
	transactionId: string | null;
	principal: number | null;
	interest: number | null;
};

export type AmortizationMode = 'REDUCE_TERM' | 'REDUCE_INSTALLMENT';

export type ScheduleSummary = { installment: number; periods: number; totalInterest: number; endDate: string | null };

export type Simulation = {
	baseline: ScheduleSummary;
	scenario: ScheduleSummary;
	interestSaved: number;
	periodsSaved: number;
};

const baseFields = `id name institution principal openingBalance annualRate installment unit interval startDate endDate
	installments accountId archivedAt paymentsCount remaining principalPaid interestPaid nextPaymentDate
	remainingInstallments projectedEndDate`;
const scheduleFields = 'schedule { number date installment principal interest balance }';
const summaryFields = 'installment periods totalInterest endDate';

export async function listCredits(includeArchived = false): Promise<Credit[]> {
	const data = await gql<{ credits: Credit[] }>(
		`query ($includeArchived: Boolean!) { credits(includeArchived: $includeArchived) { ${baseFields} } }`,
		{ includeArchived }
	);
	return data.credits;
}

export async function getCredit(id: string): Promise<Credit> {
	const data = await gql<{ credit: Credit }>(`query ($id: UUID!) { credit(id: $id) { ${baseFields} ${scheduleFields} } }`, {
		id
	});
	return data.credit;
}

export async function saveCredit(id: string | null, input: CreditInput, createRecurrence = false): Promise<Credit> {
	if (id) {
		const data = await gql<{ updateCredit: Credit }>(
			`mutation ($id: UUID!, $input: CreditInput!) { updateCredit(id: $id, input: $input) { ${baseFields} } }`,
			{ id, input }
		);
		return data.updateCredit;
	}
	const data = await gql<{ createCredit: Credit }>(
		`mutation ($input: CreditInput!, $createRecurrence: Boolean!) {
			createCredit(input: $input, createRecurrence: $createRecurrence) { ${baseFields} }
		}`,
		{ input, createRecurrence }
	);
	return data.createCredit;
}

export async function setCreditArchived(id: string, archived: boolean): Promise<void> {
	await gql(`mutation ($id: UUID!, $archived: Boolean!) { setCreditArchived(id: $id, archived: $archived) { id } }`, {
		id,
		archived
	});
}

export async function deleteCredit(id: string): Promise<void> {
	await gql(`mutation ($id: UUID!) { deleteCredit(id: $id) }`, { id });
}

export async function listPayments(creditId: string): Promise<CreditPayment[]> {
	const data = await gql<{ creditPayments: CreditPayment[] }>(
		`query ($creditId: UUID) { creditPayments(creditId: $creditId) { id creditId transactionId date principal interest amount } }`,
		{ creditId }
	);
	return data.creditPayments;
}

export async function registerPayment(input: PaymentInput): Promise<void> {
	await gql(`mutation ($input: PaymentInput!) { registerCreditPayment(input: $input) { id } }`, { input });
}

export async function deletePayment(id: string): Promise<void> {
	await gql(`mutation ($id: UUID!) { deleteCreditPayment(id: $id) }`, { id });
}

export async function simulateCredit(
	id: string,
	extraPayment: number,
	mode: AmortizationMode,
	installment: number | null
): Promise<Simulation> {
	const data = await gql<{ simulateCredit: Simulation }>(
		`query ($id: UUID!, $input: SimulationInput!) {
			simulateCredit(id: $id, input: $input) {
				baseline { ${summaryFields} } scenario { ${summaryFields} } interestSaved periodsSaved
			}
		}`,
		{ id, input: { extraPayment, mode, installment } }
	);
	return data.simulateCredit;
}
