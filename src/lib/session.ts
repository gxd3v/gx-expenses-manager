import { invoke } from '@tauri-apps/api/core';

export type Status = {
	initialized: boolean;
	unlocked: boolean;
};

export const status = () => invoke<Status>('status');
export const unlock = (password: string) => invoke<void>('unlock', { password });
export const lock = () => invoke<void>('lock');
