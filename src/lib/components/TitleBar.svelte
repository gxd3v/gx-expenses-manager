<script lang="ts">
	import { onMount } from 'svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';

	const appWindow = getCurrentWindow();

	let maximized = $state(false);

	async function sync() {
		maximized = await appWindow.isMaximized();
	}

	onMount(() => {
		sync();
		const unlisten = appWindow.onResized(sync);
		return () => {
			unlisten.then((stop) => stop());
		};
	});
</script>

<header
	data-tauri-drag-region
	class="relative z-[60] hidden h-9 shrink-0 items-center justify-between border-b border-stone-200 bg-stone-50 select-none md:flex dark:border-stone-800 dark:bg-stone-950"
>
	<div data-tauri-drag-region class="flex items-center gap-2 pl-3 text-xs font-medium text-stone-500 dark:text-stone-400">
		<img src="/favicon.png" alt="" class="pointer-events-none size-4" />
		Expenses Manager
	</div>

	<div class="flex h-full">
		<button class="titlebar-button" onclick={() => appWindow.minimize()} aria-label="Minimizar" title="Minimizar">
			<svg viewBox="0 0 10 10" class="size-2.5" aria-hidden="true"><path d="M0 5h10" stroke="currentColor" stroke-width="1" /></svg>
		</button>
		<button
			class="titlebar-button"
			onclick={() => appWindow.toggleMaximize()}
			aria-label={maximized ? 'Restaurar' : 'Maximizar'}
			title={maximized ? 'Restaurar' : 'Maximizar'}
		>
			{#if maximized}
				<svg viewBox="0 0 10 10" class="size-2.5" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1">
					<path d="M2.5 2.5V0.5h7v7h-2" />
					<rect x="0.5" y="2.5" width="7" height="7" />
				</svg>
			{:else}
				<svg viewBox="0 0 10 10" class="size-2.5" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1">
					<rect x="0.5" y="0.5" width="9" height="9" />
				</svg>
			{/if}
		</button>
		<button class="titlebar-button hover:!bg-red-600 hover:!text-white" onclick={() => appWindow.close()} aria-label="Fechar" title="Fechar">
			<svg viewBox="0 0 10 10" class="size-2.5" aria-hidden="true" stroke="currentColor" stroke-width="1">
				<path d="M0 0l10 10M10 0L0 10" />
			</svg>
		</button>
	</div>
</header>
