<script lang="ts">
	import { ImageOff } from '@lucide/svelte';
	import type { ContentGalleryImage } from '$lib/types/content';

	interface Props {
		images: ContentGalleryImage[];
		onImageClick?: (index: number) => void;
	}

	let { images, onImageClick }: Props = $props();
</script>

<div class="gallery-sidebar">
	{#if images.length === 0}
		<div
			class="border-border bg-muted/30 flex aspect-video items-center justify-center border-2 border-dashed"
		>
			<div class="text-muted-foreground flex flex-col items-center gap-2 text-center">
				<ImageOff class="h-8 w-8 opacity-50" />
				<span class="text-sm">No gallery images</span>
			</div>
		</div>
	{:else}
		{#each images as image, index (image.rawUrl ?? image.url)}
			<button
				type="button"
				class="gallery-sidebar-item"
				onclick={() => onImageClick?.(index)}
				aria-label={image.title ?? `Gallery image ${index + 1}`}
			>
				<img
					src={image.rawUrl ?? image.url}
					alt={image.title ?? `Gallery image ${index + 1}`}
					loading="lazy"
				/>
			</button>
		{/each}
	{/if}
</div>
