<script lang="ts">
	import { Camera, ChevronLeft, ChevronRight } from '@lucide/svelte';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import ScreenshotLightbox from '$lib/components/ScreenshotLightbox.svelte';
	import * as instanceDetailService from '$lib/services/instance-detail';
	import type { HomepageScreenshot } from '$lib/types';

	interface Props {
		screenshots: HomepageScreenshot[];
	}

	let { screenshots }: Props = $props();

	let scrollContainer = $state<HTMLDivElement | null>(null);
	let canScrollLeft = $state(false);
	let canScrollRight = $state(false);

	// Lightbox state
	let lightboxIndex = $state<number | null>(null);
	let lightboxData = $state<string | null>(null);
	let lightboxLoading = $state(false);
	let previewSources = $state<Record<string, string>>({});

	const dateFormatter = new Intl.DateTimeFormat(undefined, {
		month: 'short',
		day: 'numeric',
	});

	function formatDate(timestamp: number): string {
		return dateFormatter.format(timestamp);
	}

	function updateScrollButtons() {
		if (!scrollContainer) return;
		canScrollLeft = scrollContainer.scrollLeft > 0;
		canScrollRight =
			scrollContainer.scrollLeft < scrollContainer.scrollWidth - scrollContainer.clientWidth - 1;
	}

	function scrollLeft() {
		scrollContainer?.scrollBy({ left: -300, behavior: 'smooth' });
	}

	function scrollRight() {
		scrollContainer?.scrollBy({ left: 300, behavior: 'smooth' });
	}

	$effect(() => {
		if (scrollContainer) {
			updateScrollButtons();
			scrollContainer.addEventListener('scroll', updateScrollButtons);
			window.addEventListener('resize', updateScrollButtons);
			return () => {
				scrollContainer?.removeEventListener('scroll', updateScrollButtons);
				window.removeEventListener('resize', updateScrollButtons);
			};
		}
	});

	async function openLightbox(index: number) {
		lightboxIndex = index;
		await loadLightboxImage(index);
	}

	async function loadLightboxImage(index: number) {
		const shot = screenshots[index];
		if (!shot) return;

		lightboxLoading = true;
		try {
			const data = await instanceDetailService.getScreenshotData(shot.instanceId, shot.filename);
			const dataUrl = `data:image/png;base64,${data}`;
			lightboxData = dataUrl;
			previewSources = { ...previewSources, [`${shot.instanceId}:${shot.filename}`]: dataUrl };
		} catch (e) {
			console.error('Failed to load screenshot data:', e);
		} finally {
			lightboxLoading = false;
		}
	}

	function closeLightbox() {
		lightboxIndex = null;
		lightboxData = null;
	}

	async function loadPreview(shot: HomepageScreenshot) {
		const key = `${shot.instanceId}:${shot.filename}`;
		if (previewSources[key]) return;
		try {
			const data = await instanceDetailService.getScreenshotData(shot.instanceId, shot.filename);
			previewSources = {
				...previewSources,
				[key]: `data:image/png;base64,${data}`,
			};
		} catch (e) {
			console.error('Failed to load screenshot preview', e);
		}
	}

	function goPrev() {
		if (lightboxIndex === null || lightboxIndex === 0) return;
		const nextIndex = lightboxIndex - 1;
		lightboxIndex = nextIndex;
		loadLightboxImage(nextIndex);
	}

	function goNext() {
		if (lightboxIndex === null) return;
		const nextIndex = lightboxIndex + 1;
		if (nextIndex >= screenshots.length) return;
		lightboxIndex = nextIndex;
		loadLightboxImage(nextIndex);
	}

	const canPrev = $derived(lightboxIndex !== null && lightboxIndex > 0);
	const canNext = $derived(lightboxIndex !== null && lightboxIndex < screenshots.length - 1);
</script>

{#if screenshots.length > 0}
	<div class="space-y-2">
		<!-- Header -->
		<div class="flex items-center justify-between">
			<div class="flex items-center gap-2">
				<Camera class="text-primary h-4 w-4" />
				<h2 class="text-sm font-bold uppercase tracking-wider">Recent Screenshots</h2>
			</div>
			<!-- Scroll buttons -->
			<div class="flex items-center gap-1">
				<button
					type="button"
					class="bg-muted hover:bg-muted/80 disabled:opacity-30 rounded p-1 transition-colors disabled:cursor-not-allowed"
					onclick={scrollLeft}
					disabled={!canScrollLeft}
					aria-label="Scroll left"
				>
					<ChevronLeft class="h-4 w-4" />
				</button>
				<button
					type="button"
					class="bg-muted hover:bg-muted/80 disabled:opacity-30 rounded p-1 transition-colors disabled:cursor-not-allowed"
					onclick={scrollRight}
					disabled={!canScrollRight}
					aria-label="Scroll right"
				>
					<ChevronRight class="h-4 w-4" />
				</button>
			</div>
		</div>

		<!-- Horizontal scrolling container -->
		<div
			bind:this={scrollContainer}
			class="scrollbar-thin scrollbar-thumb-muted scrollbar-track-transparent -mx-2 flex gap-3 overflow-x-auto px-2 pb-2"
		>
			{#each screenshots as shot, index (shot.path)}
				{@const key = `${shot.instanceId}:${shot.filename}`}
				<button
					type="button"
					class="border-border bg-muted/30 hover:border-primary/60 group relative aspect-video h-44 flex-shrink-0 overflow-hidden border-2 transition-all"
					onclick={() => openLightbox(index)}
				>
					<img
						src={previewSources[key] ?? convertFileSrc(shot.path)}
						alt={shot.filename}
						class="h-full w-full object-cover transition-transform group-hover:scale-[1.02]"
						loading="lazy"
						onerror={() => loadPreview(shot)}
					/>
					<div
						class="absolute right-0 bottom-0 left-0 bg-gradient-to-t from-black/70 to-transparent px-2 py-1"
					>
						<p class="truncate text-[10px] text-white/90">{shot.instanceName}</p>
						<p class="text-[10px] text-white/60">{formatDate(shot.takenAt)}</p>
					</div>
				</button>
			{/each}
		</div>
	</div>
{/if}

<ScreenshotLightbox
	open={lightboxIndex !== null}
	src={lightboxData}
	filename={lightboxIndex !== null ? screenshots[lightboxIndex]?.filename : undefined}
	isLoading={lightboxLoading}
	{canPrev}
	{canNext}
	onClose={closeLightbox}
	onPrev={goPrev}
	onNext={goNext}
/>
