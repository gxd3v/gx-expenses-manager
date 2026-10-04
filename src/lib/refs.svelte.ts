import { listAccounts, type Account } from './api/accounts.ts';
import { listCategories, type Category } from './api/categories.ts';

export const refs = $state<{ accounts: Account[]; categories: Category[]; version: number }>({
	accounts: [],
	categories: [],
	version: 0
});

export async function loadRefs() {
	const [accounts, categories] = await Promise.all([listAccounts(), listCategories()]);
	refs.accounts = accounts;
	refs.categories = categories;
}

export async function dataChanged() {
	await loadRefs();
	refs.version++;
}
