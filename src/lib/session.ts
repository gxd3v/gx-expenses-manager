import { invoke } from '@tauri-apps/api/core';
import { isTauri } from './graphql.ts';

export type Profile = {
	id: string;
	name: string;
	color: string;
	createdAt: string;
};

export type Status = {
	users: Profile[];
	legacy: boolean;
	unlocked: boolean;
	user: Profile | null;
	simulation: boolean;
	dataDir: string;
};

export const USER_COLORS = ['#BFDDF7', '#F8C8DC', '#C7EFCF', '#FFD8B1', '#D9CCF5', '#FDF1B8', '#BDE7E3', '#EBCBEF'];

export function status(): Promise<Status> {
	if (!isTauri) return Promise.resolve({ users: [], legacy: false, unlocked: true, user: null, simulation: false, dataDir: '' });
	return invoke<Status>('status');
}

export const unlock = (userId: string, password: string) => invoke<void>('unlock', { userId, password });
export const createUser = (name: string, color: string, password: string) => invoke<void>('create_user', { name, color, password });
export const migrateUser = (name: string, color: string, password: string) => invoke<void>('migrate_user', { name, color, password });
export const updateUser = (name: string, color: string) => invoke<Profile>('update_user', { name, color });
export const deleteUser = (password: string) => invoke<void>('delete_user', { password });
export const lock = () => (isTauri ? invoke<void>('lock') : Promise.resolve());
export const changePassword = (current: string, newPassword: string) =>
	invoke<void>('change_password', { current, newPassword });
export const restoreBackup = (path: string, password: string | null) =>
	invoke<void>('restore_backup', { path, password });
export const resetData = (password: string) => invoke<void>('reset_data', { password });
export const startSimulation = () => invoke<void>('start_simulation');
export const stopSimulation = () => invoke<void>('stop_simulation');
