<script lang="ts">
	import { refs } from '#lib/refs.svelte.ts';

	let {
		value = $bindable(),
		allowEmpty = false,
		emptyLabel = 'Todas as contas',
		exclude = null
	}: { value: string | null; allowEmpty?: boolean; emptyLabel?: string; exclude?: string | null } = $props();
</script>

<select bind:value class="input">
	{#if allowEmpty}
		<option value={null}>{emptyLabel}</option>
	{/if}
	{#each refs.accounts.filter((a) => a.id !== exclude) as account (account.id)}
		<option value={account.id}>{account.name}</option>
	{/each}
</select>
