<script lang="ts">
	import { RadioGroup as RadioGroupPrimitive } from 'bits-ui';
	import { cn, type WithoutChildrenOrChild } from '$lib/utils.js';

	let {
		ref = $bindable(null),
		value = $bindable<string>(),
		class: className,
		children,
		onValueChange,
		...restProps
	}: WithoutChildrenOrChild<RadioGroupPrimitive.RootProps> & {
		children?: import('svelte').Snippet;
	} = $props();

	function handleValueChange(v: string) {
		value = v;
		onValueChange?.(v);
	}
</script>

<RadioGroupPrimitive.Root
	bind:ref
	{value}
	onValueChange={handleValueChange}
	class={cn('flex flex-col gap-2', className)}
	{...restProps}
>
	{@render children?.()}
</RadioGroupPrimitive.Root>
