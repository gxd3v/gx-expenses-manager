import { gql } from '../graphql.ts';

export type AccountKind = 'BANK' | 'SAVINGS' | 'CARD' | 'CASH' | 'OTHER';

export type Account = {
	id: string;
	name: string;
	kind: AccountKind;
	currency: string;
	initialBalance: number;
	color: string | null;
	icon: string | null;
	archivedAt: string | null;
	balance: number;
	availableBalance: number;
	projectedBalance: number;
};

export type AccountInput = Pick<Account, 'name' | 'kind' | 'currency' | 'initialBalance' | 'color' | 'icon'>;

export const accountKinds: Record<AccountKind, string> = {
	BANK: 'Conta bancária',
	SAVINGS: 'Poupança',
	CARD: 'Cartão',
	CASH: 'Dinheiro físico',
	OTHER: 'Outro'
};

const fields = 'id name kind currency initialBalance color icon archivedAt balance availableBalance projectedBalance';

export async function listAccounts(includeArchived = false): Promise<Account[]> {
	const data = await gql<{ accounts: Account[] }>(
		`query ($includeArchived: Boolean!) { accounts(includeArchived: $includeArchived) { ${fields} } }`,
		{ includeArchived }
	);
	return data.accounts;
}

export async function getAccount(id: string): Promise<Account> {
	const data = await gql<{ account: Account }>(`query ($id: UUID!) { account(id: $id) { ${fields} } }`, { id });
	return data.account;
}

export async function saveAccount(id: string | null, input: AccountInput): Promise<Account> {
	if (id) {
		const data = await gql<{ updateAccount: Account }>(
			`mutation ($id: UUID!, $input: AccountInput!) { updateAccount(id: $id, input: $input) { ${fields} } }`,
			{ id, input }
		);
		return data.updateAccount;
	}
	const data = await gql<{ createAccount: Account }>(
		`mutation ($input: AccountInput!) { createAccount(input: $input) { ${fields} } }`,
		{ input }
	);
	return data.createAccount;
}

export async function setAccountArchived(id: string, archived: boolean): Promise<void> {
	await gql(`mutation ($id: UUID!, $archived: Boolean!) { setAccountArchived(id: $id, archived: $archived) { id } }`, {
		id,
		archived
	});
}

export async function deleteAccount(id: string): Promise<void> {
	await gql(`mutation ($id: UUID!) { deleteAccount(id: $id) }`, { id });
}
