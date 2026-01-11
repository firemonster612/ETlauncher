<script lang="ts">
	import type { HTMLTextareaAttributes } from 'svelte/elements';
	import { cn, type WithElementRef } from '$lib/utils.js';

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
		'bg-background border-border placeholder:text-muted-foreground w-full resize-none border-2 px-3 py-2 text-sm transition-colors outline-none disabled:cursor-not-allowed disabled:opacity-50',
		'focus:border-primary',
		'aria-invalid:border-destructive',
		className
	)}
	{...restProps}
></textarea>
