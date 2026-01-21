<script lang="ts">
	import { Sparkles, Package, Download, Loader2, ChevronLeft, ChevronRight } from '@lucide/svelte';
	import type { Modpack, LoaderType } from '$lib/types';

	interface Props {
		modpacks: Modpack[];
		mcVersion: string | null;
		loading?: boolean;
		onModpackClick?: (modpack: Modpack) => void;
	}

	let { modpacks, mcVersion, loading = false, onModpackClick }: Props = $props();

	let displayVersion = $derived(mcVersion || 'Latest');
	let scrollContainer: HTMLDivElement | null = $state(null);
	let canScrollLeft = $state(false);
	let canScrollRight = $state(true);

	function updateScrollButtons() {
		if (!scrollContainer) return;
		canScrollLeft = scrollContainer.scrollLeft > 0;
		canScrollRight =
			scrollContainer.scrollLeft < scrollContainer.scrollWidth - scrollContainer.clientWidth - 1;
	}

	function scrollLeft() {
		if (!scrollContainer) return;
		scrollContainer.scrollBy({ left: -400, behavior: 'smooth' });
	}

	function scrollRight() {
		if (!scrollContainer) return;
		scrollContainer.scrollBy({ left: 400, behavior: 'smooth' });
	}

	function formatDownloads(downloads: number): string {
		if (downloads >= 1_000_000) return `${(downloads / 1_000_000).toFixed(1)}M`;
		if (downloads >= 1_000) return `${(downloads / 1_000).toFixed(1)}K`;
		return downloads.toString();
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

	function getBannerImage(modpack: Modpack): string | null {
		if (modpack.bannerUrl) return modpack.bannerUrl;
		if (modpack.gallery && modpack.gallery.length > 0) {
			return modpack.gallery[0].rawUrl || modpack.gallery[0].url;
		}
		return null;
	}
</script>

<section>
	<!-- Section Header -->
	<div class="section-header">
		<Sparkles class="section-header-icon h-6 w-6" />
		<h2 class="section-header-title">MC {displayVersion} Modpacks</h2>
		<!-- Navigation Arrows -->
		{#if modpacks.length > 3}
			<div class="ml-auto flex gap-2">
				<button
					type="button"
					class="scroll-nav-btn"
					onclick={scrollLeft}
					disabled={!canScrollLeft}
					aria-label="Scroll left"
				>
					<ChevronLeft class="h-5 w-5" />
				</button>
				<button
					type="button"
					class="scroll-nav-btn"
					onclick={scrollRight}
					disabled={!canScrollRight}
					aria-label="Scroll right"
				>
					<ChevronRight class="h-5 w-5" />
				</button>
			</div>
		{/if}
	</div>

	{#if loading}
		<div class="border-border bg-card flex h-48 items-center justify-center border-2">
			<Loader2 class="text-muted-foreground h-8 w-8 animate-spin" />
		</div>
	{:else if modpacks.length === 0}
		<div
			class="border-border bg-card/50 flex h-48 items-center justify-center border-2 border-dashed"
		>
			<div class="text-muted-foreground text-center">
				<Package class="mx-auto h-12 w-12 opacity-50" />
				<p class="mt-2 text-sm">No modpacks for MC {displayVersion} available</p>
			</div>
		</div>
	{:else}
		<div class="latest-version-container">
			<!-- Left fade indicator -->
			{#if canScrollLeft}
				<div class="scroll-fade scroll-fade-left"></div>
			{/if}

			<div class="latest-version-scroll" bind:this={scrollContainer} onscroll={updateScrollButtons}>
				{#each modpacks as modpack (`${modpack.platform}-${modpack.id}`)}
					<button
						type="button"
						class="latest-version-card border-border hover:border-primary/50 overflow-hidden border-2 text-left transition-colors"
						onclick={() => onModpackClick?.(modpack)}
					>
						<!-- 3:1 Banner -->
						<div class="latest-version-banner">
							{#if getBannerImage(modpack)}
								<img
									src={getBannerImage(modpack)}
									alt=""
									class="h-full w-full object-cover"
									loading="lazy"
								/>
							{:else if modpack.iconUrl}
								<div
									class="bg-card relative flex h-full w-full items-center justify-center overflow-hidden"
								>
									<!-- Blurred background -->
									<img
										src={modpack.iconUrl}
										alt=""
										class="absolute inset-0 h-full w-full object-cover opacity-60 blur-md"
									/>
									<!-- Centered sharp icon - larger -->
									<img
										src={modpack.iconUrl}
										alt=""
										class="relative z-10 h-20 w-20 object-contain drop-shadow-lg"
									/>
								</div>
							{:else}
								<div class="bg-muted flex h-full w-full items-center justify-center">
									<Package class="text-muted-foreground/30 h-12 w-12" />
								</div>
							{/if}
							<!-- Gradient overlay -->
							<div
								class="absolute inset-0 bg-gradient-to-t from-black/80 via-black/20 to-transparent"
							></div>
						</div>

						<!-- Content overlaying bottom of banner -->
						<div class="latest-version-content">
							{#if modpack.iconUrl}
								<img
									src={modpack.iconUrl}
									alt=""
									class="border-card h-16 w-16 flex-shrink-0 border-4 object-cover shadow-lg"
								/>
							{:else}
								<div
									class="bg-muted border-card flex h-16 w-16 flex-shrink-0 items-center justify-center border-4"
								>
									<Package class="text-muted-foreground/50 h-8 w-8" />
								</div>
							{/if}
							<div class="min-w-0 flex-1">
								<h3 class="truncate font-bold text-white">{modpack.name}</h3>
								<p class="line-clamp-2 text-sm text-white/80">{modpack.description}</p>
								<div class="mt-2 flex flex-wrap items-center gap-2">
									{#each (modpack.loaders || [])
										.filter((l) => l && l !== 'unknown' && l !== 'vanilla')
										.slice(0, 2) as loader (loader)}
										<span class="rounded px-1.5 py-0.5 text-xs {getLoaderColor(loader)}">
											{loader}
										</span>
									{/each}
									<span class="flex items-center gap-1 text-xs text-white/70">
										<Download class="h-3 w-3" />
										{formatDownloads(modpack.downloads)}
									</span>
								</div>
							</div>
						</div>
					</button>
				{/each}
			</div>

			<!-- Right fade indicator -->
			{#if canScrollRight}
				<div class="scroll-fade scroll-fade-right"></div>
			{/if}
		</div>
	{/if}
</section>

<style>
	.latest-version-container {
		position: relative;
		overflow: hidden;
	}

	.latest-version-scroll {
		display: flex;
		gap: 1rem;
		overflow-x: auto;
		scroll-behavior: smooth;
		scrollbar-width: none;
		-ms-overflow-style: none;
		padding-bottom: 0.5rem;
	}

	.latest-version-scroll::-webkit-scrollbar {
		display: none;
	}

	.scroll-fade {
		position: absolute;
		top: 0;
		bottom: 0.5rem;
		width: 60px;
		pointer-events: none;
		z-index: 10;
	}

	.scroll-fade-left {
		left: 0;
		background: linear-gradient(to right, var(--background), transparent);
	}

	.scroll-fade-right {
		right: 0;
		background: linear-gradient(to left, var(--background), transparent);
	}

	.latest-version-card {
		display: flex;
		flex-direction: column;
		position: relative;
		background: var(--card);
		flex-shrink: 0;
		width: 340px;
	}

	@media (max-width: 640px) {
		.latest-version-card {
			width: 280px;
		}
	}

	.latest-version-banner {
		position: relative;
		aspect-ratio: 3 / 1;
		width: 100%;
		overflow: hidden;
	}

	.latest-version-content {
		display: flex;
		gap: 0.75rem;
		padding: 0.75rem;
		margin-top: -2rem;
		position: relative;
		z-index: 10;
	}

	.scroll-nav-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 2rem;
		height: 2rem;
		border: 2px solid var(--border);
		background: var(--card);
		color: var(--foreground);
		transition: all 0.2s ease;
	}

	.scroll-nav-btn:hover:not(:disabled) {
		border-color: var(--primary);
		color: var(--primary);
	}

	.scroll-nav-btn:disabled {
		opacity: 0.3;
		cursor: not-allowed;
	}
</style>
