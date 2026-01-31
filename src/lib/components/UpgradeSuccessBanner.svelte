<script lang="ts">
	import { onMount } from 'svelte';
	import { getVersion } from '@tauri-apps/api/app';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import { X, ExternalLink } from '@lucide/svelte';

	const LAST_VERSION_KEY = 'etlauncher_last_version';
	const RELEASES_URL = 'https://github.com/firemonster612/ETlauncher/releases/tag/v';

	let showBanner = $state(false);
	let currentVersion = $state('');

	function dismiss() {
		showBanner = false;
	}

	function openChangelog() {
		openUrl(`${RELEASES_URL}${currentVersion}`);
	}

	onMount(async () => {
		try {
			currentVersion = await getVersion();
			const lastVersion = localStorage.getItem(LAST_VERSION_KEY);

			// Show banner if version changed (upgraded)
			if (lastVersion && lastVersion !== currentVersion) {
				showBanner = true;

				// Auto-dismiss after 8 seconds
				setTimeout(() => {
					showBanner = false;
				}, 8000);
			}

			// Always save current version
			localStorage.setItem(LAST_VERSION_KEY, currentVersion);
		} catch (e) {
			console.error('Failed to check version:', e);
		}
	});
</script>

{#if showBanner}
	<div
		class="animate-in slide-in-from-top fade-in bg-green-600 text-white flex items-center justify-center gap-3 py-2 px-4 duration-300"
	>
		<span class="font-medium">
			Updated to v{currentVersion}
		</span>
		<button
			onclick={openChangelog}
			class="flex items-center gap-1 underline underline-offset-2 hover:no-underline text-sm"
		>
			<ExternalLink class="h-3.5 w-3.5" />
			View changelog
		</button>
		<button
			onclick={dismiss}
			class="ml-2 hover:bg-green-700 rounded p-0.5 transition-colors"
			aria-label="Dismiss"
		>
			<X class="h-4 w-4" />
		</button>
	</div>
{/if}
