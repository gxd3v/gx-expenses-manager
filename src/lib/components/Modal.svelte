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

	let pressedOutside = false;

	function pointerdown(event: PointerEvent) {
		const backdrop = event.currentTarget as HTMLElement;
		pressedOutside = event.target === backdrop && event.offsetX < backdrop.clientWidth;
	}

	function click(event: MouseEvent) {
		if (pressedOutside && event.target === event.currentTarget) onclose();
		pressedOutside = false;
	}

	function focusFirst(node: HTMLElement) {
		node.querySelector<HTMLElement>('input, select, textarea, button')?.focus();
	}
</script>

<svelte:window onkeydown={keydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
	class="fixed inset-0 z-40 flex items-start justify-center overflow-y-auto bg-black/40 p-4 pt-16"
	role="presentation"
	onpointerdown={pointerdown}
	onclick={click}
>
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
