<script lang="ts">
	import { RadioGroup as RadioGroupPrimitive } from "bits-ui";
	import CircleIcon from "@lucide/svelte/icons/circle";
	import { cn, type WithoutChildrenOrChild } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		value,
		class: className,
		...restProps
	}: WithoutChildrenOrChild<RadioGroupPrimitive.ItemProps> = $props();
</script>

<RadioGroupPrimitive.Item
	bind:ref
	{value}
	class={cn(
		"border-2 border-foreground/60 bg-background data-[state=checked]:border-primary hover:border-foreground focus-visible:border-primary peer flex size-5 shrink-0 items-center justify-center rounded-full outline-none transition-colors disabled:cursor-not-allowed disabled:opacity-50",
		className
	)}
	{...restProps}
>
	{#snippet children({ checked })}
		{#if checked}
			<CircleIcon class="size-2.5 fill-primary text-primary" />
		{/if}
	{/snippet}
</RadioGroupPrimitive.Item>
