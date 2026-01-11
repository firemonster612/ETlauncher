<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { SidebarProvider } from '$lib/ui/sidebar';
	import Titlebar from '$lib/components/layout/Titlebar.svelte';
	import TutorialOverlay from '$lib/components/TutorialOverlay.svelte';
	import TutorialWelcome from '$lib/components/TutorialWelcome.svelte';
	import { tutorialStore } from '$lib/stores/tutorial.svelte';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { accountsStore } from '$lib/stores/accounts.svelte';

	let { children } = $props();

	onMount(async () => {
		// Provide navigation function to tutorial store
		// eslint-disable-next-line svelte/no-navigation-without-resolve
		tutorialStore.setNavigate((path: string) => goto(path));

		// Load settings and accounts in parallel
		await Promise.all([settingsStore.load(), accountsStore.load()]);
		if (settingsStore.settings && !settingsStore.settings.setupCompleted) {
			tutorialStore.showWelcomeModal();
		}
	});
</script>

<div class="flex h-screen flex-col overflow-hidden">
	<Titlebar />
	<SidebarProvider>
		<div class="flex flex-1 overflow-hidden">
			{@render children()}
		</div>
	</SidebarProvider>
</div>

<!-- Tutorial System -->
<TutorialWelcome open={tutorialStore.showWelcome} />
<TutorialOverlay />
