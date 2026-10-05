import { listOccurrences, resetOccurrence, skipOccurrence, type Occurrence } from './api/recurrences.ts';
import { deleteTransaction, listTransactions, saveTransaction, toInput, type Transaction } from './api/transactions.ts';
import { dataChanged, refs } from './refs.svelte.ts';
import { notify, notifyError } from './toasts.svelte.ts';

const MAX_SCHEDULED = 500;

export type Planned = {
	key: string;
	date: string;
	description: string;
	amount: number;
	credit: boolean;
	occurrence: Occurrence | null;
	transaction: Transaction | null;
};

export async function listPlanned(from: string, to: string): Promise<Planned[]> {
	if (from > to) return [];
	const [occurrences, scheduled] = await Promise.all([
		listOccurrences(from, to),
		listTransactions({ dateFrom: from, dateTo: to }, MAX_SCHEDULED)
	]);
	const ignored = new Set(refs.accounts.filter((a) => a.kind === 'MEAL').map((a) => a.id));
	return [
		...occurrences
			.filter((o) => !o.toAccountId && !ignored.has(o.accountId))
			.map((o) => ({
				key: `${o.recurrenceId}-${o.occurrenceDate}`,
				date: o.date,
				description: o.description,
				amount: o.kind === 'INCOME' ? o.amount : -o.amount,
				credit: o.creditId !== null,
				occurrence: o,
				transaction: null
			})),
		...scheduled.items
			.filter((t) => t.kind !== 'TRANSFER' && !ignored.has(t.accountId))
			.map((t) => ({
				key: t.id,
				date: t.date,
				description: t.description,
				amount: t.amount,
				credit: false,
				occurrence: null,
				transaction: t
			}))
	].sort((a, b) => a.date.localeCompare(b.date));
}

async function remove(item: Planned): Promise<() => Promise<void>> {
	const { occurrence, transaction } = item;
	if (occurrence) {
		await skipOccurrence(occurrence.recurrenceId, occurrence.occurrenceDate);
		return () => resetOccurrence(occurrence.recurrenceId, occurrence.occurrenceDate);
	}
	if (transaction) {
		await deleteTransaction(transaction.id);
		return async () => {
			await saveTransaction(null, toInput(transaction));
		};
	}
	return async () => {};
}

export async function dismissPlanned(item: Planned) {
	try {
		const undo = await remove(item);
		await dataChanged();
		notify('Movimento previsto removido', 'success', {
			label: 'Desfazer',
			run: async () => {
				await undo();
				await dataChanged();
			}
		});
	} catch (e) {
		notifyError(e);
	}
}
