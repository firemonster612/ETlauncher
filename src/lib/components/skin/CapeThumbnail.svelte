<script lang="ts">
	import { onMount } from 'svelte';

	interface Props {
		url: string;
		alt: string;
		class?: string;
	}

	let { url, alt, class: className = '' }: Props = $props();

	let canvasEl: HTMLCanvasElement;
	let renderedUrl = $state<string | null>(null);

	onMount(() => {
		renderCapeFront();
	});

	// Re-render when URL changes
	$effect(() => {
		if (url) {
			renderCapeFront();
		}
	});

	async function renderCapeFront() {
		const img = new Image();
		img.crossOrigin = 'anonymous';

		img.onload = () => {
			const canvas = canvasEl;
			if (!canvas) return;

			const ctx = canvas.getContext('2d');
			if (!ctx) return;

			// Cape texture dimensions vary:
			// - Standard: 64x32 (front at 1,1 size 10x16)
			// - Elytra/new format: 64x64 (front at 1,1 size 10x16)
			// The front of the cape is always at (1, 1) with size 10x16

			// Set canvas to the cropped cape size (scaled up for visibility)
			const scale = 4;
			const capeWidth = 10;
			const capeHeight = 16;
			canvas.width = capeWidth * scale;
			canvas.height = capeHeight * scale;

			// Disable image smoothing for pixelated look
			ctx.imageSmoothingEnabled = false;

			// Draw just the front of the cape, scaled up
			// Source: x=1, y=1, width=10, height=16
			// Destination: full canvas
			ctx.drawImage(
				img,
				1, 1, // source x, y
				capeWidth, capeHeight, // source width, height
				0, 0, // dest x, y
				canvas.width, canvas.height // dest width, height
			);

			renderedUrl = canvas.toDataURL();
		};

		img.onerror = () => {
			// Fallback to original URL if CORS fails
			renderedUrl = url;
		};

		img.src = url;
	}
</script>

<canvas bind:this={canvasEl} class="hidden"></canvas>

{#if renderedUrl}
	<img
		src={renderedUrl}
		{alt}
		class="pixelated {className}"
	/>
{:else}
	<div class="bg-muted animate-pulse {className}"></div>
{/if}
