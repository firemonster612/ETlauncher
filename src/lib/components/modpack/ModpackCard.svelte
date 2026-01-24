<script lang="ts">
	import { Package, Download, Calendar } from '@lucide/svelte';
	import type { Modpack, ModpackPlatform, LoaderType } from '$lib/types';

	interface Props {
		modpack: Modpack;
		onclick?: () => void;
	}

	let { modpack, onclick }: Props = $props();

	// Get banner image - try bannerUrl first, then first gallery image, then null
	let bannerImage = $derived(() => {
		if (modpack.bannerUrl) return modpack.bannerUrl;
		if (modpack.gallery && modpack.gallery.length > 0) {
			return modpack.gallery[0].rawUrl || modpack.gallery[0].url;
		}
		return null;
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
</script>

<button
	type="button"
	class="modpack-card border-border bg-card flex h-full w-full cursor-pointer flex-col border-2 text-left"
	{onclick}
>
	<!-- 16:9 Banner Area -->
	<div class="relative aspect-video w-full overflow-hidden">
		{#if bannerImage()}
			<img
				src={bannerImage()}
				alt=""
				width="300"
				height="169"
				loading="lazy"
				decoding="async"
				class="modpack-card-image h-full w-full object-cover"
			/>
		{:else}
			<!-- Blurred icon background with centered icon when no banner image -->
			<div class="relative flex h-full w-full items-center justify-center overflow-hidden">
				{#if modpack.iconUrl}
					<!-- Softly blurred icon as background -->
					<img
						src={modpack.iconUrl}
						alt=""
						width="64"
						height="64"
						loading="lazy"
						decoding="async"
						class="absolute inset-0 h-full w-full object-cover opacity-50 blur-sm"
					/>
					<!-- Main icon -->
					<img
						src={modpack.iconUrl}
						alt={modpack.name}
						width="64"
						height="64"
						loading="lazy"
						decoding="async"
						class="relative z-10 h-16 w-16 rounded object-cover shadow-lg"
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
					width="40"
					height="40"
					loading="lazy"
					decoding="async"
					class="h-10 w-10 flex-shrink-0 rounded object-cover"
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
