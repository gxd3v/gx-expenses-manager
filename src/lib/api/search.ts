import { gql } from '../graphql.ts';
import type { Account } from './accounts.ts';
import type { Category } from './categories.ts';
import type { Credit } from './credits.ts';
import type { Goal } from './goals.ts';
import type { Transaction } from './transactions.ts';

export type SearchResult = {
	accounts: Pick<Account, 'id' | 'name' | 'balance' | 'currency'>[];
	categories: Pick<Category, 'id' | 'name'>[];
	transactions: Pick<Transaction, 'id' | 'description' | 'date' | 'amount' | 'currency' | 'accountName'>[];
	credits: Pick<Credit, 'id' | 'name' | 'remaining'>[];
	goals: Pick<Goal, 'id' | 'name' | 'progress'>[];
};

export async function search(text: string): Promise<SearchResult> {
	const data = await gql<{ search: SearchResult }>(
		`query ($text: String!) {
			search(text: $text) {
				accounts { id name balance currency }
				categories { id name }
				transactions { id description date amount currency accountName }
				credits { id name remaining }
				goals { id name progress }
			}
		}`,
		{ text }
	);
	return data.search;
}
