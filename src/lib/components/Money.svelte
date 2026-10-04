<script lang="ts">
	import { formatMoney } from '#lib/format.ts';

	let { value, currency }: { value: number; currency?: string } = $props();
</script>

<span class="money tabular-nums">{formatMoney(value, currency)}</span>
