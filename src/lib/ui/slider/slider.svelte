<script lang="ts">
	import { Slider as SliderPrimitive } from 'bits-ui';
	import { cn } from '$lib/utils.js';

	type Props = {
		ref?: HTMLSpanElement | null;
		value?: number;
		onValueChange?: (value: number) => void;
		onValueCommit?: (value: number) => void;
		min?: number;
		max?: number;
		step?: number;
		disabled?: boolean;
		class?: string;
	};

	let {
		ref = $bindable(null),
		value = $bindable(50),
		class: className,
		onValueChange,
		onValueCommit,
		min = 0,
		max = 100,
		step = 1,
		disabled = false,
	}: Props = $props();

	function handleValueChange(v: number) {
		value = v;
		onValueChange?.(v);
	}
</script>

<SliderPrimitive.Root
	bind:ref
	type="single"
	{value}
	onValueChange={handleValueChange}
	{onValueCommit}
	{min}
	{max}
	{step}
	{disabled}
	class={cn(
		'border-foreground/60 bg-background relative flex h-3 w-full touch-none items-center border-2 select-none',
		className
	)}
>
	{#snippet children({ thumbItems })}
		<SliderPrimitive.Range class="bg-primary absolute h-full" />
		{#each thumbItems as thumb (thumb.index)}
			<SliderPrimitive.Thumb
				index={thumb.index}
				class={cn(
					'border-foreground/60 bg-background block size-5 border-2 transition-colors',
					'hover:border-foreground',
					'focus-visible:border-primary focus-visible:ring-ring/50 focus-visible:ring-[3px] focus-visible:outline-none',
					'disabled:pointer-events-none disabled:opacity-50'
				)}
			/>
		{/each}
	{/snippet}
</SliderPrimitive.Root>
