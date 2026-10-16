<script lang="ts">
	let { color, size = 'md' }: { color: string; size?: 'sm' | 'md' | 'lg' } = $props();

	const sizes = { sm: 'size-5', md: 'size-10', lg: 'size-20' };
</script>

<span class="inline-flex shrink-0 items-end justify-center overflow-hidden rounded-full {sizes[size]}" style:background-color={color} aria-hidden="true">
	<svg viewBox="0 0 24 24" class="size-[85%] text-stone-800/45" fill="currentColor">
		<circle cx="12" cy="9" r="4.5" />
		<path d="M3 24c0-5.5 4-9 9-9s9 3.5 9 9z" />
	</svg>
</span>
