<script lang="ts">
	import { untrack } from 'svelte';
	import { accountKinds, saveAccount, type Account } from '#lib/api/accounts.ts';
	import MoneyInput from '#lib/components/MoneyInput.svelte';
	import { errorMessage } from '#lib/graphql.ts';
	import { app } from '#lib/settings.svelte.ts';
	import { notify } from '#lib/toasts.svelte.ts';

	let { account, onsaved, oncancel }: { account: Account | null; onsaved: () => void; oncancel: () => void } = $props();

	let form = $state(
		untrack(() => ({
			name: account?.name ?? '',
			kind: account?.kind ?? 'BANK',
			currency: account?.currency ?? app.settings?.currency ?? 'EUR',
			initialBalance: account?.initialBalance ?? 0 as number | null,
			color: account?.color ?? '#2a78d6',
			icon: account?.icon ?? ''
		}))
	);
	let error = $state('');
	let busy = $state(false);

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		busy = true;
		try {
			await saveAccount(account?.id ?? null, { ...form, initialBalance: form.initialBalance ?? 0, icon: form.icon || null });
			notify(account ? 'Conta atualizada' : 'Conta criada');
			onsaved();
		} catch (e) {
			error = errorMessage(e);
		} finally {
			busy = false;
		}
	}
</script>

<form onsubmit={submit} class="grid grid-cols-2 gap-4">
	<label class="label col-span-2">
		Nome
		<input bind:value={form.name} class="input" required />
	</label>
	<label class="label">
		Tipo
		<select bind:value={form.kind} class="input">
			{#each Object.entries(accountKinds) as [value, label] (value)}
				<option {value}>{label}</option>
			{/each}
		</select>
	</label>
	<label class="label">
		Moeda
		<input bind:value={form.currency} maxlength="3" class="input uppercase" required />
	</label>
	<label class="label">
		Saldo inicial
		<MoneyInput bind:value={form.initialBalance} />
	</label>
	<div class="grid grid-cols-2 gap-2">
		<label class="label">
			Cor
			<input type="color" bind:value={form.color} class="h-9 w-full rounded-md" />
		</label>
		<label class="label">
			Ícone
			<input bind:value={form.icon} maxlength="2" class="input text-center" placeholder="🏦" />
		</label>
	</div>

	{#if error}
		<p class="col-span-2 text-sm text-red-600" role="alert">{error}</p>
	{/if}

	<div class="col-span-2 flex justify-end gap-2">
		<button type="button" onclick={oncancel} class="btn-secondary">Cancelar</button>
		<button type="submit" disabled={busy} class="btn-primary">Guardar</button>
	</div>
</form>
