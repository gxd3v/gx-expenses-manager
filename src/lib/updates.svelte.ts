import { relaunch } from '@tauri-apps/plugin-process';
import { check, type Update } from '@tauri-apps/plugin-updater';
import { saveWindowState, StateFlags } from '@tauri-apps/plugin-window-state';
import { isTauri } from './graphql.ts';
import { lock } from './session.ts';

export const updates = $state<{
	available: Update | null;
	dismissed: string | null;
	installing: boolean;
	progress: number | null;
	notesOpen: boolean;
}>({ available: null, dismissed: null, installing: false, progress: null, notesOpen: false });

export async function checkForUpdates() {
	if (!isTauri || updates.installing) return;
	try {
		const update = await check();
		if (update?.version !== updates.available?.version) updates.available = update;
	} catch {
		return;
	}
}

export async function checkNow(): Promise<Update | null> {
	if (!isTauri) return null;
	const update = await check();
	updates.available = update;
	updates.dismissed = null;
	updates.notesOpen = update !== null;
	return update;
}

export async function installUpdate() {
	const update = updates.available;
	if (!update) return;
	updates.installing = true;
	let total = 0;
	let received = 0;
	try {
		await update.download((event) => {
			if (event.event === 'Started') total = event.data.contentLength ?? 0;
			if (event.event === 'Progress') received += event.data.chunkLength;
			updates.progress = total ? Math.round((received / total) * 100) : null;
		});
		await saveWindowState(StateFlags.ALL);
		await lock();
		try {
			await update.install();
			await relaunch();
		} catch (error) {
			location.reload();
			throw error;
		}
	} finally {
		updates.installing = false;
	}
}
