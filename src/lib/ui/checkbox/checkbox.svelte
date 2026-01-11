<script lang="ts">
	import { Checkbox as CheckboxPrimitive } from 'bits-ui';
	import CheckIcon from '@lucide/svelte/icons/check';
	import { cn, type WithoutChildrenOrChild } from '$lib/utils.js';

	let {
		ref = $bindable(null),
		checked = $bindable(false),
		class: className,
		onCheckedChange,
		...restProps
	}: WithoutChildrenOrChild<CheckboxPrimitive.RootProps> & {
		checked?: boolean;
	} = $props();

	function handleCheckedChange(v: boolean | 'indeterminate') {
		if (v === 'indeterminate') return;
		checked = v;
		onCheckedChange?.(v);
	}
</script>

<CheckboxPrimitive.Root
	bind:ref
	{checked}
	onCheckedChange={handleCheckedChange}
	class={cn(
		'peer border-muted-foreground bg-background flex size-5 shrink-0 items-center justify-center rounded-sm border-2 border-solid transition-colors outline-none',
		'hover:border-foreground',
		'focus-visible:border-primary focus-visible:ring-ring/50 focus-visible:ring-[3px]',
		'data-[state=checked]:border-primary data-[state=checked]:bg-primary data-[state=checked]:text-primary-foreground',
		'disabled:cursor-not-allowed disabled:opacity-50',
		className
	)}
	{...restProps}
>
	{#snippet children({ checked })}
		{#if checked}
			<CheckIcon class="size-3.5" strokeWidth={3} />
		{/if}
	{/snippet}
</CheckboxPrimitive.Root>
