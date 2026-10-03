import { gql } from './graphql';

export type AccountKind = 'BANK' | 'SAVINGS' | 'CARD' | 'CASH' | 'OTHER';

export type Account = {
	id: string;
	name: string;
	kind: AccountKind;
	currency: string;
	initialBalance: number;
	color: string | null;
	archivedAt: string | null;
};

export type AccountInput = Omit<Account, 'id' | 'archivedAt'>;

export const accountKinds: Record<AccountKind, string> = {
	BANK: 'Conta bancária',
	SAVINGS: 'Poupança',
	CARD: 'Cartão',
	CASH: 'Dinheiro',
	OTHER: 'Outro'
};

const fields = 'id name kind currency initialBalance color archivedAt';

export async function listAccounts(includeArchived: boolean): Promise<Account[]> {
	const data = await gql<{ accounts: Account[] }>(
		`query ($includeArchived: Boolean!) { accounts(includeArchived: $includeArchived) { ${fields} } }`,
		{ includeArchived }
	);
	return data.accounts;
}

export async function createAccount(input: AccountInput): Promise<Account> {
	const data = await gql<{ createAccount: Account }>(
		`mutation ($input: AccountInput!) { createAccount(input: $input) { ${fields} } }`,
		{ input }
	);
	return data.createAccount;
}

export async function updateAccount(id: string, input: AccountInput): Promise<Account> {
	const data = await gql<{ updateAccount: Account }>(
		`mutation ($id: UUID!, $input: AccountInput!) { updateAccount(id: $id, input: $input) { ${fields} } }`,
		{ id, input }
	);
	return data.updateAccount;
}

export async function archiveAccount(id: string): Promise<Account> {
	const data = await gql<{ archiveAccount: Account }>(
		`mutation ($id: UUID!) { archiveAccount(id: $id) { ${fields} } }`,
		{ id }
	);
	return data.archiveAccount;
}
