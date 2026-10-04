<script lang="ts">
	import { page } from '$app/state';
	import PageHeader from '#lib/components/PageHeader.svelte';
	import TransactionList from '#lib/components/TransactionList.svelte';
	import { openQuickAdd } from '#lib/ui.svelte.ts';

	const params = page.url.searchParams;
	const initialFilter = {
		search: params.get('search'),
		categoryId: params.get('category'),
		accountId: params.get('account'),
		confirmed: params.get('confirmed') === 'false' ? false : null
	};
</script>

<PageHeader title="Movimentos" subtitle="Receitas, despesas e transferências, incluindo movimentos futuros.">
	{#snippet actions()}
		<button class="btn-secondary" onclick={() => openQuickAdd('transfer')}>Nova transferência</button>
		<button class="btn-primary" onclick={() => openQuickAdd()}>Novo movimento</button>
	{/snippet}
</PageHeader>

{#key page.url.search}
	<TransactionList {initialFilter} />
{/key}
