<script lang="ts">
	import { Package, Download, Calendar } from '@lucide/svelte';
	import { onDestroy } from 'svelte';
	import type { Modpack, ModpackPlatform, LoaderType } from '$lib/types';

	interface Props {
		modpack: Modpack;
		onclick?: () => void;
	}

	let { modpack, onclick }: Props = $props();

	// Hover state for gallery cycling
	let isHovered = $state(false);
	let currentGalleryIndex = $state(0);
	let galleryInterval: ReturnType<typeof setInterval> | null = null;

	// Get all available gallery images
	let galleryImages = $derived(
		modpack.gallery?.map((img) => img.rawUrl || img.url).filter(Boolean) ?? []
	);

	// Get banner image - try bannerUrl first, then current gallery image, then null
	let bannerImage = $derived(() => {
		if (modpack.bannerUrl) return modpack.bannerUrl;
		if (galleryImages.length > 0) {
			// When hovered with multiple gallery images, cycle through them
			if (isHovered && galleryImages.length > 1) {
				return galleryImages[currentGalleryIndex];
			}
			return galleryImages[0];
		}
		return null;
	});

	// Handle hover start - begin gallery cycling
	function handleMouseEnter() {
		isHovered = true;
		if (galleryImages.length > 1 && !modpack.bannerUrl) {
			currentGalleryIndex = 0;
			galleryInterval = setInterval(() => {
				currentGalleryIndex = (currentGalleryIndex + 1) % galleryImages.length;
			}, 2000);
		}
	}

	// Handle hover end - stop cycling and reset
	function handleMouseLeave() {
		isHovered = false;
		if (galleryInterval) {
			clearInterval(galleryInterval);
			galleryInterval = null;
		}
		currentGalleryIndex = 0;
	}

	// Cleanup on destroy
	onDestroy(() => {
		if (galleryInterval) {
			clearInterval(galleryInterval);
		}
	});

	function formatDownloads(downloads: number): string {
		if (downloads >= 1_000_000) {
			return `${(downloads / 1_000_000).toFixed(1)}M`;
		}
		if (downloads >= 1_000) {
			return `${(downloads / 1_000).toFixed(1)}K`;
		}
		return downloads.toString();
	}

	function formatDate(timestamp?: number): string {
		if (!timestamp) return '';
		const date = new Date(timestamp * 1000);
		const now = new Date();
		const diffMs = now.getTime() - date.getTime();
		const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

		if (diffDays === 0) return 'Today';
		if (diffDays === 1) return 'Yesterday';
		if (diffDays < 7) return `${diffDays}d ago`;
		if (diffDays < 30) return `${Math.floor(diffDays / 7)}w ago`;
		if (diffDays < 365) return `${Math.floor(diffDays / 30)}mo ago`;
		return `${Math.floor(diffDays / 365)}y ago`;
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

	// Get a gradient background color based on platform for cards without images
	function getGradientBg(platform: ModpackPlatform): string {
		switch (platform) {
			case 'modrinth':
				return 'from-green-900/50 to-green-950/80';
			case 'curseforge':
				return 'from-orange-900/50 to-orange-950/80';
			case 'ftb':
				return 'from-blue-900/50 to-blue-950/80';
			case 'technic':
				return 'from-yellow-900/50 to-yellow-950/80';
			case 'atlauncher':
				return 'from-purple-900/50 to-purple-950/80';
			default:
				return 'from-muted to-background';
		}
	}
</script>

<button
	type="button"
	class="modpack-card border-border bg-card flex w-full cursor-pointer flex-col border-2 text-left"
	{onclick}
	onmouseenter={handleMouseEnter}
	onmouseleave={handleMouseLeave}
>
	<!-- 16:9 Banner Area -->
	<div class="relative aspect-video w-full overflow-hidden">
		{#if bannerImage()}
			<img
				src={bannerImage()}
				alt=""
				class="modpack-card-image h-full w-full object-cover"
				loading="lazy"
			/>
		{:else}
			<!-- Gradient background with centered icon when no image -->
			<div
				class="flex h-full w-full items-center justify-center bg-gradient-to-br {getGradientBg(
					modpack.platform
				)}"
			>
				{#if modpack.iconUrl}
					<img
						src={modpack.iconUrl}
						alt={modpack.name}
						class="h-16 w-16 rounded object-cover shadow-lg"
					/>
				{:else}
					<Package class="text-muted-foreground/30 h-16 w-16" />
				{/if}
			</div>
		{/if}

		<!-- Gradient overlay -->
		<div class="modpack-card-gradient absolute inset-0"></div>

		<!-- Platform badge - top right -->
		<div class="absolute top-2 right-2">
			<span
				class="rounded px-1.5 py-0.5 text-[10px] font-bold {getPlatformColor(modpack.platform)}"
			>
				{getPlatformLabel(modpack.platform)}
			</span>
		</div>

		<!-- Loader badges - top left -->
		{#if modpack.loaders && modpack.loaders.filter((l) => l && l !== 'unknown' && l !== 'vanilla').length > 0}
			<div class="absolute top-2 left-2 flex gap-1">
				{#each modpack.loaders
					.filter((l) => l && l !== 'unknown' && l !== 'vanilla')
					.slice(0, 2) as loader (loader)}
					<span class="rounded px-1.5 py-0.5 text-[10px] font-medium {getLoaderColor(loader)}">
						{loader}
					</span>
				{/each}
			</div>
		{/if}
	</div>

	<!-- Content -->
	<div class="flex flex-1 flex-col p-3">
		<!-- Title + Author Row with Icon -->
		<div class="mb-2 flex items-start gap-2">
			{#if modpack.iconUrl && bannerImage()}
				<img
					src={modpack.iconUrl}
					alt=""
					class="h-10 w-10 flex-shrink-0 rounded object-cover"
					loading="lazy"
				/>
			{:else if !bannerImage()}
				<!-- Icon already shown in banner area, skip here -->
			{:else}
				<div class="bg-muted flex h-10 w-10 flex-shrink-0 items-center justify-center rounded">
					<Package class="text-muted-foreground/50 h-5 w-5" />
				</div>
			{/if}
			<div class="min-w-0 flex-1">
				<h3 class="truncate text-sm leading-tight font-bold">{modpack.name}</h3>
				<p class="text-muted-foreground truncate text-xs">{modpack.author}</p>
			</div>
		</div>

		<!-- Description - 2 lines -->
		<p class="text-muted-foreground mb-3 line-clamp-2 flex-1 text-xs leading-relaxed">
			{modpack.description}
		</p>

		<!-- Footer: Downloads + Updated -->
		<div class="text-muted-foreground flex items-center justify-between text-[10px]">
			<span class="flex items-center gap-1">
				<Download class="h-3 w-3" />
				{formatDownloads(modpack.downloads)}
			</span>
			<div class="flex items-center gap-3">
				{#if modpack.mcVersions.length > 0}
					<span>MC {modpack.mcVersions[0]}</span>
				{/if}
				{#if modpack.updatedAt}
					<span class="flex items-center gap-1">
						<Calendar class="h-3 w-3" />
						{formatDate(modpack.updatedAt)}
					</span>
				{/if}
			</div>
		</div>
	</div>
</button>
