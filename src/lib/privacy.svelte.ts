import { readUserItem, writeUserItem } from './user.svelte.ts';

const STORAGE_KEY = 'expenses-manager-privacy';

export const privacy = $state({ hidden: false });

export function togglePrivacy() {
	privacy.hidden = !privacy.hidden;
	applyPrivacy();
	writeUserItem(STORAGE_KEY, privacy.hidden ? '1' : '0');
}

export function loadPrivacy() {
	privacy.hidden = readUserItem(STORAGE_KEY) === '1';
	applyPrivacy();
}

export function applyPrivacy() {
	document.documentElement.classList.toggle('privacy', privacy.hidden);
}

export function scramble(text: string): string {
	let seed = [...text].reduce((hash, char) => (hash * 31 + char.charCodeAt(0)) >>> 0, 7);
	return text.replace(/\d/g, () => {
		seed = (seed * 1103515245 + 12345) >>> 0;
		return String((seed >>> 16) % 10);
	});
}
