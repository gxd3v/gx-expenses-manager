<script lang="ts">
	import { untrack } from 'svelte';
	import { accountKinds, createAccount, updateAccount, type Account } from '#lib/accounts.ts';
	import { fromCents, toCents } from '#lib/money.ts';

	let {
		account,
		onsaved,
		oncancel
	}: { account: Account | null; onsaved: () => void; oncancel: () => void } = $props();

	let form = $state(
		untrack(() => ({
			name: account?.name ?? '',
			kind: account?.kind ?? 'BANK',
			currency: account?.currency ?? 'EUR',
			initialBalance: fromCents(account?.initialBalance ?? 0),
			color: account?.color ?? '#6366f1'
		}))
	);
	let error = $state('');
	let busy = $state(false);

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		const initialBalance = toCents(form.initialBalance);
		if (initialBalance === null) {
			error = 'Saldo inicial inválido.';
			return;
		}

		busy = true;
		try {
			const input = { ...form, initialBalance };
			await (account ? updateAccount(account.id, input) : createAccount(input));
			onsaved();
		} catch (e) {
			error = String(e);
		} finally {
			busy = false;
		}
	}
</script>

<form onsubmit={submit} class="grid max-w-lg grid-cols-2 gap-4 rounded-xl border border-slate-200 p-6 dark:border-slate-800">
	<label class="col-span-2 space-y-1 text-sm">
		<span>Nome</span>
		<input bind:value={form.name} class="input" required />
	</label>

	<label class="space-y-1 text-sm">
		<span>Tipo</span>
		<select bind:value={form.kind} class="input">
			{#each Object.entries(accountKinds) as [value, label] (value)}
				<option {value}>{label}</option>
			{/each}
		</select>
	</label>

	<label class="space-y-1 text-sm">
		<span>Moeda</span>
		<input bind:value={form.currency} maxlength="3" class="input uppercase" required />
	</label>

	<label class="space-y-1 text-sm">
		<span>Saldo inicial</span>
		<input bind:value={form.initialBalance} inputmode="decimal" class="input text-right" />
	</label>

	<label class="space-y-1 text-sm">
		<span>Cor</span>
		<input type="color" bind:value={form.color} class="block h-9 w-full rounded-md" />
	</label>

	{#if error}
		<p class="col-span-2 text-sm text-red-600">{error}</p>
	{/if}

	<div class="col-span-2 flex justify-end gap-2">
		<button type="button" onclick={oncancel} class="btn-secondary">Cancelar</button>
		<button type="submit" disabled={busy} class="btn-primary">Guardar</button>
	</div>
</form>
