<script lang="ts">
	import { updaterStore } from '$lib/stores/updater.svelte';
	import { Button } from '$lib/ui/button';
	import { Download, X, Loader2 } from '@lucide/svelte';

	function dismiss() {
		updaterStore.clearUpdate();
	}
</script>

{#if updaterStore.updateAvailable}
	<div
		class="animate-in slide-in-from-bottom-4 fade-in fixed right-4 bottom-4 z-50 flex items-center gap-3 rounded-lg border-2 border-primary bg-card p-4 shadow-lg duration-300"
	>
		<div class="flex-1">
			<p class="text-sm font-medium">Update Available</p>
			<p class="text-muted-foreground text-xs">
				Version {updaterStore.latestVersion} is ready
			</p>
		</div>

		<div class="flex items-center gap-2">
			<Button
				size="sm"
				onclick={() => updaterStore.downloadAndInstall()}
				disabled={updaterStore.isDownloading}
			>
				{#if updaterStore.isDownloading}
					<Loader2 class="mr-1.5 h-3.5 w-3.5 animate-spin" />
					{Math.round(updaterStore.downloadProgress)}%
				{:else}
					<Download class="mr-1.5 h-3.5 w-3.5" />
					Update
				{/if}
			</Button>
			<button
				onclick={dismiss}
				class="text-muted-foreground hover:text-foreground transition-colors"
				aria-label="Dismiss"
			>
				<X class="h-4 w-4" />
			</button>
		</div>
	</div>
{/if}
