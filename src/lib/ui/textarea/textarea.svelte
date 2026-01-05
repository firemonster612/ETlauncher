<script lang="ts">
	import type { HTMLTextareaAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		value = $bindable(),
		class: className,
		...restProps
	}: WithElementRef<HTMLTextareaAttributes, HTMLTextAreaElement> = $props();
</script>

<textarea
	bind:this={ref}
	bind:value
	class={cn(
		"w-full px-3 py-2 bg-background border-2 border-border text-sm transition-colors outline-none resize-none placeholder:text-muted-foreground disabled:cursor-not-allowed disabled:opacity-50",
		"focus:border-primary",
		"aria-invalid:border-destructive",
		className
	)}
	{...restProps}
></textarea>
