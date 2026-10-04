import { invoke } from '@tauri-apps/api/core';
import { isTauri } from './graphql.ts';

export type Status = {
	initialized: boolean;
	unlocked: boolean;
	dataDir: string;
};

export function status(): Promise<Status> {
	if (!isTauri) return Promise.resolve({ initialized: true, unlocked: true, dataDir: '' });
	return invoke<Status>('status');
}

export const unlock = (password: string) => invoke<void>('unlock', { password });
export const lock = () => (isTauri ? invoke<void>('lock') : Promise.resolve());
export const changePassword = (current: string, newPassword: string) =>
	invoke<void>('change_password', { current, newPassword });
export const restoreBackup = (path: string, password: string | null) =>
	invoke<void>('restore_backup', { path, password });
