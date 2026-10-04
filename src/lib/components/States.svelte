<script lang="ts">
	import type { Snippet } from 'svelte';
	import { errorMessage } from '#lib/graphql.ts';

	let {
		state,
		error = null,
		message = '',
		children
	}: { state: 'loading' | 'error' | 'empty'; error?: unknown; message?: string; children?: Snippet } = $props();
</script>

{#if state === 'loading'}
	<div class="flex items-center gap-2 py-8 muted" role="status">
		<span class="size-4 animate-spin rounded-full border-2 border-stone-300 border-t-indigo-500"></span>
		A carregar…
	</div>
{:else if state === 'error'}
	<div class="rounded-lg border border-red-200 bg-red-50 p-4 text-sm text-red-700 dark:border-red-900 dark:bg-red-950 dark:text-red-300" role="alert">
		{errorMessage(error)}
	</div>
{:else}
	<div class="flex flex-col items-center gap-3 rounded-xl border border-dashed border-stone-300 py-10 text-center dark:border-stone-700">
		<p class="muted">{message}</p>
		{@render children?.()}
	</div>
{/if}
