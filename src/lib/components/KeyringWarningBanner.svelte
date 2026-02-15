<script lang="ts">
	import { AlertTriangle, X } from '@lucide/svelte';
	import { onMount } from 'svelte';
	import { isKeyringAvailable } from '$lib/services/auth';

	let showBanner = $state(false);
	let dismissed = $state(false);

	onMount(async () => {
		try {
			const available = await isKeyringAvailable();
			if (!available) {
				showBanner = true;
			}
		} catch (e) {
			console.error('Failed to check keyring availability:', e);
		}
	});

	function dismiss() {
		dismissed = true;
		showBanner = false;
	}
</script>

{#if showBanner && !dismissed}
	<div
		class="animate-in slide-in-from-top fade-in bg-amber-600 text-white flex items-center justify-center gap-3 py-2 px-4 duration-300"
	>
		<AlertTriangle class="h-4 w-4 shrink-0" />
		<span class="text-sm">
			No secure keyring found. Your login tokens are stored insecurely in a plaintext file.
			Install a Secret Service provider (e.g. <code class="bg-amber-700 rounded px-1">gnome-keyring</code>) for secure storage.
		</span>
		<button
			onclick={dismiss}
			class="ml-2 hover:bg-amber-700 rounded p-0.5 transition-colors shrink-0"
			aria-label="Dismiss"
		>
			<X class="h-4 w-4" />
		</button>
	</div>
{/if}
