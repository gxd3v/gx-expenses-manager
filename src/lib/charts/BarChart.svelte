<script lang="ts">
	import { formatCompact } from '#lib/format.ts';
	import DataTable from './DataTable.svelte';
	import Legend from './Legend.svelte';
	import { barPath, labelStride, niceTicks, type Series } from './scale.ts';

	let {
		labels,
		series,
		format,
		height = 240
	}: { labels: string[]; series: Series[]; format: (value: number) => string; height?: number } = $props();

	const pad = { top: 12, right: 16, bottom: 28, left: 56 };
	const gap = 2;
	let width = $state(600);
	let hover = $state<number | null>(null);

	const values = $derived(series.flatMap((s) => s.values));
	const ticks = $derived(niceTicks(Math.min(...values, 0), Math.max(...values, 0)));
	const innerWidth = $derived(Math.max(1, width - pad.left - pad.right));
	const innerHeight = $derived(height - pad.top - pad.bottom);
	const band = $derived(innerWidth / Math.max(labels.length, 1));
	const barWidth = $derived(Math.max(2, Math.min(24, (band * 0.7 - gap * (series.length - 1)) / series.length)));
	const groupWidth = $derived(barWidth * series.length + gap * (series.length - 1));
	const stride = $derived(labelStride(labels.length, innerWidth));

	const y = (value: number) => {
		const [min, max] = [ticks[0], ticks[ticks.length - 1]];
		return pad.top + innerHeight - ((value - min) / (max - min)) * innerHeight;
	};
	const groupX = (index: number) => pad.left + index * band + (band - groupWidth) / 2;
	const color = (item: Series, value: number) => (value < 0 && item.negativeColor ? item.negativeColor : item.color);
</script>

<Legend {series} />
<div class="relative" bind:clientWidth={width}>
	<svg {width} {height} role="img" aria-label={series.map((s) => s.name).join(', ')}>
		{#each ticks as tick (tick)}
			<line x1={pad.left} x2={width - pad.right} y1={y(tick)} y2={y(tick)} stroke="var(--chart-grid)" stroke-width="1" />
			<text x={pad.left - 8} y={y(tick)} dy="0.32em" text-anchor="end" font-size="11" fill="var(--chart-text)">
				{formatCompact(tick)}
			</text>
		{/each}
		{#each labels as label, index (index)}
			{#if index % stride === 0}
				<text x={pad.left + index * band + band / 2} y={height - 8} text-anchor="middle" font-size="11" fill="var(--chart-text)">
					{label}
				</text>
			{/if}
			{#each series as item, s (item.name)}
				{@const value = item.values[index] ?? 0}
				{#if value !== 0}
					<path d={barPath(groupX(index) + s * (barWidth + gap), barWidth, y(0), y(value))} fill={color(item, value)} />
				{/if}
			{/each}
			<rect
				x={pad.left + index * band}
				y={pad.top}
				width={band}
				height={innerHeight}
				fill="transparent"
				role="presentation"
				onpointerenter={() => (hover = index)}
				onpointerleave={() => (hover = null)}
			/>
		{/each}
	</svg>
	{#if hover !== null}
		<div
			class="pointer-events-none absolute top-0 z-10 rounded-md bg-stone-900 px-3 py-2 text-xs text-white shadow-lg dark:bg-stone-100 dark:text-stone-900"
			style:left="{Math.min(pad.left + hover * band + band, width - 180)}px"
		>
			<p class="font-medium">{labels[hover]}</p>
			{#each series as item (item.name)}
				<p class="flex items-center gap-2 tabular-nums">
					<span class="size-2 rounded-full" style:background-color={color(item, item.values[hover] ?? 0)}></span>
					{item.name}: {format(item.values[hover] ?? 0)}
				</p>
			{/each}
		</div>
	{/if}
</div>
<DataTable {labels} {series} {format} />
