<script lang="ts">
	import { privacy, togglePrivacy } from '#lib/privacy.svelte.ts';

	let { compact = false }: { compact?: boolean } = $props();

	const label = $derived(privacy.hidden ? 'Mostrar valores (Ctrl+H)' : 'Esconder valores (Ctrl+H)');
</script>

<button
	class={compact ? 'titlebar-button' : 'btn-ghost'}
	onclick={togglePrivacy}
	aria-pressed={privacy.hidden}
	aria-label={label}
	title={label}
>
	<svg viewBox="0 0 24 24" class="size-4" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" aria-hidden="true">
		<path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12z" />
		<circle cx="12" cy="12" r="3" />
		{#if privacy.hidden}
			<path d="M3 3l18 18" />
		{/if}
	</svg>
</button>
