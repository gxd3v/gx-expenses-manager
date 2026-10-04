import type { TransactionInput } from './api/transactions.ts';

type QuickAddState = { mode: 'transaction' | 'transfer'; initial: Partial<TransactionInput> | null };

export const ui = $state<{ quickAdd: QuickAddState | null; search: boolean }>({ quickAdd: null, search: false });

export function openQuickAdd(mode: QuickAddState['mode'] = 'transaction', initial: QuickAddState['initial'] = null) {
	ui.quickAdd = { mode, initial };
}
