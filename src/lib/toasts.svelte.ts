export type Toast = {
	id: number;
	message: string;
	kind: 'success' | 'error';
	action?: { label: string; run: () => void };
};

const DURATION = 6000;
let nextId = 1;

export const toasts = $state<Toast[]>([]);

export function notify(message: string, kind: Toast['kind'] = 'success', action?: Toast['action']) {
	const id = nextId++;
	toasts.push({ id, message, kind, action });
	setTimeout(() => dismiss(id), DURATION);
}

export function notifyError(error: unknown) {
	notify(error instanceof Error ? error.message : String(error), 'error');
}

export function dismiss(id: number) {
	const index = toasts.findIndex((t) => t.id === id);
	if (index >= 0) toasts.splice(index, 1);
}
