<script lang="ts">
	import { ChevronLeft, ChevronRight } from '@lucide/svelte';
	import type { ContentGalleryImage } from '$lib/types/content';

	interface Props {
		images: ContentGalleryImage[];
		onImageClick?: (index: number) => void;
	}

	let { images, onImageClick }: Props = $props();

	let currentIndex = $state(0);

	function prev() {
		currentIndex = Math.max(0, currentIndex - 1);
	}

	function next() {
		currentIndex = Math.min(images.length - 1, currentIndex + 1);
	}

	function goTo(index: number) {
		currentIndex = index;
	}

	function handleImageClick() {
		onImageClick?.(currentIndex);
	}

	// Auto-advance timer for hover carousel effect
	let autoAdvanceTimer: ReturnType<typeof setInterval> | null = null;

	function startAutoAdvance() {
		if (images.length <= 1) return;
		autoAdvanceTimer = setInterval(() => {
			currentIndex = (currentIndex + 1) % images.length;
		}, 3000);
	}

	function stopAutoAdvance() {
		if (autoAdvanceTimer) {
			clearInterval(autoAdvanceTimer);
			autoAdvanceTimer = null;
		}
	}
</script>

{#if images.length > 0}
	<div
		class="gallery-carousel border-border border-2"
		role="region"
		aria-label="Image gallery"
		onmouseenter={startAutoAdvance}
		onmouseleave={stopAutoAdvance}
	>
		<!-- Main Image Area -->
		<button
			type="button"
			class="relative aspect-video w-full cursor-pointer"
			onclick={handleImageClick}
		>
			<div
				class="gallery-carousel-track h-full"
				style="transform: translateX(-{currentIndex * 100}%)"
			>
				{#each images as image, idx (image.rawUrl ?? image.url)}
					<div class="gallery-carousel-slide h-full">
						<img
							src={image.rawUrl ?? image.url}
							alt={image.title ?? `Gallery image ${idx + 1}`}
							class="h-full w-full object-cover"
							loading={idx === 0 ? 'eager' : 'lazy'}
						/>
					</div>
				{/each}
			</div>

			<!-- Gradient overlay with title -->
			{#if images[currentIndex]?.title || images[currentIndex]?.description}
				{@const currentImage = images[currentIndex]}
				<div
					class="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/80 via-black/40 to-transparent p-3"
				>
					{#if currentImage.title}
						<div class="truncate text-sm font-semibold text-white">{currentImage.title}</div>
					{/if}
					{#if currentImage.description}
						<p class="line-clamp-2 text-xs text-white/80">{currentImage.description}</p>
					{/if}
				</div>
			{/if}
		</button>

		<!-- Navigation Arrows -->
		{#if images.length > 1}
			<button
				type="button"
				class="gallery-carousel-nav prev"
				onclick={prev}
				disabled={currentIndex === 0}
				aria-label="Previous image"
			>
				<ChevronLeft class="h-5 w-5" />
			</button>
			<button
				type="button"
				class="gallery-carousel-nav next"
				onclick={next}
				disabled={currentIndex === images.length - 1}
				aria-label="Next image"
			>
				<ChevronRight class="h-5 w-5" />
			</button>

			<!-- Dots -->
			<div class="gallery-carousel-dots">
				<!-- eslint-disable-next-line @typescript-eslint/no-unused-vars -->
				{#each images as _image, idx (idx)}
					<button
						type="button"
						class="gallery-carousel-dot"
						data-active={currentIndex === idx}
						onclick={() => goTo(idx)}
						aria-label="Go to image {idx + 1}"
					></button>
				{/each}
			</div>
		{/if}
	</div>
{/if}
