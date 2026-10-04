<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		title,
		onclose,
		children,
		wide = false
	}: { title: string; onclose: () => void; children: Snippet; wide?: boolean } = $props();

	function keydown(event: KeyboardEvent) {
		if (event.key === 'Escape') onclose();
	}

	function focusFirst(node: HTMLElement) {
		node.querySelector<HTMLElement>('input, select, textarea, button')?.focus();
	}
</script>

<svelte:window onkeydown={keydown} />

<div class="fixed inset-0 z-40 flex items-start justify-center overflow-y-auto bg-black/40 p-4 pt-16" role="presentation">
	<div
		class="card w-full shadow-xl {wide ? 'max-w-3xl' : 'max-w-lg'}"
		role="dialog"
		aria-modal="true"
		aria-label={title}
		use:focusFirst
	>
		<header class="mb-4 flex items-center justify-between">
			<h2 class="text-lg font-semibold">{title}</h2>
			<button class="btn-ghost" onclick={onclose} aria-label="Fechar">✕</button>
		</header>
		{@render children()}
	</div>
</div>
