<script lang="ts">
	import { dismiss, toasts } from '#lib/toasts.svelte.ts';
</script>

<div class="fixed right-4 bottom-4 z-50 flex w-80 flex-col gap-2" aria-live="polite">
	{#each toasts as toast (toast.id)}
		<div
			class="flex items-start gap-3 rounded-lg px-4 py-3 text-sm shadow-lg {toast.kind === 'error'
				? 'bg-red-600 text-white'
				: 'bg-stone-900 text-white dark:bg-stone-100 dark:text-stone-900'}"
			role={toast.kind === 'error' ? 'alert' : 'status'}
		>
			<p class="flex-1 whitespace-pre-line">{toast.message}</p>
			{#if toast.action}
				<button
					class="font-semibold underline"
					onclick={() => {
						toast.action?.run();
						dismiss(toast.id);
					}}
				>
					{toast.action.label}
				</button>
			{/if}
			<button onclick={() => dismiss(toast.id)} aria-label="Fechar">✕</button>
		</div>
	{/each}
</div>
