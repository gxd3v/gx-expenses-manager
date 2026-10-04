<script lang="ts">
	let {
		checked = $bindable(),
		label,
		disabled = false,
		onchange
	}: { checked: boolean; label: string; disabled?: boolean; onchange?: () => void } = $props();

	function toggle() {
		checked = !checked;
		onchange?.();
	}
</script>

<label class="inline-flex items-center gap-2.5 text-sm {disabled ? 'cursor-not-allowed opacity-50' : 'cursor-pointer'}">
	<button
		type="button"
		role="switch"
		aria-checked={checked}
		aria-label={label}
		{disabled}
		onclick={toggle}
		class="relative inline-flex h-5 w-9 shrink-0 items-center rounded-full transition-colors duration-200 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-500 {checked
			? 'bg-indigo-600 dark:bg-indigo-500'
			: 'bg-stone-300 dark:bg-stone-700'}"
	>
		<span
			class="inline-block size-4 rounded-full bg-white shadow-sm ring-1 ring-black/5 transition-transform duration-200 {checked
				? 'translate-x-[18px]'
				: 'translate-x-0.5'}"
		></span>
	</button>
	<span aria-hidden="true">{label}</span>
</label>
