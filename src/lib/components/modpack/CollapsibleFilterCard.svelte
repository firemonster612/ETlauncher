<script lang="ts">
	import { ChevronDown } from '@lucide/svelte';
	import { slide } from 'svelte/transition';

	interface Props {
		title: string;
		open?: boolean;
		badge?: number;
		onToggle?: (open: boolean) => void;
		children?: import('svelte').Snippet;
	}

	let { title, open = $bindable(true), badge, onToggle, children }: Props = $props();

	function handleToggle() {
		open = !open;
		onToggle?.(open);
	}
</script>

<div class="filter-card" data-open={open}>
	<button type="button" class="filter-card-header w-full" onclick={handleToggle}>
		<span class="flex items-center gap-2">
			<span class="text-sm font-semibold">{title}</span>
			{#if badge}
				<span class="bg-primary text-primary-foreground rounded-full px-1.5 text-[10px] font-bold">
					{badge}
				</span>
			{/if}
		</span>
		<ChevronDown class="filter-card-chevron text-muted-foreground h-4 w-4" />
	</button>
	{#if open}
		<div class="filter-card-content" transition:slide={{ duration: 150 }}>
			{@render children?.()}
		</div>
	{/if}
</div>
