<script lang="ts">
	import { Loader2, Download } from '@lucide/svelte';
	import { formatBytes, formatSpeed, formatEta } from '$lib/utils/format';

	interface Props {
		stage: string;
		progress: number;
		currentItem?: string;
		totalBytes?: number;
		downloadedBytes?: number;
		totalItems?: number;
		completedItems?: number;
		speedBytesPerSec?: number;
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
		speedBytesPerSec = 0,
		compact = false,
	}: Props = $props();

	// Calculate ETA based on remaining bytes and speed
	const eta = $derived.by(() => {
		if (speedBytesPerSec <= 0 || !totalBytes || totalBytes <= 0 || !downloadedBytes) return '';
		if (downloadedBytes >= totalBytes) return '';
		const remainingBytes = totalBytes - downloadedBytes;
		const seconds = remainingBytes / speedBytesPerSec;
		return formatEta(seconds);
	});

	const hasSpeed = $derived(speedBytesPerSec > 0);
</script>

<div class="space-y-2">
	<!-- Header with stage and speed/ETA -->
	<div class="flex items-center justify-between gap-2">
		<div class="flex min-w-0 items-center gap-2 text-sm">
			<Loader2 class="text-primary h-4 w-4 flex-shrink-0 animate-spin" />
			<span class="truncate font-medium">{stage}</span>
		</div>

		<!-- Speed and ETA on the right (when not compact) -->
		{#if hasSpeed && !compact}
			<div class="text-muted-foreground flex flex-shrink-0 items-center gap-1.5 text-xs">
				<Download class="h-3 w-3" />
				<span class="font-mono">{formatSpeed(speedBytesPerSec)}</span>
				{#if eta}
					<span class="text-muted-foreground/60">•</span>
					<span>{eta}</span>
				{/if}
			</div>
		{/if}
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
		<div class="text-muted-foreground flex items-center justify-between text-xs">
			{#if totalBytes && downloadedBytes !== undefined}
				<span>{formatBytes(downloadedBytes)} / {formatBytes(totalBytes)}</span>
			{:else if totalItems && completedItems !== undefined}
				<span>{completedItems} / {totalItems} items</span>
			{:else}
				<span></span>
			{/if}
			<span class="font-mono">{Math.round(progress)}%</span>
		</div>
	{:else if hasSpeed}
		<!-- Compact mode with speed -->
		<div class="text-muted-foreground flex items-center justify-between text-xs">
			<span class="font-mono">{formatSpeed(speedBytesPerSec)}</span>
			{#if eta}
				<span>{eta}</span>
			{/if}
		</div>
	{/if}
</div>
