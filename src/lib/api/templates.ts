import { gql } from '../graphql.ts';
import type { EntryKind } from './transactions.ts';

export type Template = {
	id: string;
	name: string;
	accountId: string;
	accountName: string;
	categoryId: string | null;
	categoryName: string | null;
	kind: EntryKind;
	amount: number | null;
	description: string;
};

export type TemplateInput = Omit<Template, 'id' | 'accountName' | 'categoryName'>;

const templateFields = 'id name accountId accountName categoryId categoryName kind amount description';

export async function listTemplates(): Promise<Template[]> {
	const data = await gql<{ templates: Template[] }>(`{ templates { ${templateFields} } }`);
	return data.templates;
}

export async function saveTemplate(id: string | null, input: TemplateInput): Promise<void> {
	if (id) {
		await gql(`mutation ($id: UUID!, $input: TemplateInput!) { updateTemplate(id: $id, input: $input) { id } }`, {
			id,
			input
		});
		return;
	}
	await gql(`mutation ($input: TemplateInput!) { createTemplate(input: $input) { id } }`, { input });
}

export async function deleteTemplate(id: string): Promise<void> {
	await gql(`mutation ($id: UUID!) { deleteTemplate(id: $id) }`, { id });
}
