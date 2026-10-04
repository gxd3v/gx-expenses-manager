<script lang="ts">
	import { fromCents, toCents } from '#lib/format.ts';

	let {
		value = $bindable(),
		required = false,
		placeholder = '0,00',
		id
	}: { value: number | null; required?: boolean; placeholder?: string; id?: string } = $props();

	let text = $state(fromCents(value));
	let invalid = $state(false);

	$effect(() => {
		if (toCents(text) !== value) text = fromCents(value);
	});

	function input(event: Event & { currentTarget: HTMLInputElement }) {
		text = event.currentTarget.value;
		const cents = toCents(text);
		invalid = text.trim() !== '' && cents === null;
		value = cents;
	}
</script>

<input
	{id}
	value={text}
	oninput={input}
	inputmode="decimal"
	{placeholder}
	{required}
	aria-invalid={invalid}
	class="input text-right tabular-nums {invalid ? 'border-red-500' : ''}"
/>
