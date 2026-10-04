<script lang="ts">
	const ICONS = [
		'🏦', '💳', '💶', '💰', '🪙', '🐷', '📈', '📉', '💸', '🧾', '🏛️', '💼',
		'🏠', '🔑', '💡', '💧', '🔥', '📶', '📱', '💻', '📺', '🧹', '🛠️', '🌱',
		'🛒', '🍽️', '☕', '🍔', '🍕', '🥖', '🍷', '🚗', '⛽', '🅿️', '🚌', '🚆',
		'✈️', '🧳', '🏖️', '🏥', '💊', '🦷', '👓', '🐾', '🎮', '🎬', '🎵', '📚',
		'🎓', '👕', '👟', '💄', '🎁', '💍', '👶', '🧒', '🏋️', '⚽', '❤️', '🤝',
		'📦', '📄', '🔒', '⭐'
	];

	let { value = $bindable(), label = 'Ícone' }: { value: string | null; label?: string } = $props();

	let open = $state(false);
	let root = $state<HTMLElement>();

	function outside(event: PointerEvent) {
		if (open && root && !root.contains(event.target as Node)) open = false;
	}

	function keydown(event: KeyboardEvent) {
		if (event.key !== 'Escape' || !open) return;
		event.stopPropagation();
		open = false;
	}

	function choose(icon: string | null) {
		value = icon;
		open = false;
	}
</script>

<svelte:window onpointerdown={outside} />

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div class="relative" bind:this={root} onkeydown={keydown} role="group" aria-label={label}>
	<button
		type="button"
		class="input flex h-9 items-center justify-center text-lg"
		onclick={() => (open = !open)}
		aria-haspopup="true"
		aria-expanded={open}
		aria-label="Escolher ícone"
	>
		{#if value}{value}{:else}<span class="text-sm text-stone-400">—</span>{/if}
	</button>
	{#if open}
		<div class="card absolute right-0 z-50 mt-1 w-80 p-2 shadow-lg">
			<div class="grid grid-cols-8 gap-1">
				{#each ICONS as icon (icon)}
					<button
						type="button"
						class="rounded-md p-1 text-xl hover:bg-stone-100 dark:hover:bg-stone-800 {value === icon ? 'bg-indigo-100 dark:bg-indigo-900' : ''}"
						onclick={() => choose(icon)}
						aria-label={icon}
						aria-pressed={value === icon}>{icon}</button
					>
				{/each}
			</div>
			<button type="button" class="btn-ghost mt-1 w-full" onclick={() => choose(null)}>Sem ícone</button>
		</div>
	{/if}
</div>
