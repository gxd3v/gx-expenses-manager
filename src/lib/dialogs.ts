import { ask, open, save } from '@tauri-apps/plugin-dialog';
import { isTauri } from './graphql.ts';

export async function confirmAction(message: string): Promise<boolean> {
	if (!isTauri) return window.confirm(message);
	return ask(message, { title: 'Confirmar', kind: 'warning', okLabel: 'Confirmar', cancelLabel: 'Cancelar' });
}

export async function pickSavePath(name: string, extension: string): Promise<string | null> {
	if (!isTauri) return window.prompt('Caminho do ficheiro', name);
	return save({ defaultPath: name, filters: [{ name: extension.toUpperCase(), extensions: [extension] }] });
}

export async function pickOpenPath(extensions: string[]): Promise<string | null> {
	if (!isTauri) return window.prompt('Caminho do ficheiro');
	const result = await open({ multiple: false, filters: [{ name: 'Backup', extensions }] });
	return typeof result === 'string' ? result : null;
}

export async function pickDirectory(): Promise<string | null> {
	if (!isTauri) return window.prompt('Pasta');
	const result = await open({ directory: true, multiple: false });
	return typeof result === 'string' ? result : null;
}
