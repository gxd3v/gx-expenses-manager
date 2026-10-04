<script lang="ts">
	let {
		label,
		value,
		hint = '',
		tone = 'neutral',
		plain = false,
		privateHint = false,
		ondetail
	}: {
		label: string;
		value: string;
		hint?: string;
		tone?: 'neutral' | 'positive' | 'negative';
		plain?: boolean;
		privateHint?: boolean;
		ondetail?: () => void;
	} = $props();

	const toneClass = {
		neutral: '',
		positive: 'text-emerald-700 dark:text-emerald-400',
		negative: 'text-red-700 dark:text-red-400'
	};
</script>

<div class="card">
	<p class="muted">{label}</p>
	<p class="mt-1 text-2xl font-semibold tabular-nums {toneClass[tone]}" class:money={!plain}>{value}</p>
	{#if hint}
		<p class="mt-1 text-xs text-stone-500" class:money={privateHint}>{hint}</p>
	{/if}
	{#if ondetail}
		<button class="mt-2 text-xs font-medium text-indigo-600 hover:underline dark:text-indigo-400" onclick={ondetail}>Detalhe</button>
	{/if}
</div>
