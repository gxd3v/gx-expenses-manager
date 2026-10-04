<script lang="ts">
	import { refs } from '#lib/refs.svelte.ts';
	import type { EntryKind } from '#lib/api/transactions.ts';

	let {
		value = $bindable(),
		kind = null,
		emptyLabel = 'Sem categoria'
	}: { value: string | null; kind?: EntryKind | null; emptyLabel?: string } = $props();

	const parents = $derived(refs.categories.filter((c) => !c.parentId && (!kind || c.kind === 'BOTH' || c.kind === kind)));
	const childrenOf = (id: string) => refs.categories.filter((c) => c.parentId === id);
</script>

<select bind:value class="input">
	<option value={null}>{emptyLabel}</option>
	{#each parents as parent (parent.id)}
		<optgroup label={parent.name}>
			<option value={parent.id}>{parent.name}</option>
			{#each childrenOf(parent.id) as child (child.id)}
				<option value={child.id}>&nbsp;&nbsp;{child.name}</option>
			{/each}
		</optgroup>
	{/each}
</select>
