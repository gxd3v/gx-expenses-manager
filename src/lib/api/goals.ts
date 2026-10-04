import { gql } from '../graphql.ts';

export type Goal = {
	id: string;
	name: string;
	accountId: string;
	accountName: string;
	targetAmount: number;
	targetDate: string | null;
	archivedAt: string | null;
	currentAmount: number;
	remaining: number;
	progress: number;
	monthlyNeeded: number | null;
	projectedDate: string | null;
};

export type GoalInput = Pick<Goal, 'name' | 'accountId' | 'targetAmount' | 'targetDate'>;

const fields =
	'id name accountId accountName targetAmount targetDate archivedAt currentAmount remaining progress monthlyNeeded projectedDate';

export async function listGoals(includeArchived = false): Promise<Goal[]> {
	const data = await gql<{ goals: Goal[] }>(
		`query ($includeArchived: Boolean!) { goals(includeArchived: $includeArchived) { ${fields} } }`,
		{ includeArchived }
	);
	return data.goals;
}

export async function saveGoal(id: string | null, input: GoalInput): Promise<void> {
	if (id) {
		await gql(`mutation ($id: UUID!, $input: GoalInput!) { updateGoal(id: $id, input: $input) { id } }`, { id, input });
		return;
	}
	await gql(`mutation ($input: GoalInput!) { createGoal(input: $input) { id } }`, { input });
}

export async function setGoalArchived(id: string, archived: boolean): Promise<void> {
	await gql(`mutation ($id: UUID!, $archived: Boolean!) { setGoalArchived(id: $id, archived: $archived) { id } }`, {
		id,
		archived
	});
}

export async function deleteGoal(id: string): Promise<void> {
	await gql(`mutation ($id: UUID!) { deleteGoal(id: $id) }`, { id });
}
