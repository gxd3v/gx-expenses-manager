import { gql } from '../graphql.ts';

export type TransactionKind = 'INCOME' | 'OUTCOME' | 'TRANSFER';
export type EntryKind = 'INCOME' | 'OUTCOME';

export type Transaction = {
	id: string;
	accountId: string;
	accountName: string;
	currency: string;
	categoryId: string | null;
	categoryName: string | null;
	kind: TransactionKind;
	amount: number;
	date: string;
	description: string;
	notes: string | null;
	confirmed: boolean;
	transferId: string | null;
	recurrenceId: string | null;
	counterpartAccountId: string | null;
	counterpartAccountName: string | null;
};

export type TransactionInput = {
	accountId: string;
	categoryId: string | null;
	kind: EntryKind;
	amount: number;
	date: string;
	description: string;
	notes: string | null;
	confirmed: boolean;
};

export type TransactionFilter = {
	accountId?: string | null;
	categoryId?: string | null;
	kind?: TransactionKind | null;
	dateFrom?: string | null;
	dateTo?: string | null;
	search?: string | null;
	minAmount?: number | null;
	maxAmount?: number | null;
	confirmed?: boolean | null;
};

export type TransactionPage = {
	items: Transaction[];
	totalCount: number;
	income: number;
	outcome: number;
	net: number;
};

export type Transfer = {
	id: string;
	fromAccountId: string;
	toAccountId: string;
	amount: number;
	date: string;
	description: string;
};

export type TransferInput = Omit<Transfer, 'id'>;

export type SavedFilter = { id: string; name: string; filter: string };

export const transactionKinds: Record<TransactionKind, string> = {
	INCOME: 'Receita',
	OUTCOME: 'Despesa',
	TRANSFER: 'Transferência'
};

const fields =
	'id accountId accountName currency categoryId categoryName kind amount date description notes confirmed transferId recurrenceId counterpartAccountId counterpartAccountName';

export function cleanFilter(filter: TransactionFilter): TransactionFilter {
	return Object.fromEntries(Object.entries(filter).filter(([, value]) => value !== null && value !== '' && value !== undefined));
}

export async function listTransactions(filter: TransactionFilter, limit = 50, offset = 0): Promise<TransactionPage> {
	const data = await gql<{ transactions: TransactionPage }>(
		`query ($filter: TransactionFilter!, $limit: Int!, $offset: Int!) {
			transactions(filter: $filter, limit: $limit, offset: $offset) { items { ${fields} } totalCount income outcome net }
		}`,
		{ filter: cleanFilter(filter), limit, offset }
	);
	return data.transactions;
}

export async function pendingConfirmations(): Promise<Transaction[]> {
	const data = await gql<{ pendingConfirmations: Transaction[] }>(`{ pendingConfirmations { ${fields} } }`);
	return data.pendingConfirmations;
}

export async function saveTransaction(id: string | null, input: TransactionInput): Promise<Transaction> {
	if (id) {
		const data = await gql<{ updateTransaction: Transaction }>(
			`mutation ($id: UUID!, $input: TransactionInput!) { updateTransaction(id: $id, input: $input) { ${fields} } }`,
			{ id, input }
		);
		return data.updateTransaction;
	}
	const data = await gql<{ createTransaction: Transaction }>(
		`mutation ($input: TransactionInput!) { createTransaction(input: $input) { ${fields} } }`,
		{ input }
	);
	return data.createTransaction;
}

export async function deleteTransaction(id: string): Promise<void> {
	await gql(`mutation ($id: UUID!) { deleteTransaction(id: $id) }`, { id });
}

export async function setConfirmed(ids: string[], confirmed: boolean): Promise<void> {
	await gql(`mutation ($ids: [UUID!]!, $confirmed: Boolean!) { setTransactionsConfirmed(ids: $ids, confirmed: $confirmed) }`, {
		ids,
		confirmed
	});
}

export async function getTransfer(id: string): Promise<Transfer> {
	const data = await gql<{ transfer: Transfer }>(
		`query ($id: UUID!) { transfer(id: $id) { id fromAccountId toAccountId amount date description } }`,
		{ id }
	);
	return data.transfer;
}

export async function saveTransfer(id: string | null, transfer: TransferInput): Promise<void> {
	const { fromAccountId, toAccountId, amount, date, description } = transfer;
	const input = { fromAccountId, toAccountId, amount, date, description };
	if (id) {
		await gql(`mutation ($id: UUID!, $input: TransferInput!) { updateTransfer(id: $id, input: $input) { id } }`, { id, input });
		return;
	}
	await gql(`mutation ($input: TransferInput!) { createTransfer(input: $input) { id } }`, { input });
}

export async function listSavedFilters(): Promise<SavedFilter[]> {
	const data = await gql<{ savedFilters: SavedFilter[] }>(`{ savedFilters { id name filter } }`);
	return data.savedFilters;
}

export async function saveFilter(name: string, filter: TransactionFilter): Promise<void> {
	await gql(`mutation ($name: String!, $filter: String!) { saveFilter(name: $name, filter: $filter) { id } }`, {
		name,
		filter: JSON.stringify(cleanFilter(filter))
	});
}

export async function deleteSavedFilter(id: string): Promise<void> {
	await gql(`mutation ($id: UUID!) { deleteSavedFilter(id: $id) }`, { id });
}

export function toInput(transaction: Transaction): TransactionInput {
	return {
		accountId: transaction.accountId,
		categoryId: transaction.categoryId,
		kind: transaction.amount >= 0 ? 'INCOME' : 'OUTCOME',
		amount: Math.abs(transaction.amount),
		date: transaction.date,
		description: transaction.description,
		notes: transaction.notes,
		confirmed: transaction.confirmed
	};
}
