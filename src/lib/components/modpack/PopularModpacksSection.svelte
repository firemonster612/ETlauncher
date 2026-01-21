<script lang="ts">
	import { onDestroy } from 'svelte';
	import { Flame, ChevronLeft, ChevronRight, Package, Loader2 } from '@lucide/svelte';
	import type { Modpack, LoaderType } from '$lib/types';

	interface Props {
		modpacks: Modpack[];
		loading?: boolean;
		onModpackClick?: (modpack: Modpack) => void;
	}

	let { modpacks, loading = false, onModpackClick }: Props = $props();

	let currentIndex = $state(0);
	let currentGalleryIndex = $state(0);
	let isPaused = $state(false);
	let tickCount = $state(0);
	let intervalId: ReturnType<typeof setInterval> | null = null;

	// Timing: tick every 500ms, gallery changes every 4 ticks (2s), modpack switch after 6 ticks (3s)
	const TICK_MS = 500;
	const GALLERY_TICKS = 4; // 2 seconds
	const MODPACK_SWITCH_TICKS = 6; // 3 seconds on last image

	// Thumbnail modpacks are the next 4 (or remaining) modpacks after featured
	let thumbnailModpacks = $derived(() => {
		const thumbs: Modpack[] = [];
		for (let i = 1; i <= 4 && i < modpacks.length; i++) {
			const idx = (currentIndex + i) % modpacks.length;
			thumbs.push(modpacks[idx]);
		}
		return thumbs;
	});

	function startInterval() {
		if (intervalId) return;
		intervalId = setInterval(() => {
			if (isPaused || modpacks.length === 0) return;

			tickCount++;
			const gallery = modpacks[currentIndex]?.gallery || [];
			const isLastGalleryImage = gallery.length <= 1 || currentGalleryIndex >= gallery.length - 1;
			const ticksNeeded = isLastGalleryImage ? MODPACK_SWITCH_TICKS : GALLERY_TICKS;

			if (tickCount >= ticksNeeded) {
				tickCount = 0;
				if (gallery.length > 1 && currentGalleryIndex < gallery.length - 1) {
					currentGalleryIndex++;
				} else {
					currentIndex = (currentIndex + 1) % modpacks.length;
					currentGalleryIndex = 0;
				}
			}
		}, TICK_MS);
	}

	function stopInterval() {
		if (intervalId) {
			clearInterval(intervalId);
			intervalId = null;
		}
	}

	function handleMouseEnter() {
		isPaused = true;
	}

	function handleMouseLeave() {
		isPaused = false;
	}

	// Start interval when modpacks are loaded
	$effect(() => {
		if (modpacks.length > 0 && !loading) {
			startInterval();
		}
		return () => stopInterval();
	});

	onDestroy(() => {
		stopInterval();
	});

	function prev() {
		if (modpacks.length === 0) return;
		currentIndex = (currentIndex - 1 + modpacks.length) % modpacks.length;
		currentGalleryIndex = 0;
		tickCount = 0;
	}

	function next() {
		if (modpacks.length === 0) return;
		currentIndex = (currentIndex + 1) % modpacks.length;
		currentGalleryIndex = 0;
		tickCount = 0;
	}

	function goTo(index: number) {
		currentIndex = index;
		currentGalleryIndex = 0;
		tickCount = 0;
	}

	function getLoaderColor(loader: LoaderType): string {
		switch (loader) {
			case 'fabric':
				return 'bg-amber-500/20 text-amber-400';
			case 'forge':
				return 'bg-orange-500/20 text-orange-400';
			case 'neoforge':
				return 'bg-red-500/20 text-red-400';
			case 'quilt':
				return 'bg-purple-500/20 text-purple-400';
			default:
				return 'bg-muted/50 text-muted-foreground';
		}
	}

	function getCategoryColor(category: string): string {
		const colors: Record<string, string> = {
			adventure: 'bg-emerald-500/20 text-emerald-400',
			technology: 'bg-blue-500/20 text-blue-400',
			tech: 'bg-blue-500/20 text-blue-400',
			magic: 'bg-purple-500/20 text-purple-400',
			quests: 'bg-yellow-500/20 text-yellow-400',
			challenging: 'bg-red-500/20 text-red-400',
			hardcore: 'bg-red-500/20 text-red-400',
			lightweight: 'bg-cyan-500/20 text-cyan-400',
			optimization: 'bg-cyan-500/20 text-cyan-400',
			combat: 'bg-orange-500/20 text-orange-400',
			multiplayer: 'bg-indigo-500/20 text-indigo-400',
			'kitchen-sink': 'bg-pink-500/20 text-pink-400',
		};
		const key = category.toLowerCase().replace(/[^a-z-]/g, '');
		return colors[key] || 'bg-muted/50 text-muted-foreground';
	}
</script>

<section>
	<!-- Section Header -->
	<div class="section-header">
		<Flame class="section-header-icon h-6 w-6" />
		<h2 class="section-header-title">Popular Modpacks</h2>
	</div>

	{#if loading}
		<div class="border-border bg-card flex h-64 items-center justify-center border-2">
			<Loader2 class="text-muted-foreground h-8 w-8 animate-spin" />
		</div>
	{:else if modpacks.length === 0}
		<div
			class="border-border bg-card/50 flex h-64 items-center justify-center border-2 border-dashed"
		>
			<div class="text-muted-foreground text-center">
				<Package class="mx-auto h-12 w-12 opacity-50" />
				<p class="mt-2 text-sm">No popular modpacks available</p>
			</div>
		</div>
	{:else}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="popular-section-grid"
			onmouseenter={handleMouseEnter}
			onmouseleave={handleMouseLeave}
		>
			<!-- Left: Featured Banner Carousel -->
			<div class="featured-carousel">
				{#each modpacks as modpack, idx (`${modpack.platform}-${modpack.id}`)}
					<button
						type="button"
						class="featured-carousel-slide {idx === currentIndex ? 'active' : ''}"
						onclick={() => onModpackClick?.(modpack)}
					>
						{#if modpack.gallery && modpack.gallery.length > 0 && modpack.iconUrl}
							<!-- Has gallery + icon: show screenshot with icon overlay -->
							{@const galleryIdx = idx === currentIndex ? currentGalleryIndex : 0}
							{@const galleryImage = modpack.gallery[galleryIdx] || modpack.gallery[0]}
							<div class="relative h-full w-full">
								<img
									src={galleryImage?.rawUrl || galleryImage?.url}
									alt={modpack.name}
									class="h-full w-full object-cover transition-opacity duration-500"
								/>
								<!-- Icon overlay in top-left -->
								<div class="absolute top-3 left-3 z-10">
									<img
										src={modpack.iconUrl}
										alt=""
										class="h-14 w-14 border-2 border-black/30 object-cover shadow-2xl"
										style="will-change: transform; transform: translateZ(0);"
									/>
								</div>
								<!-- Gallery progress dots -->
								{#if modpack.gallery.length > 1 && idx === currentIndex}
									<div class="absolute top-3 right-3 z-10 flex gap-1">
										{#each Array.from({ length: modpack.gallery.length }, (_, i) => i) as gIdx (gIdx)}
											<div
												class="h-1.5 w-1.5 rounded-full transition-colors {gIdx ===
												currentGalleryIndex
													? 'bg-white'
													: 'bg-white/40'}"
											></div>
										{/each}
									</div>
								{/if}
							</div>
						{:else if modpack.bannerUrl}
							<img src={modpack.bannerUrl} alt={modpack.name} class="h-full w-full object-cover" />
						{:else if modpack.iconUrl}
							<div
								class="bg-card relative flex h-full w-full items-center justify-center overflow-hidden"
							>
								<!-- Softly blurred background icon -->
								<img
									src={modpack.iconUrl}
									alt=""
									class="absolute inset-0 h-full w-full object-cover opacity-60 blur-md"
									style="will-change: transform; transform: translateZ(0);"
								/>
								<!-- Centered sharp icon -->
								<img
									src={modpack.iconUrl}
									alt={modpack.name}
									class="relative z-10 mb-16 h-32 w-32 object-contain drop-shadow-lg"
									style="will-change: transform; transform: translateZ(0);"
								/>
							</div>
						{:else}
							<div class="bg-card flex h-full w-full items-center justify-center">
								<Package class="text-muted-foreground/50 h-32 w-32" />
							</div>
						{/if}
						<!-- Overlay with modpack info -->
						<div
							class="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/90 via-black/60 to-transparent p-4"
						>
							<h3 class="truncate text-lg font-bold text-white">{modpack.name}</h3>
							<p class="text-sm text-white/80">{modpack.author}</p>
						</div>
					</button>
				{/each}

				<!-- Navigation Arrows -->
				{#if modpacks.length > 1}
					<button
						type="button"
						class="featured-carousel-nav prev"
						onclick={prev}
						aria-label="Previous modpack"
					>
						<ChevronLeft class="h-5 w-5" />
					</button>
					<button
						type="button"
						class="featured-carousel-nav next"
						onclick={next}
						aria-label="Next modpack"
					>
						<ChevronRight class="h-5 w-5" />
					</button>
				{/if}

				<!-- Pagination Dots -->
				{#if modpacks.length > 1}
					<div class="featured-carousel-dots">
						{#each Array(modpacks.length)
							.fill(0)
							.map((__, i) => i) as idx (idx)}
							<button
								type="button"
								class="featured-carousel-dot"
								data-active={currentIndex === idx}
								onclick={() => goTo(idx)}
								aria-label="Go to modpack {idx + 1}"
							></button>
						{/each}
					</div>
				{/if}
			</div>

			<!-- Right: 2x2 Thumbnail Grid -->
			<div class="thumbnail-grid-2x2">
				{#each thumbnailModpacks() as thumb (`${thumb.platform}-${thumb.id}`)}
					<button
						type="button"
						class="border-border hover:border-primary/50 flex flex-col overflow-hidden border-2 text-left transition-colors"
						onclick={() => onModpackClick?.(thumb)}
					>
						<!-- Thumbnail image -->
						<div class="bg-muted aspect-video w-full overflow-hidden">
							{#if thumb.bannerUrl || (thumb.gallery && thumb.gallery[0])}
								<img
									src={thumb.bannerUrl || thumb.gallery?.[0]?.rawUrl || thumb.gallery?.[0]?.url}
									alt={thumb.name}
									class="h-full w-full object-cover"
									loading="lazy"
								/>
							{:else if thumb.iconUrl}
								<div
									class="bg-card relative flex h-full w-full items-center justify-center overflow-hidden"
								>
									<!-- Softly blurred background icon -->
									<img
										src={thumb.iconUrl}
										alt=""
										class="absolute inset-0 h-full w-full object-cover opacity-50 blur-sm"
										style="will-change: transform; transform: translateZ(0);"
									/>
									<!-- Centered sharp icon - larger -->
									<img
										src={thumb.iconUrl}
										alt={thumb.name}
										class="relative z-10 h-20 w-20 object-contain drop-shadow-lg"
										style="will-change: transform; transform: translateZ(0);"
									/>
								</div>
							{:else}
								<div class="bg-card flex h-full w-full items-center justify-center">
									<Package class="text-muted-foreground/50 h-16 w-16" />
								</div>
							{/if}
						</div>
						<!-- Info -->
						<div class="flex-1 space-y-1 p-2">
							<h4 class="truncate text-sm font-bold">{thumb.name}</h4>
							<div class="flex flex-wrap gap-1">
								{#each (thumb.loaders || [])
									.filter((l) => l && l !== 'unknown' && l !== 'vanilla')
									.slice(0, 1) as loader (loader)}
									<span class="rounded px-1 py-0.5 text-[10px] {getLoaderColor(loader)}">
										{loader}
									</span>
								{/each}
								{#each thumb.categories.slice(0, 2) as category (category)}
									<span class="rounded px-1 py-0.5 text-[10px] {getCategoryColor(category)}">
										{category}
									</span>
								{/each}
								{#if thumb.mcVersions.length > 0}
									<span class="bg-primary/20 text-primary rounded px-1 py-0.5 text-[10px]">
										{thumb.mcVersions[0]}
									</span>
								{/if}
							</div>
						</div>
					</button>
				{/each}
			</div>
		</div>
	{/if}
</section>
