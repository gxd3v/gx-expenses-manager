<script lang="ts">
	import { unlock } from '#lib/session.ts';

	let { initialized, onunlock }: { initialized: boolean; onunlock: () => void } = $props();

	let password = $state('');
	let confirmation = $state('');
	let error = $state('');
	let busy = $state(false);

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		if (!initialized && password !== confirmation) {
			error = 'As passwords não coincidem.';
			return;
		}

		busy = true;
		try {
			await unlock(password);
			onunlock();
		} catch (e) {
			error = e === 'wrong password' ? 'Password incorreta.' : String(e);
		} finally {
			busy = false;
		}
	}
</script>

<div class="flex h-screen items-center justify-center bg-slate-100 dark:bg-slate-950">
	<form onsubmit={submit} class="w-80 space-y-4 rounded-xl bg-white p-6 shadow dark:bg-slate-900">
		<h1 class="text-lg font-semibold">{initialized ? 'Desbloquear' : 'Criar password'}</h1>

		{#if !initialized}
			<p class="text-sm text-slate-500">
				Os dados ficam encriptados com esta password. Se a perderes, não há forma de os recuperar.
			</p>
		{/if}

		<input type="password" bind:value={password} placeholder="Password" class="input" required />

		{#if !initialized}
			<input type="password" bind:value={confirmation} placeholder="Confirmar password" class="input" required />
		{/if}

		{#if error}
			<p class="text-sm text-red-600">{error}</p>
		{/if}

		<button type="submit" disabled={busy} class="btn-primary w-full">
			{initialized ? 'Desbloquear' : 'Criar'}
		</button>
	</form>
</div>
