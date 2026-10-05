<script lang="ts">
	import { tick } from 'svelte';
	import type { EntryKind } from '#lib/api/transactions.ts';
	import { searchable } from '#lib/format.ts';
	import { refs } from '#lib/refs.svelte.ts';

	type Option = { id: string | null; label: string; path: string; child: boolean };

	let {
		value = $bindable(),
		kind = null,
		emptyLabel = 'Sem categoria',
		parentsOnly = false,
		exclude = null,
		compact = false,
		onchange
	}: {
		value: string | null;
		kind?: EntryKind | null;
		emptyLabel?: string;
		parentsOnly?: boolean;
		exclude?: string | null;
		compact?: boolean;
		onchange?: (value: string | null) => void;
	} = $props();

	let open = $state(false);
	let query = $state('');
	let active = $state(0);
	let root = $state<HTMLElement>();
	let search = $state<HTMLInputElement>();

	const options = $derived.by(() => {
		const usable = refs.categories.filter((c) => c.id !== exclude);
		const parents = usable.filter((c) => !c.parentId && (!kind || c.kind === 'BOTH' || c.kind === kind));
		const list: Option[] = [{ id: null, label: emptyLabel, path: emptyLabel, child: false }];
		for (const parent of parents) {
			list.push({ id: parent.id, label: parent.name, path: parent.name, child: false });
			if (parentsOnly) continue;
			for (const child of usable.filter((c) => c.parentId === parent.id)) {
				list.push({ id: child.id, label: child.name, path: `${parent.name} / ${child.name}`, child: true });
			}
		}
		return list;
	});

	const filtered = $derived.by(() => {
		const needle = searchable(query.trim());
		return needle ? options.filter((o) => o.id !== null && searchable(o.path).includes(needle)) : options;
	});

	const selected = $derived(options.find((o) => o.id === value)?.path ?? emptyLabel);

	async function toggle() {
		open = !open;
		if (!open) return;
		query = '';
		await tick();
		search?.focus();
	}

	function choose(option: Option) {
		value = option.id;
		open = false;
		onchange?.(option.id);
	}

	function keydown(event: KeyboardEvent) {
		if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
			event.preventDefault();
			const step = event.key === 'ArrowDown' ? 1 : -1;
			active = (active + step + filtered.length) % Math.max(filtered.length, 1);
		} else if (event.key === 'Enter') {
			event.preventDefault();
			if (filtered[active]) choose(filtered[active]);
		} else if (event.key === 'Escape') {
			event.stopPropagation();
			open = false;
		} else if (event.key === 'Tab') {
			open = false;
		}
	}

	function outside(event: PointerEvent) {
		if (open && root && !root.contains(event.target as Node)) open = false;
	}

	$effect(() => {
		void query;
		active = 0;
	});
</script>

<svelte:window onpointerdown={outside} />

<div class="relative" bind:this={root}>
	<button
		type="button"
		class={compact ? 'header-filter max-w-48 truncate text-left' : 'input flex items-center justify-between gap-2 text-left'}
		onclick={toggle}
		aria-haspopup="listbox"
		aria-expanded={open}
	>
		<span class="truncate">{selected}</span>
		{#if !compact}<span class="text-xs text-stone-400" aria-hidden="true">▾</span>{/if}
	</button>
	{#if open}
		<div class="card absolute left-0 z-50 mt-1 w-72 p-2 normal-case shadow-lg">
			<input
				bind:this={search}
				bind:value={query}
				onkeydown={keydown}
				class="input mb-2"
				placeholder="Pesquisar categoria…"
				aria-label="Pesquisar categoria"
				autocomplete="off"
			/>
			<ul class="max-h-64 overflow-y-auto" role="listbox">
				{#each filtered as option, index (option.id ?? 'none')}
					<li role="option" aria-selected={option.id === value}>
						<button
							type="button"
							class="w-full rounded-md px-2 py-1 text-left text-sm font-normal tracking-normal text-stone-900 dark:text-stone-100 {index === active
								? 'bg-stone-100 dark:bg-stone-800'
								: ''} {option.child && !query ? 'pl-6' : ''} {option.id === value ? 'font-semibold' : ''}"
							onclick={() => choose(option)}
							onmouseenter={() => (active = index)}>{query ? option.path : option.label}</button
						>
					</li>
				{:else}
					<li class="px-2 py-1 text-sm text-stone-500">Sem resultados.</li>
				{/each}
			</ul>
		</div>
	{/if}
</div>
