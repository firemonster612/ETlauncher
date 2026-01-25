<script lang="ts">
	import { Loader2, StopCircle, Download } from '@lucide/svelte';
	import { modpackInstallStore } from '$lib/stores/modpackInstall.svelte';
	import { alertDialogStore } from '$lib/stores/alertDialog.svelte';
	import { Button } from '$lib/ui/button';
	import { formatBytes, formatSpeed, formatEta } from '$lib/utils/format';

	async function handleCancel() {
		if (!modpackInstallStore.modpackName || modpackInstallStore.isCancelling) return;

		const confirmed = await alertDialogStore.confirm({
			title: `Cancel installation of "${modpackInstallStore.modpackName}"?`,
			message: 'This will stop the download and remove any partially installed files.',
			type: 'warning',
			confirmText: 'Cancel Installation',
			cancelText: 'Continue',
		});

		if (confirmed) {
			modpackInstallStore.cancel();
		}
	}

	// Calculate ETA based on bytes (when available)
	const bytesEta = $derived.by(() => {
		const p = modpackInstallStore.progress;
		if (!p || !p.speedBytesPerSec || p.speedBytesPerSec <= 0) return '';
		if (!p.totalBytes || !p.downloadedBytes) return '';
		if (p.downloadedBytes >= p.totalBytes) return '';
		const remaining = p.totalBytes - p.downloadedBytes;
		return formatEta(remaining / p.speedBytesPerSec);
	});
</script>

{#if modpackInstallStore.isInstalling}
	<div class="bg-card border-border fixed right-0 bottom-0 left-0 z-40 border-t-2 p-4 shadow-lg">
		<div class="mx-auto flex max-w-4xl items-center gap-4">
			<!-- Progress info -->
			<div class="min-w-0 flex-1">
				<!-- Header with name -->
				<div class="mb-1 flex items-center gap-2 text-sm">
					<Loader2 class="text-primary h-4 w-4 flex-shrink-0 animate-spin" />
					<span class="truncate font-medium">
						{#if modpackInstallStore.isCancelling}
							Cancelling...
						{:else}
							Installing {modpackInstallStore.modpackName}
						{/if}
					</span>
				</div>

				{#if modpackInstallStore.progress}
					<!-- Stage, current item, and speed/ETA -->
					<div class="text-muted-foreground mb-1 flex items-center justify-between gap-2 text-xs">
						<span class="min-w-0 truncate">
							{modpackInstallStore.progress.stage}
							{#if modpackInstallStore.progress.currentItem}
								- {modpackInstallStore.progress.currentItem}
							{/if}
						</span>
						{#if modpackInstallStore.progress.speedBytesPerSec && modpackInstallStore.progress.speedBytesPerSec > 0}
							<span class="flex flex-shrink-0 items-center gap-1.5">
								<Download class="h-3 w-3" />
								<span class="font-mono">{formatSpeed(modpackInstallStore.progress.speedBytesPerSec)}</span>
								{#if bytesEta}
									<span class="text-muted-foreground/60">•</span>
									<span>{bytesEta}</span>
								{/if}
							</span>
						{/if}
					</div>

					<!-- Progress bar -->
					<div class="bg-muted h-2 overflow-hidden rounded-full">
						<div
							class="bg-primary h-full transition-all duration-150 ease-out"
							style="width: {Math.min(100, Math.max(0, modpackInstallStore.progress.progress))}%"
						></div>
					</div>

					<!-- Progress stats -->
					<div class="text-muted-foreground mt-1 flex justify-between text-xs">
						<!-- Bytes progress if available, otherwise items -->
						{#if modpackInstallStore.progress.totalBytes && modpackInstallStore.progress.totalBytes > 0}
							<span>
								{formatBytes(modpackInstallStore.progress.downloadedBytes || 0)} / {formatBytes(modpackInstallStore.progress.totalBytes)}
							</span>
						{:else if modpackInstallStore.progress.totalItems > 0}
							<span>
								{modpackInstallStore.progress.completedItems} / {modpackInstallStore.progress.totalItems} items
							</span>
						{:else}
							<span></span>
						{/if}

						<!-- Percentage -->
						<span class="font-mono">{Math.round(modpackInstallStore.progress.progress)}%</span>
					</div>
				{:else}
					<div class="text-muted-foreground mb-1 text-xs">Starting...</div>
					<div class="bg-muted h-2 overflow-hidden rounded-full">
						<div
							class="bg-primary h-full animate-pulse transition-all duration-150 ease-out"
							style="width: 10%"
						></div>
					</div>
				{/if}
			</div>

			<!-- Cancel button -->
			<Button
				variant="destructive"
				size="sm"
				onclick={handleCancel}
				disabled={modpackInstallStore.isCancelling}
			>
				<StopCircle class="mr-1 h-4 w-4" />
				Cancel
			</Button>
		</div>
	</div>
{/if}
