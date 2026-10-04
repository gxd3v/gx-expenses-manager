import { gql } from '../graphql.ts';
import type { EntryKind } from './transactions.ts';

export type FrequencyUnit = 'DAY' | 'WEEK' | 'MONTH' | 'YEAR';

export type Recurrence = {
	id: string;
	accountId: string;
	accountName: string;
	categoryId: string | null;
	categoryName: string | null;
	kind: EntryKind;
	amount: number;
	description: string;
	startDate: string;
	endDate: string | null;
	unit: FrequencyUnit;
	interval: number;
	pausedAt: string | null;
	creditId: string | null;
	toAccountId: string | null;
	toAccountName: string | null;
	nextDate: string | null;
};

export type RecurrenceInput = Pick<
	Recurrence,
	'accountId' | 'categoryId' | 'kind' | 'amount' | 'description' | 'startDate' | 'endDate' | 'unit' | 'interval' | 'toAccountId'
>;

export type Occurrence = {
	recurrenceId: string;
	occurrenceDate: string;
	date: string;
	amount: number;
	kind: EntryKind;
	accountId: string;
	categoryId: string | null;
	description: string;
	creditId: string | null;
	toAccountId: string | null;
	modified: boolean;
};

export const frequencyPresets = [
	{ label: 'Diária', unit: 'DAY', interval: 1 },
	{ label: 'Semanal', unit: 'WEEK', interval: 1 },
	{ label: 'Quinzenal', unit: 'WEEK', interval: 2 },
	{ label: 'Mensal', unit: 'MONTH', interval: 1 },
	{ label: 'Bimestral', unit: 'MONTH', interval: 2 },
	{ label: 'Trimestral', unit: 'MONTH', interval: 3 },
	{ label: 'Semestral', unit: 'MONTH', interval: 6 },
	{ label: 'Anual', unit: 'YEAR', interval: 1 }
] as const;

export const unitLabels: Record<FrequencyUnit, string> = { DAY: 'dias', WEEK: 'semanas', MONTH: 'meses', YEAR: 'anos' };

export function frequencyLabel(unit: FrequencyUnit, interval: number): string {
	const preset = frequencyPresets.find((p) => p.unit === unit && p.interval === interval);
	return preset ? preset.label : `A cada ${interval} ${unitLabels[unit]}`;
}

const fields =
	'id accountId accountName categoryId categoryName kind amount description startDate endDate unit interval pausedAt creditId toAccountId toAccountName nextDate';
const occurrenceFields = 'recurrenceId occurrenceDate date amount kind accountId categoryId description creditId toAccountId modified';

export async function listRecurrences(): Promise<Recurrence[]> {
	const data = await gql<{ recurrences: Recurrence[] }>(`{ recurrences { ${fields} } }`);
	return data.recurrences;
}

export async function listOccurrences(from: string, to: string, recurrenceId: string | null = null): Promise<Occurrence[]> {
	const data = await gql<{ occurrences: Occurrence[] }>(
		`query ($from: NaiveDate!, $to: NaiveDate!, $recurrenceId: UUID) {
			occurrences(from: $from, to: $to, recurrenceId: $recurrenceId) { ${occurrenceFields} }
		}`,
		{ from, to, recurrenceId }
	);
	return data.occurrences;
}

export async function saveRecurrence(id: string | null, input: RecurrenceInput): Promise<void> {
	if (id) {
		await gql(`mutation ($id: UUID!, $input: RecurrenceInput!) { updateRecurrence(id: $id, input: $input) { id } }`, {
			id,
			input
		});
		return;
	}
	await gql(`mutation ($input: RecurrenceInput!) { createRecurrence(input: $input) { id } }`, { input });
}

export async function recurrenceAction(action: 'pauseRecurrence' | 'resumeRecurrence' | 'deleteRecurrence', id: string) {
	const selection = action === 'deleteRecurrence' ? '' : '{ id }';
	await gql(`mutation ($id: UUID!) { ${action}(id: $id) ${selection} }`, { id });
}

export async function endRecurrence(id: string, endDate: string): Promise<void> {
	await gql(`mutation ($id: UUID!, $endDate: NaiveDate!) { endRecurrence(id: $id, endDate: $endDate) { id } }`, {
		id,
		endDate
	});
}

export async function skipOccurrence(recurrenceId: string, occurrenceDate: string): Promise<void> {
	await gql(
		`mutation ($recurrenceId: UUID!, $occurrenceDate: NaiveDate!) { skipOccurrence(recurrenceId: $recurrenceId, occurrenceDate: $occurrenceDate) }`,
		{ recurrenceId, occurrenceDate }
	);
}

export async function modifyOccurrence(
	recurrenceId: string,
	occurrenceDate: string,
	amount: number | null,
	date: string | null
): Promise<void> {
	await gql(
		`mutation ($recurrenceId: UUID!, $occurrenceDate: NaiveDate!, $amount: Int, $date: NaiveDate) {
			modifyOccurrence(recurrenceId: $recurrenceId, occurrenceDate: $occurrenceDate, amount: $amount, date: $date)
		}`,
		{ recurrenceId, occurrenceDate, amount, date }
	);
}

export async function resetOccurrence(recurrenceId: string, occurrenceDate: string): Promise<void> {
	await gql(
		`mutation ($recurrenceId: UUID!, $occurrenceDate: NaiveDate!) { resetOccurrence(recurrenceId: $recurrenceId, occurrenceDate: $occurrenceDate) }`,
		{ recurrenceId, occurrenceDate }
	);
}

export async function materializeDue(): Promise<number> {
	const data = await gql<{ materializeDueOccurrences: number }>(`mutation { materializeDueOccurrences }`);
	return data.materializeDueOccurrences;
}
