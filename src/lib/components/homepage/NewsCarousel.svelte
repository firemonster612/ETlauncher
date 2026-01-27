<script lang="ts">
	import { ChevronLeft, ChevronRight, Newspaper, ExternalLink } from '@lucide/svelte';
	import { onMount, onDestroy } from 'svelte';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import type { NewsArticle } from '$lib/types';

	interface Props {
		articles: NewsArticle[];
	}

	let { articles }: Props = $props();

	let currentIndex = $state(0);
	let autoPlayInterval: ReturnType<typeof setInterval> | null = null;
	let isPaused = $state(false);

	// Auto-rotate every 6 seconds
	function startAutoPlay() {
		if (autoPlayInterval) clearInterval(autoPlayInterval);
		autoPlayInterval = setInterval(() => {
			if (!isPaused && articles.length > 1) {
				currentIndex = (currentIndex + 1) % articles.length;
			}
		}, 6000);
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
		startAutoPlay();
	}

	function goToPrev() {
		currentIndex = currentIndex === 0 ? articles.length - 1 : currentIndex - 1;
		startAutoPlay();
	}

	function goToNext() {
		currentIndex = (currentIndex + 1) % articles.length;
		startAutoPlay();
	}

	function handleMouseEnter() {
		isPaused = true;
	}

	function handleMouseLeave() {
		isPaused = false;
	}

	async function handleArticleClick(article: NewsArticle) {
		try {
			await openUrl(article.articleUrl);
		} catch (e) {
			console.error('Failed to open article URL:', e);
		}
	}

	function formatDate(dateStr: string): string {
		try {
			const date = new Date(dateStr);
			return date.toLocaleDateString(undefined, {
				year: 'numeric',
				month: 'short',
				day: 'numeric',
			});
		} catch {
			return dateStr;
		}
	}

	function getCategoryColor(category: string): string {
		const lowerCategory = category.toLowerCase();
		if (lowerCategory.includes('snapshot') || lowerCategory.includes('beta')) {
			return 'bg-yellow-500/90 text-black';
		}
		if (lowerCategory.includes('release')) {
			return 'bg-green-500/90 text-white';
		}
		if (lowerCategory.includes('minecraft') || lowerCategory.includes('java')) {
			return 'bg-emerald-600/90 text-white';
		}
		if (lowerCategory.includes('dungeons')) {
			return 'bg-orange-500/90 text-white';
		}
		if (lowerCategory.includes('legends')) {
			return 'bg-purple-500/90 text-white';
		}
		return 'bg-primary/90 text-primary-foreground';
	}
</script>

{#if articles.length > 0}
	<div class="space-y-2">
		<!-- Header -->
		<div class="flex items-center gap-2">
			<Newspaper class="text-primary h-4 w-4" />
			<h2 class="text-sm font-bold uppercase tracking-wider">Minecraft News</h2>
		</div>

		<!-- Carousel -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="news-carousel border-border relative overflow-hidden border-2"
			onmouseenter={handleMouseEnter}
			onmouseleave={handleMouseLeave}
		>
			<!-- Track -->
			<div
				class="flex transition-transform duration-500 ease-in-out"
				style="transform: translateX(-{currentIndex * 100}%)"
			>
				{#each articles as article, idx (article.id)}
					<button
						type="button"
						class="relative aspect-[21/9] w-full flex-shrink-0 cursor-pointer"
						onclick={() => handleArticleClick(article)}
					>
						<!-- Background image -->
						<div class="absolute inset-0">
							{#if article.imageUrl}
								<img
									src={article.imageUrl}
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
								<!-- Category badge -->
								<div class="flex items-center gap-2">
									<span
										class="inline-block rounded px-2 py-1 text-xs font-bold {getCategoryColor(
											article.category
										)}"
									>
										{article.category}
									</span>
									<span class="text-xs text-white/60">
										{formatDate(article.date)}
									</span>
								</div>

								<!-- Title -->
								<h3 class="text-2xl font-bold text-white drop-shadow-lg">
									{article.title}
								</h3>

								<!-- Description -->
								{#if article.description}
									<p class="line-clamp-2 text-sm text-white/70">
										{article.description}
									</p>
								{/if}

								<!-- Read more indicator -->
								<div class="flex items-center gap-1 text-sm text-white/60">
									<span>Read more</span>
									<ExternalLink class="h-3 w-3" />
								</div>
							</div>
						</div>
					</button>
				{/each}
			</div>

			<!-- Navigation arrows -->
			{#if articles.length > 1}
				<button
					type="button"
					class="featured-carousel-nav prev"
					onclick={goToPrev}
					aria-label="Previous slide"
				>
					<ChevronLeft class="h-5 w-5" />
				</button>
				<button
					type="button"
					class="featured-carousel-nav next"
					onclick={goToNext}
					aria-label="Next slide"
				>
					<ChevronRight class="h-5 w-5" />
				</button>
			{/if}

			<!-- Navigation dots -->
			{#if articles.length > 1}
				<div class="absolute right-0 bottom-2 left-0 flex justify-center gap-1.5">
					<!-- eslint-disable-next-line @typescript-eslint/no-unused-vars -->
					{#each articles as _article, idx (idx)}
						<button
							type="button"
							class="h-1.5 rounded-full transition-all {idx === currentIndex
								? 'bg-primary w-6'
								: 'bg-white/50 hover:bg-white/70 w-1.5'}"
							onclick={() => goToSlide(idx)}
							aria-label="Go to slide {idx + 1}"
						></button>
					{/each}
				</div>
			{/if}
		</div>
	</div>
{/if}
