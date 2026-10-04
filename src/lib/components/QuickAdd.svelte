<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import { listTemplates, type Template } from '#lib/api/templates.ts';
	import type { TransactionInput } from '#lib/api/transactions.ts';
	import { today } from '#lib/format.ts';
	import { dataChanged, refs } from '#lib/refs.svelte.ts';
	import Modal from './Modal.svelte';
	import TransactionForm from './TransactionForm.svelte';
	import TransferForm from './TransferForm.svelte';

	let {
		onclose,
		mode = 'transaction',
		initial = null
	}: { onclose: () => void; mode?: 'transaction' | 'transfer'; initial?: Partial<TransactionInput> | null } = $props();

	let tab = $state(untrack(() => mode));
	let templates = $state<Template[]>([]);
	let input = $state<TransactionInput>(blank());
	let formKey = $state(0);

	function blank(): TransactionInput {
		return {
			accountId: refs.accounts[0]?.id ?? '',
			categoryId: null,
			kind: 'OUTCOME',
			amount: 0,
			date: today(),
			description: '',
			notes: null,
			confirmed: false,
			oneOff: false,
			...initial
		};
	}

	function useTemplate(template: Template) {
		input = {
			...blank(),
			accountId: template.accountId,
			categoryId: template.categoryId,
			kind: template.kind,
			amount: template.amount ?? 0,
			description: template.description
		};
		formKey++;
	}

	async function saved() {
		await dataChanged();
		onclose();
	}

	onMount(async () => {
		templates = await listTemplates().catch(() => []);
	});
</script>

<Modal title="Adicionar" {onclose}>
	<div class="mb-4 flex gap-2">
		<button class={tab === 'transaction' ? 'btn-primary' : 'btn-secondary'} onclick={() => (tab = 'transaction')}>Movimento</button>
		<button class={tab === 'transfer' ? 'btn-primary' : 'btn-secondary'} onclick={() => (tab = 'transfer')}>Transferência</button>
	</div>

	{#if tab === 'transaction'}
		{#if templates.length}
			<div class="mb-4 flex flex-wrap gap-2" aria-label="Templates">
				{#each templates as template (template.id)}
					<button class="badge hover:bg-indigo-100 dark:hover:bg-indigo-900" onclick={() => useTemplate(template)}>
						{template.name}
					</button>
				{/each}
			</div>
		{/if}
		{#key formKey}
			<TransactionForm initial={input} onsaved={saved} oncancel={onclose} />
		{/key}
	{:else}
		<TransferForm
			initial={{
				fromAccountId: refs.accounts[0]?.id ?? '',
				toAccountId: refs.accounts[1]?.id ?? '',
				amount: 0,
				date: today(),
				description: ''
			}}
			onsaved={saved}
			oncancel={onclose}
		/>
	{/if}
</Modal>
