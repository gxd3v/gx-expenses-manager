<script lang="ts">
	let {
		items,
		format,
		limit = 8
	}: { items: { label: string; value: number }[]; format: (value: number) => string; limit?: number } = $props();

	const shown = $derived(items.slice(0, limit));
	const max = $derived(Math.max(...shown.map((i) => i.value), 1));
</script>

<ul class="flex flex-col gap-2">
	{#each shown as item (item.label)}
		<li class="grid grid-cols-[minmax(0,9rem)_1fr_auto] items-center gap-3 text-sm" title="{item.label}: {format(item.value)}">
			<span class="truncate text-stone-600 dark:text-stone-300">{item.label}</span>
			<span class="h-3 rounded-r" style:width="{Math.max((item.value / max) * 100, 1)}%" style:background-color="var(--series-1)"></span>
			<span class="money tabular-nums">{format(item.value)}</span>
		</li>
	{/each}
</ul>
