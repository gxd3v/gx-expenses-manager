<script lang="ts">
	let { value, label }: { value: number; label: string } = $props();

	const percent = $derived(Math.round(Math.min(Math.max(value, 0), 1) * 100));
</script>

<div
	class="h-2 w-full overflow-hidden rounded-full bg-indigo-100 dark:bg-indigo-950"
	role="progressbar"
	aria-label={label}
	aria-valuenow={percent}
	aria-valuemin={0}
	aria-valuemax={100}
>
	<div class="h-full rounded-full bg-indigo-600 dark:bg-indigo-400" style:width="{percent}%"></div>
</div>
