<script lang="ts">
	import { Loader2 } from '@lucide/svelte';

	interface Props {
		stage: string;
		progress: number;
		currentItem?: string;
		totalBytes?: number;
		downloadedBytes?: number;
		totalItems?: number;
		completedItems?: number;
		compact?: boolean;
	}

	let {
		stage,
		progress,
		currentItem,
		totalBytes,
		downloadedBytes,
		totalItems,
		completedItems,
		compact = false,
	}: Props = $props();

	function formatBytes(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
		return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
	}
</script>

<div class="space-y-2">
	<div class="flex items-center gap-2 text-sm">
		<Loader2 class="text-primary h-4 w-4 flex-shrink-0 animate-spin" />
		<span class="truncate font-medium">{stage}</span>
	</div>

	{#if currentItem && !compact}
		<p class="text-muted-foreground truncate text-xs" title={currentItem}>{currentItem}</p>
	{/if}

	<div class="bg-muted h-2 overflow-hidden rounded-full">
		<div
			class="bg-primary h-full transition-all duration-150 ease-out"
			style="width: {Math.min(100, Math.max(0, progress))}%"
		></div>
	</div>

	{#if !compact}
		{#if totalBytes && downloadedBytes !== undefined}
			<p class="text-muted-foreground text-xs">
				{formatBytes(downloadedBytes)} / {formatBytes(totalBytes)}
			</p>
		{:else if totalItems && completedItems !== undefined}
			<p class="text-muted-foreground text-xs">
				{completedItems} / {totalItems} items
			</p>
		{/if}
	{/if}
</div>
