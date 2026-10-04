import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';
import { autoBackup } from './api/backups.ts';
import { materializeDue } from './api/recurrences.ts';
import { listAlerts, type Alert } from './api/reports.ts';
import { formatDate, formatMonth } from './format.ts';
import { isTauri } from './graphql.ts';

const NOTIFIED_KEY = 'gx-notified-alerts';
const MAX_REMEMBERED = 500;

export async function runBackgroundTasks(): Promise<Alert[]> {
	await materializeDue().catch(() => 0);
	await autoBackup().catch(() => undefined);
	const alerts = await listAlerts().catch(() => []);
	await notifyNew(alerts);
	return alerts;
}

async function notifyNew(alerts: Alert[]) {
	const notified = readNotified();
	const fresh = alerts.filter((alert) => !notified.includes(alert.key));
	if (!isTauri || fresh.length === 0 || !(await permitted())) return;

	for (const alert of fresh) {
		sendNotification({ title: alert.title, body: describe(alert) });
	}
	writeNotified([...fresh.map((a) => a.key), ...notified].slice(0, MAX_REMEMBERED));
}

export function describe(alert: Alert): string {
	if (!alert.date) return alert.message;
	const when = alert.kind === 'negative_forecast' ? formatMonth(alert.date, 'long') : formatDate(alert.date);
	return `${alert.message} · ${when}`;
}

async function permitted(): Promise<boolean> {
	if (await isPermissionGranted()) return true;
	return (await requestPermission()) === 'granted';
}

function readNotified(): string[] {
	try {
		return JSON.parse(localStorage.getItem(NOTIFIED_KEY) ?? '[]');
	} catch {
		return [];
	}
}

function writeNotified(keys: string[]) {
	try {
		localStorage.setItem(NOTIFIED_KEY, JSON.stringify(keys));
	} catch {
		return;
	}
}
