<script lang="ts">
	import { onMount } from 'svelte';
	import AppSidebar from '$lib/components/layout/AppSidebar.svelte';
	import ModpackInstallBar from '$lib/components/ModpackInstallBar.svelte';
	import { SidebarInset } from '$lib/ui/sidebar';
	import { launchStore } from '$lib/stores/launch.svelte';
	import { modpackInstallStore } from '$lib/stores/modpackInstall.svelte';

	let { children } = $props();

	// Initialize stores once at app level
	onMount(() => {
		launchStore.init();
		modpackInstallStore.init();
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
