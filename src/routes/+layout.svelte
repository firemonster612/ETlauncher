<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { SidebarProvider } from '$lib/ui/sidebar';
	import Titlebar from '$lib/components/layout/Titlebar.svelte';
	import OnboardingWizard from '$lib/components/OnboardingWizard.svelte';
	import AlertDialog from '$lib/components/AlertDialog.svelte';
	import { onboardingStore } from '$lib/stores/onboarding.svelte';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { accountsStore } from '$lib/stores/accounts.svelte';
	import { themeStore } from '$lib/stores/theme.svelte';

	let { children } = $props();

	onMount(async () => {
		// Initialize theme system
		themeStore.init();

		// Load settings and accounts in parallel
		await Promise.all([settingsStore.load(), accountsStore.load()]);

		// Apply theme from loaded settings
		themeStore.applyFromSettings();

		// Show onboarding wizard if setup not completed
		if (settingsStore.settings && !settingsStore.settings.setupCompleted) {
			onboardingStore.start();
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

<!-- Onboarding Wizard -->
<OnboardingWizard />

<!-- Global Alert Dialog -->
<AlertDialog />
