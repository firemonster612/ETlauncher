<script lang="ts">
	import { Slider as SliderPrimitive } from 'bits-ui';
	import { cn } from '$lib/utils.js';

	type Props = {
		ref?: HTMLSpanElement | null;
		value?: [number, number];
		onValueChange?: (value: [number, number]) => void;
		onValueCommit?: (value: [number, number]) => void;
		min?: number;
		max?: number;
		step?: number;
		disabled?: boolean;
		class?: string;
	};

	let {
		ref = $bindable(null),
		value = $bindable([25, 75] as [number, number]),
		class: className,
		onValueChange,
		onValueCommit,
		min = 0,
		max = 100,
		step = 1,
		disabled = false,
	}: Props = $props();

	// Track previous value to detect which thumb moved
	let prevValue: [number, number] = [...value];

	function handleValueChange(v: number[]) {
		let [newMin, newMax] = v;

		// Detect which thumb changed
		const minMoved = newMin !== prevValue[0];
		const maxMoved = newMax !== prevValue[1];

		// Clamp to prevent passing or having same value (min gap of 1 step)
		if (minMoved && newMin >= prevValue[1]) {
			newMin = prevValue[1] - step;
		}
		if (maxMoved && newMax <= prevValue[0]) {
			newMax = prevValue[0] + step;
		}

		const tuple: [number, number] = [newMin, newMax];
		prevValue = [...tuple];
		value = tuple;
		onValueChange?.(tuple);
	}

	function handleValueCommit() {
		onValueCommit?.(value);
	}
</script>

<SliderPrimitive.Root
	bind:ref
	type="multiple"
	{value}
	onValueChange={handleValueChange}
	onValueCommit={handleValueCommit}
	autoSort={false}
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
