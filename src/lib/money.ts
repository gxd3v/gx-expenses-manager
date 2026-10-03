export function formatMoney(cents: number, currency: string): string {
	return new Intl.NumberFormat('pt-PT', { style: 'currency', currency }).format(cents / 100);
}

export function toCents(value: string): number | null {
	const normalized = value.replace(/\s/g, '').replace(',', '.');
	if (!/^-?\d+(\.\d{1,2})?$/.test(normalized)) return null;
	return Math.round(Number(normalized) * 100);
}

export function fromCents(cents: number): string {
	return (cents / 100).toFixed(2).replace('.', ',');
}
