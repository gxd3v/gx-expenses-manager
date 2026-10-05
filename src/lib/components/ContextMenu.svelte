<script lang="ts" module>
	export type MenuItem = { label: string; run: () => void; danger?: boolean };
</script>

<script lang="ts">
	import { untrack } from 'svelte';

	const MARGIN = 8;

	let { x, y, items, onclose }: { x: number; y: number; items: MenuItem[]; onclose: () => void } = $props();

	let menu = $state<HTMLElement>();
	let position = $state(untrack(() => ({ left: x, top: y })));

	$effect(() => {
		if (!menu) return;
		const { width, height } = menu.getBoundingClientRect();
		position = {
			left: Math.max(MARGIN, Math.min(x, window.innerWidth - width - MARGIN)),
			top: Math.max(MARGIN, Math.min(y, window.innerHeight - height - MARGIN))
		};
		menu.querySelector<HTMLElement>('button')?.focus();
	});

	function outside(event: PointerEvent) {
		if (menu && !menu.contains(event.target as Node)) onclose();
	}

	function keydown(event: KeyboardEvent) {
		if (event.key !== 'Escape') return;
		event.stopPropagation();
		onclose();
	}

	function choose(item: MenuItem) {
		onclose();
		item.run();
	}
</script>

<svelte:window onpointerdown={outside} onresize={onclose} onblur={onclose} onscrollcapture={onclose} />

<div
	bind:this={menu}
	class="card fixed z-50 min-w-48 p-1 shadow-lg"
	style:left="{position.left}px"
	style:top="{position.top}px"
	role="menu"
	tabindex="-1"
	onkeydown={keydown}
	oncontextmenu={(e) => e.preventDefault()}
>
	{#each items as item (item.label)}
		<button
			type="button"
			role="menuitem"
			class="block w-full rounded-md px-3 py-1.5 text-left text-sm hover:bg-stone-100 focus:bg-stone-100 focus:outline-none dark:hover:bg-stone-800 dark:focus:bg-stone-800 {item.danger
				? 'text-red-600 dark:text-red-400'
				: ''}"
			onclick={() => choose(item)}>{item.label}</button
		>
	{/each}
</div>
