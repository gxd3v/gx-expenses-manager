import type { Profile } from './session.ts';

const LAST_USER_KEY = 'expenses-manager-last-user';

export const current = $state<{ user: Profile | null }>({ user: null });

export function setCurrentUser(user: Profile | null) {
	current.user = user;
	if (!user) return;
	try {
		localStorage.setItem(LAST_USER_KEY, user.id);
	} catch {
		return;
	}
}

export function lastUserId(): string | null {
	try {
		return localStorage.getItem(LAST_USER_KEY);
	} catch {
		return null;
	}
}

export function userKey(key: string): string {
	return current.user ? `${key}:${current.user.id}` : key;
}

export function readUserItem(key: string): string | null {
	try {
		return localStorage.getItem(userKey(key)) ?? localStorage.getItem(key);
	} catch {
		return null;
	}
}

export function writeUserItem(key: string, value: string) {
	try {
		localStorage.setItem(userKey(key), value);
	} catch {
		return;
	}
}
