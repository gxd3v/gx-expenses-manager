import { gql } from '../graphql.ts';

export type ReconciliationStatus = {
	calculatedBalance: number;
	confirmedBalance: number;
	unconfirmedCount: number;
	unconfirmedTotal: number;
};

export type Reconciliation = {
	id: string;
	accountId: string;
	date: string;
	statementBalance: number;
	calculatedBalance: number;
	difference: number;
};

export type ForgottenCandidate = {
	description: string;
	categoryName: string | null;
	averageAmount: number;
	monthsSeen: number;
	lastDate: string;
};

export async function reconciliationStatus(accountId: string, date: string): Promise<ReconciliationStatus> {
	const data = await gql<{ reconciliationStatus: ReconciliationStatus }>(
		`query ($accountId: UUID!, $date: NaiveDate!) {
			reconciliationStatus(accountId: $accountId, date: $date) { calculatedBalance confirmedBalance unconfirmedCount unconfirmedTotal }
		}`,
		{ accountId, date }
	);
	return data.reconciliationStatus;
}

export async function listReconciliations(accountId: string): Promise<Reconciliation[]> {
	const data = await gql<{ reconciliations: Reconciliation[] }>(
		`query ($accountId: UUID!) { reconciliations(accountId: $accountId) { id accountId date statementBalance calculatedBalance difference } }`,
		{ accountId }
	);
	return data.reconciliations;
}

export async function reconcile(
	accountId: string,
	date: string,
	statementBalance: number,
	confirmFrom: string | null
): Promise<Reconciliation> {
	const data = await gql<{ reconcile: Reconciliation }>(
		`mutation ($accountId: UUID!, $date: NaiveDate!, $statementBalance: Int!, $confirmFrom: NaiveDate) {
			reconcile(accountId: $accountId, date: $date, statementBalance: $statementBalance, confirmFrom: $confirmFrom) {
				id accountId date statementBalance calculatedBalance difference
			}
		}`,
		{ accountId, date, statementBalance, confirmFrom }
	);
	return data.reconcile;
}

export async function forgottenTransactions(accountId: string, month: string): Promise<ForgottenCandidate[]> {
	const data = await gql<{ forgottenTransactions: ForgottenCandidate[] }>(
		`query ($accountId: UUID!, $month: NaiveDate!) {
			forgottenTransactions(accountId: $accountId, month: $month) { description categoryName averageAmount monthsSeen lastDate }
		}`,
		{ accountId, month }
	);
	return data.forgottenTransactions;
}
