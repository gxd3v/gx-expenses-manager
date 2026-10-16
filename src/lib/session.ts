import { invoke } from '@tauri-apps/api/core';
import { isTauri } from './graphql.ts';

export type Profile = {
	id: string;
	name: string;
	color: string;
	createdAt: string;
	joint: boolean;
	members: string[];
};

export type MemberRecurrence = {
	description: string;
	kind: 'INCOME' | 'OUTCOME';
	amount: number;
	startDate: string;
	endDate: string | null;
	unit: 'DAY' | 'WEEK' | 'MONTH' | 'YEAR';
	interval: number;
	categoryName: string | null;
	accountName: string;
	variableAmount: boolean;
	linked: boolean;
};

export type Status = {
	users: Profile[];
	legacy: boolean;
	unlocked: boolean;
	user: Profile | null;
	returnTo: Profile | null;
	simulation: boolean;
	dataDir: string;
};

export const USER_COLORS = ['#BFDDF7', '#F8C8DC', '#C7EFCF', '#FFD8B1', '#D9CCF5', '#FDF1B8', '#BDE7E3', '#EBCBEF'];

export function status(): Promise<Status> {
	if (!isTauri) return Promise.resolve({ users: [], legacy: false, unlocked: true, user: null, returnTo: null, simulation: false, dataDir: '' });
	return invoke<Status>('status');
}

export const unlock = (userId: string, password: string) => invoke<void>('unlock', { userId, password });
export const createUser = (name: string, color: string, password: string, joint: boolean) =>
	invoke<void>('create_user', { name, color, password, joint });
export const migrateUser = (name: string, color: string, password: string) => invoke<void>('migrate_user', { name, color, password });
export const updateUser = (name: string, color: string) => invoke<Profile>('update_user', { name, color });
export const deleteUser = (password: string) => invoke<void>('delete_user', { password });
export const addMember = (userId: string, password: string) => invoke<Profile>('add_member', { userId, password });
export const removeMember = (userId: string) => invoke<Profile>('remove_member', { userId });
export const switchToJoint = (jointId: string, password: string | null) => invoke<void>('switch_to_joint', { jointId, password });
export const switchBack = () => invoke<void>('switch_back');
export const memberRecurrences = (userId: string, password: string | null) =>
	invoke<MemberRecurrence[]>('member_recurrences', { userId, password });
export const lock = () => (isTauri ? invoke<void>('lock') : Promise.resolve());
export const changePassword = (current: string, newPassword: string) =>
	invoke<void>('change_password', { current, newPassword });
export const restoreBackup = (path: string, password: string | null) =>
	invoke<void>('restore_backup', { path, password });
export const resetData = (password: string) => invoke<void>('reset_data', { password });
export const startSimulation = () => invoke<void>('start_simulation');
export const stopSimulation = () => invoke<void>('stop_simulation');
