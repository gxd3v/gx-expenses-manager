import { privacy, scramble } from './privacy.svelte.ts';
import { app } from './settings.svelte.ts';

const locale = 'pt-PT';

export function formatMoney(cents: number, currency?: string): string {
	const text = new Intl.NumberFormat(locale, {
		style: 'currency',
		currency: currency ?? app.settings?.currency ?? 'EUR'
	}).format((cents || 0) / 100);
	return privacy.hidden ? scramble(text) : text;
}

export function formatCompact(cents: number): string {
	const text = new Intl.NumberFormat(locale, { notation: 'compact', maximumFractionDigits: 1 }).format((cents || 0) / 100);
	return privacy.hidden ? scramble(text) : text;
}

export function formatDate(iso: string | null | undefined): string {
	if (!iso) return '—';
	const [year, month, day] = iso.slice(0, 10).split('-');
	switch (app.settings?.dateFormat) {
		case 'YEAR_MONTH_DAY':
			return `${year}-${month}-${day}`;
		case 'MONTH_DAY_YEAR':
			return `${month}/${day}/${year}`;
		default:
			return `${day}/${month}/${year}`;
	}
}

export function formatMonth(iso: string, style: 'short' | 'long' = 'short'): string {
	const date = new Date(`${iso.slice(0, 7)}-01T00:00:00`);
	return new Intl.DateTimeFormat(locale, { month: style, year: 'numeric' }).format(date);
}

export function formatPercent(value: number): string {
	return new Intl.NumberFormat(locale, { style: 'percent', maximumFractionDigits: 0 }).format(value);
}

export function toCents(value: string): number | null {
	const normalized = value.replace(/\s/g, '').replace(/\.(?=\d{3}(\D|$))/g, '').replace(',', '.');
	if (!/^-?\d+(\.\d{1,2})?$/.test(normalized)) return null;
	return Math.round(Number(normalized) * 100);
}

export function fromCents(cents: number | null | undefined): string {
	if (cents === null || cents === undefined) return '';
	return (cents / 100).toFixed(2).replace('.', ',');
}

export function isoDate(date: Date): string {
	const pad = (n: number) => String(n).padStart(2, '0');
	return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

export function today(): string {
	return isoDate(new Date());
}

export function addMonths(iso: string, months: number): string {
	const date = new Date(`${iso.slice(0, 10)}T00:00:00`);
	const day = date.getDate();
	date.setDate(1);
	date.setMonth(date.getMonth() + months);
	const last = new Date(date.getFullYear(), date.getMonth() + 1, 0).getDate();
	date.setDate(Math.min(day, last));
	return isoDate(date);
}

export function addDays(iso: string, days: number): string {
	const date = new Date(`${iso.slice(0, 10)}T00:00:00`);
	date.setDate(date.getDate() + days);
	return isoDate(date);
}

export function weekStart(iso: string, firstDayOfWeek: number): string {
	const day = new Date(`${iso.slice(0, 10)}T00:00:00`).getDay();
	return addDays(iso, -((day - firstDayOfWeek + 7) % 7));
}

export function monthStart(iso: string): string {
	return `${iso.slice(0, 7)}-01`;
}

export function monthEnd(iso: string): string {
	return addDays(addMonths(monthStart(iso), 1), -1);
}
