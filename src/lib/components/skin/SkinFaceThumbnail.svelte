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
		renderSkinFace();
	});

	// Re-render when URL changes
	$effect(() => {
		if (url) {
			renderSkinFace();
		}
	});

	async function renderSkinFace() {
		const img = new Image();
		img.crossOrigin = 'anonymous';

		img.onload = () => {
			const canvas = canvasEl;
			if (!canvas) return;

			const ctx = canvas.getContext('2d');
			if (!ctx) return;

			// Skin texture: 64x64 (or 64x32 for old format)
			// Face is at (8, 8) with size 8x8
			// Hat overlay is at (40, 8) with size 8x8

			const scale = 8;
			const faceSize = 8;
			canvas.width = faceSize * scale;
			canvas.height = faceSize * scale;

			// Disable image smoothing for pixelated look
			ctx.imageSmoothingEnabled = false;

			// Draw the face (base layer)
			ctx.drawImage(
				img,
				8, 8, // source x, y (face location)
				faceSize, faceSize, // source width, height
				0, 0, // dest x, y
				canvas.width, canvas.height // dest width, height
			);

			// Draw the hat overlay (if exists - only in 64x64 textures)
			if (img.height >= 64) {
				ctx.drawImage(
					img,
					40, 8, // source x, y (hat overlay location)
					faceSize, faceSize, // source width, height
					0, 0, // dest x, y
					canvas.width, canvas.height // dest width, height
				);
			}

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
