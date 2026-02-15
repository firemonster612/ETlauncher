<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import { onMount } from 'svelte';
	import KeyringWarningBanner from '$lib/components/KeyringWarningBanner.svelte';
	import LaunchingDialog from '$lib/components/LaunchingDialog.svelte';
	import AppSidebar from '$lib/components/layout/AppSidebar.svelte';
	import SidebarTasks from '$lib/components/layout/SidebarTasks.svelte';
	import UpdateNotification from '$lib/components/UpdateNotification.svelte';
	import UpgradeSuccessBanner from '$lib/components/UpgradeSuccessBanner.svelte';
	import { instancesStore } from '$lib/stores/instances.svelte';
	import { launchStore } from '$lib/stores/launch.svelte';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { taskManagerStore } from '$lib/stores/taskManager.svelte';
	import { updaterStore } from '$lib/stores/updater.svelte';
	import type { Instance } from '$lib/types';
	import { SidebarInset } from '$lib/ui/sidebar';

	let { children } = $props();

	// Initialize stores once at app level
	onMount(() => {
		launchStore.init();
		instancesStore.init();
		taskManagerStore.init();

		// Listen for modpack install completion to refresh instances
		let unlistenComplete: (() => void) | null = null;
		listen<Instance>('modpack_install_complete', () => {
			instancesStore.load();
		}).then((unlisten) => {
			unlistenComplete = unlisten;
		});

		// Listen for instance update completion to refresh instances
		let unlistenUpdateComplete: (() => void) | null = null;
		listen<Instance>('instance_update_complete', () => {
			instancesStore.load();
		}).then((unlisten) => {
			unlistenUpdateComplete = unlisten;
		});

		// Check for updates on startup if auto-update is enabled
		const checkForUpdates = async () => {
			// Wait for settings to load
			await settingsStore.load();
			if (settingsStore.settings?.autoUpdate) {
				// Silent check - don't show errors for automatic checks, but show notification
				updaterStore.checkForUpdates({ silent: true, showNotification: true });
			}
		};
		checkForUpdates();

		return () => {
			launchStore.cleanup();
			instancesStore.cleanup();
			taskManagerStore.cleanup();
			unlistenComplete?.();
			unlistenUpdateComplete?.();
		};
	});
</script>

<AppSidebar />
<SidebarInset class="overflow-hidden">
	<KeyringWarningBanner />
	<UpgradeSuccessBanner />
	<main class="flex-1 overflow-x-hidden overflow-y-auto p-6">
		{@render children()}
	</main>

	<!-- Task progress bar + expanding drawer -->
	<SidebarTasks />
</SidebarInset>

<!-- Global launching dialog -->
<LaunchingDialog />

<!-- Update notification toast -->
<UpdateNotification />
