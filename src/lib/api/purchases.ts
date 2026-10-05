import { gql } from '../graphql.ts';

export type PlanMonth = { month: string; lowest: number; lowestAfter: number };

export type PurchasePlan = {
	balance: number;
	floor: number;
	lowestIfToday: number;
	earliestDate: string | null;
	months: PlanMonth[];
};

export type PlanOptions = { margin: number; allowOverdraft: boolean };

export type PurchasePlanInput = PlanOptions & { accountId: string; amount: number; months: number };

export type WishlistItem = {
	id: string;
	name: string;
	amount: number;
	accountId: string;
	accountName: string;
	priority: number;
	notes: string | null;
	purchasedAt: string | null;
	plannedDate: string | null;
};

export type WishlistInput = Pick<WishlistItem, 'name' | 'amount' | 'accountId' | 'priority' | 'notes'>;

export const priorities: Record<number, string> = { 1: 'Alta', 2: 'Média', 3: 'Baixa' };

const itemFields = 'id name amount accountId accountName priority notes purchasedAt plannedDate';

export async function purchasePlan(input: PurchasePlanInput): Promise<PurchasePlan> {
	const data = await gql<{ purchasePlan: PurchasePlan }>(
		`query ($input: PurchasePlanInput!) {
			purchasePlan(input: $input) { balance floor lowestIfToday earliestDate months { month lowest lowestAfter } }
		}`,
		{ input }
	);
	return data.purchasePlan;
}

export async function listWishlist(options: PlanOptions): Promise<WishlistItem[]> {
	const data = await gql<{ wishlist: WishlistItem[] }>(
		`query ($margin: Int!, $allowOverdraft: Boolean!) { wishlist(margin: $margin, allowOverdraft: $allowOverdraft) { ${itemFields} } }`,
		options
	);
	return data.wishlist;
}

export async function saveWishlistItem(id: string | null, input: WishlistInput): Promise<void> {
	if (id) {
		await gql(`mutation ($id: UUID!, $input: WishlistInput!) { updateWishlistItem(id: $id, input: $input) { id } }`, { id, input });
		return;
	}
	await gql(`mutation ($input: WishlistInput!) { createWishlistItem(input: $input) { id } }`, { input });
}

export async function setWishlistItemPurchased(id: string, purchased: boolean): Promise<void> {
	await gql(`mutation ($id: UUID!, $purchased: Boolean!) { setWishlistItemPurchased(id: $id, purchased: $purchased) { id } }`, {
		id,
		purchased
	});
}

export async function deleteWishlistItem(id: string): Promise<void> {
	await gql(`mutation ($id: UUID!) { deleteWishlistItem(id: $id) }`, { id });
}
