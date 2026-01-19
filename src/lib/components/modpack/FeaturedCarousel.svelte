<script lang="ts">
	import { ChevronLeft, ChevronRight, Download, TrendingUp } from '@lucide/svelte';
	import { onMount, onDestroy } from 'svelte';
	import type { Modpack, ModpackPlatform } from '$lib/types';

	interface Props {
		modpacks: Modpack[];
		onModpackClick: (modpack: Modpack) => void;
	}

	let { modpacks, onModpackClick }: Props = $props();

	let currentIndex = $state(0);
	let autoPlayInterval: ReturnType<typeof setInterval> | null = null;
	let isPaused = $state(false);

	// Auto-rotate every 5 seconds
	function startAutoPlay() {
		if (autoPlayInterval) clearInterval(autoPlayInterval);
		autoPlayInterval = setInterval(() => {
			if (!isPaused && modpacks.length > 1) {
				currentIndex = (currentIndex + 1) % modpacks.length;
			}
		}, 5000);
	}

	function stopAutoPlay() {
		if (autoPlayInterval) {
			clearInterval(autoPlayInterval);
			autoPlayInterval = null;
		}
	}

	onMount(() => {
		startAutoPlay();
	});

	onDestroy(() => {
		stopAutoPlay();
	});

	function goToSlide(index: number) {
		currentIndex = index;
		// Reset auto-play timer when manually navigating
		startAutoPlay();
	}

	function goToPrev() {
		currentIndex = currentIndex === 0 ? modpacks.length - 1 : currentIndex - 1;
		startAutoPlay();
	}

	function goToNext() {
		currentIndex = (currentIndex + 1) % modpacks.length;
		startAutoPlay();
	}

	function handleMouseEnter() {
		isPaused = true;
	}

	function handleMouseLeave() {
		isPaused = false;
	}

	function formatDownloads(downloads: number): string {
		if (downloads >= 1_000_000) {
			return `${(downloads / 1_000_000).toFixed(1)}M`;
		}
		if (downloads >= 1_000) {
			return `${(downloads / 1_000).toFixed(1)}K`;
		}
		return downloads.toString();
	}

	function getPlatformColor(platform: ModpackPlatform): string {
		switch (platform) {
			case 'modrinth':
				return 'bg-green-500/90 text-white';
			case 'curseforge':
				return 'bg-orange-500/90 text-white';
			case 'ftb':
				return 'bg-blue-500/90 text-white';
			case 'technic':
				return 'bg-yellow-500/90 text-black';
			case 'atlauncher':
				return 'bg-purple-500/90 text-white';
			default:
				return 'bg-muted text-muted-foreground';
		}
	}

	function getPlatformLabel(platform: ModpackPlatform): string {
		switch (platform) {
			case 'modrinth':
				return 'Modrinth';
			case 'curseforge':
				return 'CurseForge';
			case 'ftb':
				return 'FTB';
			case 'technic':
				return 'Technic';
			case 'atlauncher':
				return 'ATLauncher';
			default:
				return platform;
		}
	}

	// Get banner or first gallery image
	function getBannerImage(modpack: Modpack): string | null {
		return modpack.bannerUrl || modpack.gallery?.[0]?.rawUrl || modpack.gallery?.[0]?.url || null;
	}
</script>

{#if modpacks.length > 0}
	<div class="space-y-2">
		<!-- Header -->
		<div class="flex items-center gap-2">
			<TrendingUp class="text-primary h-4 w-4" />
			<h2 class="text-sm font-bold tracking-wider uppercase">Trending</h2>
		</div>

		<!-- Carousel -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="featured-carousel border-border overflow-hidden border-2"
			onmouseenter={handleMouseEnter}
			onmouseleave={handleMouseLeave}
		>
			<!-- Track -->
			<div class="featured-carousel-track" style="transform: translateX(-{currentIndex * 100}%)">
				{#each modpacks as modpack, idx (modpack.id)}
					<button
						type="button"
						class="featured-carousel-slide aspect-[21/9] w-full flex-shrink-0 cursor-pointer"
						onclick={() => onModpackClick(modpack)}
					>
						<!-- Background image -->
						<div class="absolute inset-0">
							{#if getBannerImage(modpack)}
								<img
									src={getBannerImage(modpack)}
									alt=""
									class="h-full w-full object-cover"
									loading={idx === 0 ? 'eager' : 'lazy'}
								/>
							{:else}
								<div class="bg-muted h-full w-full"></div>
							{/if}
							<!-- Gradient overlay -->
							<div
								class="absolute inset-0 bg-gradient-to-r from-black/80 via-black/40 to-transparent"
							></div>
						</div>

						<!-- Content overlay -->
						<div class="absolute inset-0 flex flex-col justify-end p-6">
							<div class="max-w-lg space-y-2">
								<!-- Platform badge -->
								<span
									class="inline-block rounded px-2 py-1 text-xs font-bold {getPlatformColor(
										modpack.platform
									)}"
								>
									{getPlatformLabel(modpack.platform)}
								</span>

								<!-- Title -->
								<h3 class="text-2xl font-bold text-white drop-shadow-lg">
									{modpack.name}
								</h3>

								<!-- Author & Stats -->
								<div class="flex items-center gap-4 text-sm text-white/80">
									<span>by {modpack.author}</span>
									<span class="flex items-center gap-1">
										<Download class="h-3.5 w-3.5" />
										{formatDownloads(modpack.downloads)} downloads
									</span>
								</div>

								<!-- Description -->
								<p class="line-clamp-2 text-sm text-white/70">
									{modpack.description}
								</p>
							</div>
						</div>
					</button>
				{/each}
			</div>

			<!-- Navigation arrows -->
			{#if modpacks.length > 1}
				<button
					type="button"
					class="featured-carousel-nav bg-background/80 hover:bg-background left-2 rounded-full p-2"
					onclick={goToPrev}
					aria-label="Previous slide"
				>
					<ChevronLeft class="h-5 w-5" />
				</button>
				<button
					type="button"
					class="featured-carousel-nav bg-background/80 hover:bg-background right-2 rounded-full p-2"
					onclick={goToNext}
					aria-label="Next slide"
				>
					<ChevronRight class="h-5 w-5" />
				</button>
			{/if}

			<!-- Navigation dots -->
			{#if modpacks.length > 1}
				<div class="featured-carousel-dots absolute right-0 bottom-0 left-0">
					<!-- eslint-disable-next-line @typescript-eslint/no-unused-vars -->
					{#each modpacks as _modpack, idx (idx)}
						<button
							type="button"
							class="featured-carousel-dot"
							data-active={idx === currentIndex}
							onclick={() => goToSlide(idx)}
							aria-label="Go to slide {idx + 1}"
						></button>
					{/each}
				</div>
			{/if}
		</div>
	</div>
{/if}
