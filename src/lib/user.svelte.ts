import type { Profile, Status } from './session.ts';

const LAST_USER_KEY = 'expenses-manager-last-user';

export const current = $state<{ user: Profile | null; users: Profile[]; returnTo: Profile | null }>({
	user: null,
	users: [],
	returnTo: null
});

export function setSession(status: Status) {
	current.users = status.users;
	current.returnTo = status.returnTo;
	setCurrentUser(status.user);
}

export function jointsOf(user: Profile | null): Profile[] {
	return user ? current.users.filter((u) => u.joint && u.members.includes(user.id)) : [];
}

export function membersOf(user: Profile | null): Profile[] {
	return user ? current.users.filter((u) => user.members.includes(u.id)) : [];
}

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
