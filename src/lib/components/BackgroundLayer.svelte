<script lang="ts">
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { themeStore } from '$lib/stores/theme.svelte';

	interface Props {
		/** Use absolute positioning instead of fixed (for rendering inside modals) */
		absolute?: boolean;
	}

	let { absolute = false }: Props = $props();

	const bgConfig = $derived(settingsStore.settings?.background);
	const bgType = $derived(bgConfig?.type ?? 'none');
	const bgUrl = $derived(themeStore.backgroundUrl);
	// Use blur value directly (not CSS variable) to ensure inset and blur are always in sync
	const blur = $derived(bgConfig?.blur ?? 0);
	// Inset needs to be larger than blur to prevent white edges showing
	const inset = $derived(Math.max(50, blur + 50));

	// Track if image/video is loaded to prevent flash during loading
	let mediaLoaded = $state(false);

	// Reset loaded state when URL changes
	$effect(() => {
		if (bgUrl) {
			mediaLoaded = false;
		}
	});

	function handleMediaLoad() {
		mediaLoaded = true;
	}
</script>

{#if bgType !== 'none'}
	<div class="{absolute ? 'absolute' : 'fixed'} inset-0 z-0 pointer-events-none" style="overflow: clip;">
		{#if bgType === 'color'}
			<div
				class="absolute"
				style:inset="-{inset}px"
				style:background-color="var(--app-background-color)"
				style:filter="blur({blur}px)"
			/>
		{:else if bgType === 'image' || bgType === 'gif'}
			{#if bgUrl}
				<!-- Hidden img to detect when image is loaded -->
				<img
					src={bgUrl}
					alt=""
					onload={handleMediaLoad}
					class="hidden"
				/>
				{#if mediaLoaded}
					<div
						class="absolute"
						style:inset="-{inset}px"
						style:background-image="url({bgUrl})"
						style:background-size="cover"
						style:background-position="center"
						style:filter="blur({blur}px)"
					></div>
				{/if}
			{/if}
		{:else if bgType === 'video'}
			{#if bgUrl}
				{#key bgUrl}
					<!-- svelte-ignore a11y_media_has_caption -->
					<video
						src={bgUrl}
						autoplay
						loop
						muted
						playsinline
						onloadeddata={handleMediaLoad}
						class="absolute"
						style:inset="-{inset}px"
						style:width="calc(100% + {inset * 2}px)"
						style:height="calc(100% + {inset * 2}px)"
						style:object-fit="cover"
						style:filter="blur({blur}px)"
						style:opacity={mediaLoaded ? 1 : 0}
					></video>
				{/key}
			{/if}
		{/if}
	</div>
{/if}
