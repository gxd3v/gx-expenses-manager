<script lang="ts">
	import type { Snippet } from 'svelte';
	import { searchable } from '#lib/format.ts';

	let { children }: { children: Snippet } = $props();

	let query = $state('');
	let container = $state<HTMLElement>();
	let hidden = $state(0);

	function apply() {
		if (!container) return;
		const needle = searchable(query.trim());
		let count = 0;
		for (const row of container.querySelectorAll<HTMLTableRowElement>('tbody tr')) {
			const match = !needle || searchable(row.textContent ?? '').includes(needle);
			row.hidden = !match;
			if (!match) count++;
		}
		hidden = count;
	}

	$effect(() => {
		void query;
		apply();
	});

	$effect(() => {
		if (!container) return;
		const observer = new MutationObserver(apply);
		observer.observe(container, { childList: true, subtree: true, characterData: true });
		return () => observer.disconnect();
	});
</script>

<div class="mb-2 flex items-center justify-end gap-2">
	{#if query && hidden}<span class="text-xs text-stone-500">{hidden} {hidden === 1 ? 'linha oculta' : 'linhas ocultas'}</span>{/if}
	<div class="w-56"><input bind:value={query} class="input py-1" placeholder="Filtrar…" aria-label="Filtrar tabela" /></div>
</div>
<div bind:this={container}>
	{@render children()}
</div>
