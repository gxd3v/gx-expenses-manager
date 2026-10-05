import { invoke } from '@tauri-apps/api/core';
import { isTauri } from './graphql.ts';

export type Status = {
	initialized: boolean;
	unlocked: boolean;
	simulation: boolean;
	dataDir: string;
};

export function status(): Promise<Status> {
	if (!isTauri) return Promise.resolve({ initialized: true, unlocked: true, simulation: false, dataDir: '' });
	return invoke<Status>('status');
}

export const unlock = (password: string) => invoke<void>('unlock', { password });
export const lock = () => (isTauri ? invoke<void>('lock') : Promise.resolve());
export const changePassword = (current: string, newPassword: string) =>
	invoke<void>('change_password', { current, newPassword });
export const restoreBackup = (path: string, password: string | null) =>
	invoke<void>('restore_backup', { path, password });
export const resetData = (password: string) => invoke<void>('reset_data', { password });
export const startSimulation = () => invoke<void>('start_simulation');
export const stopSimulation = () => invoke<void>('stop_simulation');
