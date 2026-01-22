<script lang="ts">
	import { Loader2, StopCircle } from '@lucide/svelte';
	import { modpackInstallStore } from '$lib/stores/modpackInstall.svelte';
	import { alertDialogStore } from '$lib/stores/alertDialog.svelte';
	import { Button } from '$lib/ui/button';

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
</script>

{#if modpackInstallStore.isInstalling}
	<div class="bg-card border-border fixed right-0 bottom-0 left-0 z-40 border-t-2 p-4 shadow-lg">
		<div class="mx-auto flex max-w-4xl items-center gap-4">
			<!-- Progress info -->
			<div class="min-w-0 flex-1">
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
					<div class="text-muted-foreground mb-1 truncate text-xs">
						{modpackInstallStore.progress.stage}
						{#if modpackInstallStore.progress.currentItem}
							- {modpackInstallStore.progress.currentItem}
						{/if}
					</div>

					<div class="bg-muted h-2 overflow-hidden rounded-full">
						<div
							class="bg-primary h-full transition-all duration-150 ease-out"
							style="width: {Math.min(100, Math.max(0, modpackInstallStore.progress.progress))}%"
						></div>
					</div>

					{#if modpackInstallStore.progress.totalItems > 0}
						<div class="text-muted-foreground mt-1 text-xs">
							{modpackInstallStore.progress.completedItems} / {modpackInstallStore.progress
								.totalItems} items
						</div>
					{/if}
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
