<script lang="ts">
	import './layout.css';
	import { onMount, untrack } from 'svelte';
	import { getVersion } from '@tauri-apps/api/app';
	import { page } from '$app/state';
	import { listAlerts, type Alert } from '#lib/api/reports.ts';
	import { runBackgroundTasks } from '#lib/background.ts';
	import PrivacyToggle from '#lib/components/PrivacyToggle.svelte';
	import QuickAdd from '#lib/components/QuickAdd.svelte';
	import SearchPalette from '#lib/components/SearchPalette.svelte';
	import TitleBar from '#lib/components/TitleBar.svelte';
	import UpdateBanner from '#lib/components/UpdateBanner.svelte';
	import Toasts from '#lib/components/Toasts.svelte';
	import UnlockScreen from '#lib/components/UnlockScreen.svelte';
	import { errorMessage, isTauri } from '#lib/graphql.ts';
	import { dataChanged, loadRefs, refs } from '#lib/refs.svelte.ts';
	import { lock, status, type Status } from '#lib/session.ts';
	import { applyPrivacy, togglePrivacy } from '#lib/privacy.svelte.ts';
	import { app, loadSettings, restoreTheme } from '#lib/settings.svelte.ts';
	import { openQuickAdd, ui } from '#lib/ui.svelte.ts';
	import { today } from '#lib/format.ts';
	import { checkForUpdates, updates } from '#lib/updates.svelte.ts';

	let { children } = $props();

	restoreTheme();
	applyPrivacy();

	const BACKGROUND_INTERVAL = 60 * 60 * 1000;
	const CHECK_INTERVAL = 60 * 1000;
	const UPDATE_INTERVAL = 30 * 1000;

	const links = [
		{ href: '/', label: 'Dashboard' },
		{ href: '/transactions', label: 'Movimentos' },
		{ href: '/monthly', label: 'Vista mensal' },
		{ href: '/accounts', label: 'Contas' },
		{ href: '/categories', label: 'Categorias' },
		{ href: '/recurrences', label: 'Recorrências' },
		{ href: '/credits', label: 'Créditos' },
		{ href: '/goals', label: 'Objetivos' },
		{ href: '/purchases', label: 'Compras planeadas' },
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
	let version = $state('');
	let idleTimer: ReturnType<typeof setTimeout>;
	let backgroundTimer: ReturnType<typeof setInterval>;
	let lastRun = { at: 0, day: '' };

	async function background() {
		lastRun = { at: Date.now(), day: today() };
		const result = await runBackgroundTasks();
		alerts = result.alerts;
		return result.materialized;
	}

	async function refresh() {
		session = await status();
		ready = false;
		if (!session.unlocked) return;

		try {
			await loadSettings();
			await loadRefs();
			ready = true;
			await background();
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
		} else if ((event.ctrlKey || event.metaKey) && key === 'h') {
			event.preventDefault();
			togglePrivacy();
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

	$effect(() => {
		if (!ready || !app.settings?.checkUpdates) return;
		checkForUpdates();
		const timer = setInterval(checkForUpdates, UPDATE_INTERVAL);
		return () => clearInterval(timer);
	});

	$effect(() => {
		refs.version;
		untrack(() => {
			if (ready) listAlerts().then((list) => (alerts = list), () => undefined);
		});
	});

	const active = (href: string) => (href === '/' ? page.url.pathname === '/' : page.url.pathname.startsWith(href));

	onMount(() => {
		refresh();
		if (isTauri) getVersion().then((v) => (version = v));
		backgroundTimer = setInterval(async () => {
			if (!ready || (Date.now() - lastRun.at < BACKGROUND_INTERVAL && today() === lastRun.day)) return;
			if ((await background()) > 0) await dataChanged();
		}, CHECK_INTERVAL);
		return () => {
			clearInterval(backgroundTimer);
			clearTimeout(idleTimer);
		};
	});
</script>

<svelte:window onkeydown={shortcuts} onpointerdown={resetIdle} />

<div class="flex h-screen flex-col">
{#if isTauri}
	<TitleBar onlock={ready ? handleLock : undefined} />
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
			<span class="font-semibold">Gestor de Despesas</span>
			<div class="flex items-center gap-1">
				<PrivacyToggle />
				<button class="btn-ghost" onclick={handleLock} aria-label="Bloquear" title="Bloquear">
					<svg viewBox="0 0 24 24" class="size-4" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" aria-hidden="true">
						<rect x="5" y="11" width="14" height="10" rx="2" />
						<path d="M8 11V7a4 4 0 0 1 8 0v4" />
					</svg>
				</button>
				<button class="btn-primary" onclick={() => openQuickAdd()} aria-label="Adicionar">+</button>
			</div>
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
			<div class="mb-3 flex items-center justify-between px-3 pt-2">
				<span class="font-semibold">Gestor de Despesas</span>
				{#if !isTauri}<PrivacyToggle />{/if}
			</div>
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
			{#if version}
				<div class="mt-auto flex items-center gap-2 px-3 pt-2 text-xs text-stone-400 dark:text-stone-500">
					<span>v{version}</span>
					{#if updates.available}
						<button
							class="flex items-center gap-1 rounded-md px-1.5 py-0.5 text-indigo-600 hover:bg-indigo-50 dark:text-indigo-400 dark:hover:bg-indigo-950"
							onclick={() => (updates.notesOpen = true)}
							title="Nova versão {updates.available.version} disponível"
							aria-label="Ver novidades da versão {updates.available.version}"
						>
							<svg viewBox="0 0 24 24" class="size-3.5" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
								<path d="M12 4v11M7 10l5 5 5-5M5 20h14" />
							</svg>
							{updates.available.version}
						</button>
					{/if}
				</div>
			{/if}
		</nav>
		<div class="flex min-h-0 min-w-0 flex-1 flex-col">
			<UpdateBanner />
			<main class="flex-1 overflow-y-auto p-4 md:p-8">
				{@render children()}
			</main>
		</div>
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
