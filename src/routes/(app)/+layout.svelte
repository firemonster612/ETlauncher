<script lang="ts">
	import { onMount } from 'svelte';
	import AppSidebar from '$lib/components/layout/AppSidebar.svelte';
	import ModpackInstallBar from '$lib/components/ModpackInstallBar.svelte';
	import LaunchingDialog from '$lib/components/LaunchingDialog.svelte';
	import UpdateNotification from '$lib/components/UpdateNotification.svelte';
	import { SidebarInset } from '$lib/ui/sidebar';
	import { launchStore } from '$lib/stores/launch.svelte';
	import { modpackInstallStore } from '$lib/stores/modpackInstall.svelte';
	import { instancesStore } from '$lib/stores/instances.svelte';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { updaterStore } from '$lib/stores/updater.svelte';

	let { children } = $props();

	// Initialize stores once at app level
	onMount(() => {
		launchStore.init();
		modpackInstallStore.init();
		instancesStore.init();

		// Check for updates on startup if auto-update is enabled
		const checkForUpdates = async () => {
			// Wait for settings to load
			await settingsStore.load();
			if (settingsStore.settings?.autoUpdate) {
				// Silent check - don't show errors for automatic checks
				updaterStore.checkForUpdates(true);
			}
		};
		checkForUpdates();

		return () => {
			launchStore.cleanup();
			instancesStore.cleanup();
		};
	});
</script>

<AppSidebar />
<SidebarInset class="overflow-hidden">
	<main
		class="flex-1 overflow-x-hidden overflow-y-auto p-6"
		class:pb-24={modpackInstallStore.isInstalling}
	>
		{@render children()}
	</main>

	<!-- Global modpack install progress bar -->
	<ModpackInstallBar />
</SidebarInset>

<!-- Global launching dialog -->
<LaunchingDialog />

<!-- Update notification toast -->
<UpdateNotification />
