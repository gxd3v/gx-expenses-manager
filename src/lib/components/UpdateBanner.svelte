<script lang="ts">
	import { notifyError } from '#lib/toasts.svelte.ts';
	import { installUpdate, updates } from '#lib/updates.svelte.ts';
	import Modal from './Modal.svelte';

	const update = $derived(updates.available);
	const visible = $derived(update !== null && updates.dismissed !== update.version);
	const lines = $derived((update?.body ?? '').split('\n').map((line) => line.trim()).filter(Boolean));

	async function install() {
		try {
			await installUpdate();
		} catch (e) {
			notifyError(e);
		}
	}
</script>

{#if visible && update}
	<div
		class="flex shrink-0 items-center justify-between gap-3 border-b border-indigo-200 bg-indigo-50 px-4 py-2 text-sm dark:border-indigo-900 dark:bg-indigo-950"
	>
		<span>Está disponível uma nova versão ({update.version}).</span>
		<span class="flex items-center gap-2">
			<button class="btn-primary" onclick={() => (updates.notesOpen = true)}>Ver novidades</button>
			<button class="btn-ghost" onclick={() => (updates.dismissed = update.version)} aria-label="Lembrar mais tarde" title="Mais tarde">✕</button>
		</span>
	</div>
{/if}

{#if updates.notesOpen && update}
	<Modal title="Novidades da versão {update.version}" onclose={() => !updates.installing && (updates.notesOpen = false)}>
		<div class="max-h-[50vh] space-y-1 overflow-y-auto text-sm">
			{#each lines as line, index (index)}
				{#if line.startsWith('#')}
					<h3 class="pt-2 font-medium">{line.replace(/^#+\s*/, '')}</h3>
				{:else if line.startsWith('- ')}
					<p class="flex gap-2"><span class="text-stone-400">•</span><span>{line.slice(2)}</span></p>
				{:else}
					<p>{line}</p>
				{/if}
			{/each}
		</div>
		<div class="mt-4 flex items-center justify-end gap-2">
			{#if updates.installing}
				<span class="muted mr-auto">A transferir{updates.progress !== null ? ` · ${updates.progress}%` : '…'}</span>
			{:else}
				<span class="muted mr-auto">A aplicação fecha e reabre automaticamente. Os dados mantêm-se.</span>
			{/if}
			<button class="btn-secondary" disabled={updates.installing} onclick={() => (updates.notesOpen = false)}>Mais tarde</button>
			<button class="btn-primary" disabled={updates.installing} onclick={install}>Atualizar agora</button>
		</div>
	</Modal>
{/if}
