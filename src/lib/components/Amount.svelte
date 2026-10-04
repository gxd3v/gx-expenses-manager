<script lang="ts">
	import { formatMoney } from '#lib/format.ts';

	let { value, currency, signed = true }: { value: number; currency?: string; signed?: boolean } = $props();

	const tone = $derived(
		!signed || value === 0 ? '' : value > 0 ? 'text-emerald-700 dark:text-emerald-400' : 'text-red-700 dark:text-red-400'
	);
</script>

<span class="money tabular-nums whitespace-nowrap {tone}">{signed && value > 0 ? '+' : ''}{formatMoney(value, currency)}</span>
