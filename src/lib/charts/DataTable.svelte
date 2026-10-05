<script lang="ts">
	import Filterable from '#lib/components/Filterable.svelte';
	import type { Series } from './scale.ts';

	let { labels, series, format }: { labels: string[]; series: Series[]; format: (value: number) => string } = $props();
</script>

<details class="mt-2 text-sm">
	<summary class="cursor-pointer muted">Ver tabela</summary>
	<Filterable>
	<div class="mt-2 max-h-64 overflow-auto">
		<table class="table-base">
			<thead>
				<tr>
					<th></th>
					{#each series as item (item.name)}<th class="text-right">{item.name}</th>{/each}
				</tr>
			</thead>
			<tbody>
				{#each labels as label, index (index)}
					<tr>
						<td>{label}</td>
						{#each series as item (item.name)}
							<td class="money text-right tabular-nums">{format(item.values[index] ?? 0)}</td>
						{/each}
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
	</Filterable>
</details>
