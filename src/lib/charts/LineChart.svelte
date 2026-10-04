<script lang="ts">
	import { formatCompact } from '#lib/format.ts';
	import DataTable from './DataTable.svelte';
	import Legend from './Legend.svelte';
	import { labelStride, niceTicks, type Series } from './scale.ts';

	let {
		labels,
		series,
		format,
		height = 240
	}: { labels: string[]; series: Series[]; format: (value: number) => string; height?: number } = $props();

	const pad = { top: 12, right: 16, bottom: 28, left: 56 };
	let width = $state(0);
	let hover = $state<number | null>(null);

	const values = $derived(series.flatMap((s) => s.values));
	const ticks = $derived(niceTicks(Math.min(...values, 0), Math.max(...values, 0)));
	const innerWidth = $derived(Math.max(1, width - pad.left - pad.right));
	const innerHeight = $derived(height - pad.top - pad.bottom);
	const step = $derived(labels.length > 1 ? innerWidth / (labels.length - 1) : 0);
	const stride = $derived(labelStride(labels.length, innerWidth));

	const x = (index: number) => pad.left + index * step;
	const y = (value: number) => {
		const [min, max] = [ticks[0], ticks[ticks.length - 1]];
		return pad.top + innerHeight - ((value - min) / (max - min)) * innerHeight;
	};
	const showLabel = (index: number) =>
		index === labels.length - 1 || (index % stride === 0 && labels.length - 1 - index >= stride / 2);
	const anchor = (index: number) => (index === 0 ? 'start' : index === labels.length - 1 ? 'end' : 'middle');
	const line = (points: number[]) => points.map((v, i) => `${i ? 'L' : 'M'}${x(i)},${y(v)}`).join('');
	const area = (points: number[]) => `${line(points)}L${x(points.length - 1)},${y(ticks[0])}L${x(0)},${y(ticks[0])}Z`;

	function move(event: PointerEvent) {
		const rect = (event.currentTarget as SVGElement).getBoundingClientRect();
		const index = step ? Math.round((event.clientX - rect.left - pad.left) / step) : 0;
		hover = Math.min(Math.max(index, 0), labels.length - 1);
	}
</script>

<Legend {series} />
<div class="relative" bind:clientWidth={width}>
	<svg {width} {height} role="img" aria-label={series.map((s) => s.name).join(', ')} onpointermove={move} onpointerleave={() => (hover = null)}>
		{#each ticks as tick (tick)}
			<line x1={pad.left} x2={width - pad.right} y1={y(tick)} y2={y(tick)} stroke="var(--chart-grid)" stroke-width="1" />
			<text x={pad.left - 8} y={y(tick)} dy="0.32em" text-anchor="end" font-size="11" fill="var(--chart-text)">
				{formatCompact(tick)}
			</text>
		{/each}
		{#each labels as label, index (index)}
			{#if showLabel(index)}
				<text x={x(index)} y={height - 8} text-anchor={anchor(index)} font-size="11" fill="var(--chart-text)">{label}</text>
			{/if}
		{/each}
		{#if series.length === 1}
			<path d={area(series[0].values)} fill={series[0].color} opacity="0.1" />
		{/if}
		{#each series as item (item.name)}
			<path d={line(item.values)} fill="none" stroke={item.color} stroke-width="2" stroke-linejoin="round" stroke-linecap="round" />
		{/each}
		{#if hover !== null}
			<line x1={x(hover)} x2={x(hover)} y1={pad.top} y2={pad.top + innerHeight} stroke="var(--chart-text)" stroke-width="1" opacity="0.4" />
			{#each series as item (item.name)}
				<circle cx={x(hover)} cy={y(item.values[hover])} r="4" fill={item.color} stroke="var(--surface)" stroke-width="2" />
			{/each}
		{/if}
	</svg>
	{#if hover !== null}
		<div
			class="pointer-events-none absolute top-0 z-10 rounded-md bg-stone-900 px-3 py-2 text-xs text-white shadow-lg dark:bg-stone-100 dark:text-stone-900"
			style:left="{Math.min(x(hover) + 12, width - 180)}px"
		>
			<p class="font-medium">{labels[hover]}</p>
			{#each series as item (item.name)}
				<p class="flex items-center gap-2 tabular-nums">
					<span class="size-2 rounded-full" style:background-color={item.color}></span>
					{item.name}: {format(item.values[hover])}
				</p>
			{/each}
		</div>
	{/if}
</div>
<DataTable {labels} {series} {format} />
