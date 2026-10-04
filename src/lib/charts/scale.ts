export type Series = {
	name: string;
	color: string;
	values: number[];
	negativeColor?: string;
};

export function niceTicks(min: number, max: number, count = 4): number[] {
	const low = Math.min(min, max);
	const high = max === min ? low + 100 : Math.max(min, max);
	const step = niceStep((high - low) / count);
	const start = Math.floor(low / step) * step;
	const end = Math.ceil(high / step) * step;

	const ticks: number[] = [];
	for (let value = start; value <= end + step / 2; value += step) ticks.push(value);
	return ticks;
}

function niceStep(raw: number): number {
	const power = 10 ** Math.floor(Math.log10(raw));
	const fraction = raw / power;
	const nice = fraction <= 1 ? 1 : fraction <= 2 ? 2 : fraction <= 5 ? 5 : 10;
	return nice * power;
}

export function labelStride(count: number, width: number, minSpacing = 64): number {
	return Math.max(1, Math.ceil(count / Math.max(1, Math.floor(width / minSpacing))));
}

export function barPath(x: number, width: number, base: number, end: number, radius = 4): string {
	const height = Math.abs(end - base);
	const r = Math.min(radius, height, width / 2);
	const dir = end < base ? 1 : -1;
	return [
		`M${x},${base}`,
		`V${end + dir * r}`,
		`Q${x},${end} ${x + r},${end}`,
		`H${x + width - r}`,
		`Q${x + width},${end} ${x + width},${end + dir * r}`,
		`V${base}`,
		'Z'
	].join('');
}
