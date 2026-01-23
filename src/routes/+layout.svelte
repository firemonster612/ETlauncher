<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { SidebarProvider } from '$lib/ui/sidebar';
	import Titlebar from '$lib/components/layout/Titlebar.svelte';
	import TutorialOverlay from '$lib/components/TutorialOverlay.svelte';
	import TutorialWelcome from '$lib/components/TutorialWelcome.svelte';
	import AlertDialog from '$lib/components/AlertDialog.svelte';
	import { tutorialStore } from '$lib/stores/tutorial.svelte';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { accountsStore } from '$lib/stores/accounts.svelte';
	import { themeStore } from '$lib/stores/theme.svelte';

	let { children } = $props();

	onMount(async () => {
		// Initialize theme system
		themeStore.init();

		// Provide navigation function to tutorial store
		// eslint-disable-next-line svelte/no-navigation-without-resolve
		tutorialStore.setNavigate((path: string) => goto(path));

		// Load settings and accounts in parallel
		await Promise.all([settingsStore.load(), accountsStore.load()]);

		// Apply theme from loaded settings
		themeStore.applyFromSettings();

		if (settingsStore.settings && !settingsStore.settings.setupCompleted) {
			tutorialStore.showWelcomeModal();
		}
	});

	// Re-apply theme when settings change
	$effect(() => {
		if (settingsStore.settings) {
			themeStore.applyFromSettings();
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

<!-- Global Alert Dialog -->
<AlertDialog />
