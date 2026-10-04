<script lang="ts">
	import type { Series } from './scale.ts';

	let { series }: { series: Series[] } = $props();
</script>

{#if series.length > 1}
	<ul class="mb-2 flex flex-wrap gap-4 text-xs text-stone-600 dark:text-stone-300">
		{#each series as item (item.name)}
			<li class="flex items-center gap-1.5">
				<span class="h-0.5 w-3 rounded" style:background-color={item.color}></span>
				{item.name}
			</li>
		{/each}
	</ul>
{/if}
