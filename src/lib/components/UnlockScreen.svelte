<script lang="ts">
	import { unlock } from '#lib/session.ts';

	let { initialized, onunlock }: { initialized: boolean; onunlock: () => void } = $props();

	const MIN_LENGTH = 8;

	let password = $state('');
	let confirmation = $state('');
	let error = $state('');
	let busy = $state(false);

	function validate(): string {
		if (initialized) return '';
		if (password.length < MIN_LENGTH) return `A password tem de ter pelo menos ${MIN_LENGTH} caracteres.`;
		if (password !== confirmation) return 'As passwords não coincidem.';
		return '';
	}

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		error = validate();
		if (error) return;

		busy = true;
		try {
			await unlock(password);
			password = '';
			confirmation = '';
			onunlock();
		} catch (e) {
			error = String(e);
		} finally {
			busy = false;
		}
	}
</script>

<div class="flex h-full items-center justify-center bg-stone-100 p-4 dark:bg-stone-950">
	<form onsubmit={submit} class="card w-full max-w-sm space-y-4 shadow">
		<div>
			<h1 class="text-lg font-semibold">Expenses Manager</h1>
			<p class="muted">{initialized ? 'Introduz a password para desbloquear.' : 'Cria a password que protege os teus dados.'}</p>
		</div>

		{#if !initialized}
			<p class="rounded-md bg-amber-50 p-3 text-sm text-amber-800 dark:bg-amber-950 dark:text-amber-200">
				Os dados ficam encriptados com esta password. Se a perderes, não há forma de os recuperar — guarda-a num gestor de
				passwords.
			</p>
		{/if}

		<label class="label">
			Password
			<input type="password" bind:value={password} class="input" autocomplete="current-password" required />
		</label>

		{#if !initialized}
			<label class="label">
				Confirmar password
				<input type="password" bind:value={confirmation} class="input" autocomplete="new-password" required />
			</label>
		{/if}

		{#if error}
			<p class="text-sm text-red-600" role="alert">{error}</p>
		{/if}

		<button type="submit" disabled={busy} class="btn-primary w-full">
			{busy ? 'A abrir…' : initialized ? 'Desbloquear' : 'Criar'}
		</button>
	</form>
</div>
