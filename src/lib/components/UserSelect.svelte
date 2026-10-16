<script lang="ts">
	import { createUser, migrateUser, unlock, USER_COLORS, type Profile, type Status } from '#lib/session.ts';
	import { lastUserId } from '#lib/user.svelte.ts';
	import UserAvatar from './UserAvatar.svelte';

	let { status, onunlock }: { status: Status; onunlock: () => void } = $props();

	const MIN_LENGTH = 8;

	type Mode = 'pick' | 'unlock' | 'create' | 'migrate';

	const initialMode = (): Mode => (status.legacy ? 'migrate' : status.users.length ? 'pick' : 'create');

	let mode = $state<Mode>(initialMode());
	let selected = $state<Profile | null>(null);
	let form = $state({ name: '', color: USER_COLORS[0], password: '', confirmation: '' });
	let error = $state('');
	let busy = $state(false);

	const last = lastUserId();

	function open(next: Mode, user: Profile | null = null) {
		mode = next;
		selected = user;
		error = '';
		form = { name: '', color: USER_COLORS[status.users.length % USER_COLORS.length], password: '', confirmation: '' };
	}

	function validate(): string {
		if (mode === 'unlock') return '';
		if (!form.name.trim()) return 'O nome é obrigatório.';
		if (mode === 'migrate') return '';
		if (form.password.length < MIN_LENGTH) return `A password tem de ter pelo menos ${MIN_LENGTH} caracteres.`;
		if (form.password !== form.confirmation) return 'As passwords não coincidem.';
		return '';
	}

	function run(): Promise<void> {
		if (mode === 'unlock' && selected) return unlock(selected.id, form.password);
		if (mode === 'migrate') return migrateUser(form.name, form.color, form.password);
		return createUser(form.name, form.color, form.password);
	}

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		error = validate();
		if (error) return;

		busy = true;
		try {
			await run();
			form.password = '';
			form.confirmation = '';
			onunlock();
		} catch (e) {
			error = String(e);
		} finally {
			busy = false;
		}
	}

	function focus(node: HTMLElement) {
		node.focus();
	}

	const labels: Record<Mode, string> = { pick: '', unlock: 'Desbloquear', create: 'Criar utilizador', migrate: 'Continuar' };
</script>

<div class="flex h-full items-center justify-center overflow-y-auto bg-stone-100 p-4 dark:bg-stone-950">
	{#if mode === 'pick'}
		<div class="w-full max-w-2xl text-center">
			<h1 class="text-lg font-semibold">Gestor de Despesas</h1>
			<p class="muted">Escolha do utilizador</p>
			<div class="mt-8 flex flex-wrap justify-center gap-4">
				{#each status.users as user (user.id)}
					<button
						class="flex w-32 flex-col items-center gap-2 rounded-xl p-3 transition-colors hover:bg-white dark:hover:bg-stone-900 {user.id === last
							? 'bg-white/60 dark:bg-stone-900/60'
							: ''}"
						onclick={() => open('unlock', user)}
					>
						<UserAvatar color={user.color} size="lg" />
						<span class="w-full truncate text-sm font-medium">{user.name}</span>
					</button>
				{/each}
				<button class="flex w-32 flex-col items-center gap-2 rounded-xl p-3 transition-colors hover:bg-white dark:hover:bg-stone-900" onclick={() => open('create')}>
					<span class="flex size-20 items-center justify-center rounded-full border-2 border-dashed border-stone-300 text-3xl text-stone-400 dark:border-stone-700">+</span>
					<span class="text-sm text-stone-500">Novo utilizador</span>
				</button>
			</div>
		</div>
	{:else}
		<form onsubmit={submit} class="card w-full max-w-sm space-y-4 shadow">
			{#if mode === 'unlock' && selected}
				<div class="flex flex-col items-center gap-2 text-center">
					<UserAvatar color={selected.color} size="lg" />
					<h1 class="text-lg font-semibold">{selected.name}</h1>
				</div>
			{:else}
				<div>
					<h1 class="text-lg font-semibold">{mode === 'migrate' ? 'Gestor de Despesas' : 'Novo utilizador'}</h1>
					<p class="muted">
						{mode === 'migrate'
							? 'A aplicação passa a suportar vários utilizadores. Os dados existentes ficam associados a um utilizador com o nome indicado.'
							: 'Cada utilizador tem os seus próprios dados, protegidos pela sua password.'}
					</p>
				</div>

				<label class="label">
					Nome
					<input bind:value={form.name} class="input" maxlength="40" required use:focus />
				</label>

				<div class="label">
					Cor
					<div class="flex flex-wrap gap-2">
						{#each USER_COLORS as color (color)}
							<button
								type="button"
								class="rounded-full p-0.5 ring-2 {form.color === color ? 'ring-indigo-500' : 'ring-transparent'}"
								onclick={() => (form.color = color)}
								aria-label="Cor {color}"
								aria-pressed={form.color === color}
							>
								<UserAvatar {color} size="md" />
							</button>
						{/each}
					</div>
				</div>
			{/if}

			{#if mode === 'create'}
				<p class="rounded-md bg-amber-50 p-3 text-sm text-amber-800 dark:bg-amber-950 dark:text-amber-200">
					Os dados ficam encriptados com esta password. Sem ela não é possível recuperá-los; recomenda-se guardá-la num gestor de passwords.
				</p>
			{/if}

			<label class="label">
				{mode === 'migrate' ? 'Password atual' : 'Password'}
				{#if mode === 'unlock'}
					<input type="password" bind:value={form.password} class="input" autocomplete="current-password" required use:focus />
				{:else}
					<input type="password" bind:value={form.password} class="input" autocomplete={mode === 'create' ? 'new-password' : 'current-password'} required />
				{/if}
			</label>

			{#if mode === 'create'}
				<label class="label">
					Confirmar password
					<input type="password" bind:value={form.confirmation} class="input" autocomplete="new-password" required />
				</label>
			{/if}

			{#if error}
				<p class="text-sm text-red-600" role="alert">{error}</p>
			{/if}

			<div class="flex gap-2">
				{#if status.users.length && mode !== 'migrate'}
					<button type="button" class="btn-secondary flex-1" onclick={() => open('pick')}>Voltar</button>
				{/if}
				<button type="submit" disabled={busy} class="btn-primary flex-1">{busy ? 'A abrir…' : labels[mode]}</button>
			</div>
		</form>
	{/if}
</div>
