<script lang="ts">
	import { updaterStore } from '$lib/stores/updater.svelte';
	import { Button } from '$lib/ui/button';
	import { Download, X, Loader2, ExternalLink } from '@lucide/svelte';
	import { openUrl } from '@tauri-apps/plugin-opener';

	const RELEASES_URL = 'https://github.com/firemonster612/ETlauncher/releases/latest';

	function dismiss() {
		updaterStore.dismissNotification();
	}

	function openReleasesPage() {
		openUrl(RELEASES_URL);
	}

	// Auto-dismiss after 4 seconds (only if not downloading and no error)
	$effect(() => {
		if (updaterStore.showNotification && !updaterStore.isDownloading && !updaterStore.error) {
			const timer = setTimeout(() => {
				dismiss();
			}, 4000);

			return () => clearTimeout(timer);
		}
	});
</script>

{#if updaterStore.showNotification || updaterStore.error}
	<div
		class="animate-in slide-in-from-bottom-4 fade-in fixed right-4 bottom-4 z-50 flex max-w-sm items-center gap-3 rounded-lg border-2 {updaterStore.error
			? 'border-destructive'
			: 'border-primary'} bg-card p-4 shadow-lg duration-300"
	>
		<div class="flex-1">
			{#if updaterStore.error}
				<p class="text-destructive text-sm font-medium">Update Failed</p>
				<p class="text-muted-foreground text-xs">{updaterStore.error}</p>
			{:else if !updaterStore.canAutoUpdate}
				<p class="text-sm font-medium">Update Available</p>
				<p class="text-muted-foreground text-xs">
					Version {updaterStore.latestVersion} — download manually to update
				</p>
			{:else}
				<p class="text-sm font-medium">Update Available</p>
				<p class="text-muted-foreground text-xs">
					Version {updaterStore.latestVersion} is ready
				</p>
			{/if}
		</div>

		<div class="flex items-center gap-2">
			{#if !updaterStore.error}
				{#if updaterStore.canAutoUpdate}
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
				{:else}
					<Button size="sm" onclick={openReleasesPage}>
						<ExternalLink class="mr-1.5 h-3.5 w-3.5" />
						Download
					</Button>
				{/if}
			{/if}
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
