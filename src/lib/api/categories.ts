import { gql } from '../graphql.ts';

export type CategoryKind = 'INCOME' | 'OUTCOME' | 'BOTH';

export type Category = {
	id: string;
	parentId: string | null;
	name: string;
	kind: CategoryKind;
	icon: string | null;
	color: string | null;
	archivedAt: string | null;
	transactionCount: number;
};

export type CategoryInput = Pick<Category, 'parentId' | 'name' | 'kind' | 'icon' | 'color'>;

export const categoryKinds: Record<CategoryKind, string> = {
	INCOME: 'Receitas',
	OUTCOME: 'Despesas',
	BOTH: 'Ambos'
};

const fields = 'id parentId name kind icon color archivedAt transactionCount';

export async function listCategories(includeArchived = false): Promise<Category[]> {
	const data = await gql<{ categories: Category[] }>(
		`query ($includeArchived: Boolean!) { categories(includeArchived: $includeArchived) { ${fields} } }`,
		{ includeArchived }
	);
	return data.categories;
}

export async function saveCategory(id: string | null, input: CategoryInput): Promise<Category> {
	if (id) {
		const data = await gql<{ updateCategory: Category }>(
			`mutation ($id: UUID!, $input: CategoryInput!) { updateCategory(id: $id, input: $input) { ${fields} } }`,
			{ id, input }
		);
		return data.updateCategory;
	}
	const data = await gql<{ createCategory: Category }>(
		`mutation ($input: CategoryInput!) { createCategory(input: $input) { ${fields} } }`,
		{ input }
	);
	return data.createCategory;
}

export async function setCategoryArchived(id: string, archived: boolean): Promise<void> {
	await gql(`mutation ($id: UUID!, $archived: Boolean!) { setCategoryArchived(id: $id, archived: $archived) { id } }`, {
		id,
		archived
	});
}

export async function deleteCategory(id: string, reassignTo: string | null): Promise<void> {
	await gql(`mutation ($id: UUID!, $reassignTo: UUID) { deleteCategory(id: $id, reassignTo: $reassignTo) }`, {
		id,
		reassignTo
	});
}
