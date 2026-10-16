<script lang="ts">
	import { onMount } from 'svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import PrivacyToggle from './PrivacyToggle.svelte';
	import UserAvatar from './UserAvatar.svelte';
	import { current, jointsOf } from '#lib/user.svelte.ts';
	import type { Profile } from '#lib/session.ts';

	let {
		onlock,
		onjoint,
		onback
	}: { onlock?: () => void; onjoint?: (joint: Profile) => void; onback?: () => void } = $props();

	const joints = $derived(jointsOf(current.user));

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
		Gestor de Despesas
	</div>

	<div class="flex h-full">
		<PrivacyToggle compact />
		{#if onback && current.returnTo}
			<button class="titlebar-button !w-auto gap-2 !px-3 text-xs" onclick={onback} title="Voltar para {current.returnTo.name}">
				<span aria-hidden="true">←</span>
				<UserAvatar color={current.returnTo.color} size="sm" />
				<span class="max-w-32 truncate">{current.returnTo.name}</span>
			</button>
		{/if}
		{#if onjoint}
			{#each joints as joint (joint.id)}
				<button class="titlebar-button !w-auto gap-2 !px-3 text-xs" onclick={() => onjoint(joint)} title="Trocar para {joint.name}">
					<UserAvatar color={joint.color} joint size="sm" />
					<span class="max-w-32 truncate">{joint.name}</span>
				</button>
			{/each}
		{/if}
		{#if onlock && current.user}
			<button
				class="titlebar-button !w-auto gap-2 !px-3 text-xs"
				onclick={onlock}
				aria-label="Trocar de utilizador (Ctrl+L)"
				title="Trocar de utilizador (Ctrl+L)"
			>
				<UserAvatar color={current.user.color} joint={current.user.joint} size="sm" />
				<span class="max-w-40 truncate font-medium">{current.user.name}</span>
			</button>
		{/if}
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
