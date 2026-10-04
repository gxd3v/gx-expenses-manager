<script lang="ts">
	import './layout.css';
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import type { Alert } from '#lib/api/reports.ts';
	import { runBackgroundTasks } from '#lib/background.ts';
	import QuickAdd from '#lib/components/QuickAdd.svelte';
	import SearchPalette from '#lib/components/SearchPalette.svelte';
	import TitleBar from '#lib/components/TitleBar.svelte';
	import Toasts from '#lib/components/Toasts.svelte';
	import UnlockScreen from '#lib/components/UnlockScreen.svelte';
	import { errorMessage, isTauri } from '#lib/graphql.ts';
	import { dataChanged, loadRefs } from '#lib/refs.svelte.ts';
	import { lock, status, type Status } from '#lib/session.ts';
	import { app, loadSettings } from '#lib/settings.svelte.ts';
	import { openQuickAdd, ui } from '#lib/ui.svelte.ts';

	let { children } = $props();

	const BACKGROUND_INTERVAL = 60 * 60 * 1000;

	const links = [
		{ href: '/', label: 'Dashboard' },
		{ href: '/transactions', label: 'Movimentos' },
		{ href: '/monthly', label: 'Vista mensal' },
		{ href: '/accounts', label: 'Contas' },
		{ href: '/categories', label: 'Categorias' },
		{ href: '/recurrences', label: 'Recorrências' },
		{ href: '/credits', label: 'Créditos' },
		{ href: '/goals', label: 'Objetivos' },
		{ href: '/forecast', label: 'Previsões' },
		{ href: '/reports', label: 'Gráficos' },
		{ href: '/reconciliation', label: 'Reconciliação' },
		{ href: '/templates', label: 'Templates' },
		{ href: '/settings', label: 'Definições' }
	];

	let session = $state<Status | null>(null);
	let ready = $state(false);
	let startupError = $state('');
	let alerts = $state<Alert[]>([]);
	let menuOpen = $state(false);
	let idleTimer: ReturnType<typeof setTimeout>;
	let backgroundTimer: ReturnType<typeof setInterval>;

	async function refresh() {
		session = await status();
		ready = false;
		if (!session.unlocked) return;

		try {
			await loadSettings();
			await loadRefs();
			ready = true;
			alerts = await runBackgroundTasks();
			await dataChanged();
			resetIdle();
		} catch (e) {
			startupError = errorMessage(e);
		}
	}

	async function handleLock() {
		clearTimeout(idleTimer);
		await lock();
		await refresh();
	}

	function resetIdle() {
		clearTimeout(idleTimer);
		const minutes = app.settings?.lockTimeoutMinutes ?? 0;
		if (session?.unlocked && minutes > 0) idleTimer = setTimeout(handleLock, minutes * 60 * 1000);
	}

	function typing(target: EventTarget | null): boolean {
		return target instanceof HTMLElement && ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName);
	}

	function shortcuts(event: KeyboardEvent) {
		resetIdle();
		if (!ready) return;
		const key = event.key.toLowerCase();
		if ((event.ctrlKey || event.metaKey) && key === 'k') {
			event.preventDefault();
			ui.search = true;
		} else if ((event.ctrlKey || event.metaKey) && key === 'l') {
			event.preventDefault();
			handleLock();
		} else if (key === 'n' && !event.ctrlKey && !event.metaKey && !typing(event.target) && !ui.quickAdd) {
			event.preventDefault();
			openQuickAdd();
		}
	}

	$effect(() => {
		page.url.pathname;
		menuOpen = false;
	});

	const active = (href: string) => (href === '/' ? page.url.pathname === '/' : page.url.pathname.startsWith(href));

	onMount(() => {
		refresh();
		backgroundTimer = setInterval(async () => {
			if (ready) alerts = await runBackgroundTasks();
		}, BACKGROUND_INTERVAL);
		return () => {
			clearInterval(backgroundTimer);
			clearTimeout(idleTimer);
		};
	});
</script>

<svelte:window onkeydown={shortcuts} onpointerdown={resetIdle} />

<div class="flex h-screen flex-col">
{#if isTauri}
	<TitleBar />
{/if}
<div class="relative min-h-0 flex-1">
{#if session && !session.unlocked}
	<UnlockScreen initialized={session.initialized} onunlock={refresh} />
{:else if startupError}
	<div class="flex h-full items-center justify-center p-8">
		<div class="card max-w-lg space-y-3">
			<h1 class="font-semibold">Não foi possível abrir os dados</h1>
			<p class="text-sm text-red-600">{startupError}</p>
			<button class="btn-secondary" onclick={handleLock}>Bloquear e tentar novamente</button>
		</div>
	</div>
{:else if ready}
	<div class="flex h-full flex-col md:flex-row">
		<header class="flex items-center justify-between border-b border-stone-200 px-4 py-2 md:hidden dark:border-stone-800">
			<button class="btn-ghost" onclick={() => (menuOpen = !menuOpen)} aria-expanded={menuOpen} aria-label="Menu">☰</button>
			<span class="font-semibold">GX Expenses</span>
			<button class="btn-primary" onclick={() => openQuickAdd()} aria-label="Adicionar">+</button>
		</header>
		{#if menuOpen}
			<button class="fixed inset-0 z-20 bg-black/40 md:hidden" onclick={() => (menuOpen = false)} aria-label="Fechar menu"></button>
		{/if}
		<nav
			class="fixed inset-y-0 left-0 z-30 flex w-64 flex-col gap-0.5 overflow-y-auto border-r border-stone-200 bg-stone-50 p-3 transition-transform md:static md:transition-none md:w-56 md:shrink-0 md:translate-x-0 dark:border-stone-800 dark:bg-stone-950 {menuOpen
				? 'translate-x-0'
				: '-translate-x-full'}"
			aria-label="Navegação principal"
		>
			<span class="mb-3 px-3 pt-2 font-semibold">GX Expenses</span>
			<button class="btn-primary mb-2" onclick={() => openQuickAdd()} title="Atalho: N">+ Adicionar</button>
			<button class="btn-secondary mb-3 justify-between" onclick={() => (ui.search = true)}>
				Pesquisar <kbd class="text-xs text-stone-400">Ctrl K</kbd>
			</button>
			{#each links as link (link.href)}
				<a
					href={link.href}
					aria-current={active(link.href) ? 'page' : undefined}
					class="rounded-md px-3 py-1.5 text-sm hover:bg-stone-100 dark:hover:bg-stone-800 {active(link.href)
						? 'bg-stone-100 font-medium dark:bg-stone-800'
						: 'text-stone-600 dark:text-stone-400'}"
				>
					{link.label}
					{#if link.href === '/' && alerts.length}
						<span class="ml-1 rounded-full bg-amber-500 px-1.5 text-xs text-white">{alerts.length}</span>
					{/if}
				</a>
			{/each}
			<button onclick={handleLock} class="btn-secondary mt-auto" title="Atalho: Ctrl L">Bloquear</button>
		</nav>
		<main class="flex-1 overflow-y-auto p-4 md:p-8">
			{@render children()}
		</main>
	</div>

	{#if ui.quickAdd}
		<QuickAdd mode={ui.quickAdd.mode} initial={ui.quickAdd.initial} onclose={() => (ui.quickAdd = null)} />
	{/if}
	{#if ui.search}
		<SearchPalette onclose={() => (ui.search = false)} />
	{/if}
{/if}
</div>
</div>

<Toasts />
