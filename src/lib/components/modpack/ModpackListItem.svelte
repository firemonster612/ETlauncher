<script lang="ts">
	import { Package, Download, Calendar, Clock } from '@lucide/svelte';
	import type { Modpack, LoaderType } from '$lib/types';

	interface Props {
		modpack: Modpack;
		onclick?: () => void;
		onmouseenter?: () => void;
		onmouseleave?: () => void;
	}

	let { modpack, onclick, onmouseenter, onmouseleave }: Props = $props();

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
		if (!timestamp) return 'Unknown';
		return new Date(timestamp * 1000).toLocaleDateString();
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
		// Map common categories to colors
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
		const key = category.toLowerCase().replace(/[^a-z]/g, '');
		return colors[key] || 'bg-muted/50 text-muted-foreground';
	}
</script>

<button type="button" class="modpack-list-item" {onclick} {onmouseenter} {onmouseleave}>
	<!-- Icon -->
	{#if modpack.iconUrl}
		<img src={modpack.iconUrl} alt={modpack.name} class="modpack-list-item-icon" />
	{:else}
		<div class="modpack-list-item-icon-placeholder">
			<Package class="text-muted-foreground/50 h-8 w-8" />
		</div>
	{/if}

	<!-- Content -->
	<div class="flex min-w-0 flex-col gap-2 text-left">
		<!-- Title + Author -->
		<div class="flex items-baseline gap-2">
			<h3 class="truncate font-bold">{modpack.name}</h3>
			<span class="text-muted-foreground flex-shrink-0 text-sm">by {modpack.author}</span>
		</div>

		<!-- Description -->
		<p class="text-muted-foreground line-clamp-2 text-sm">
			{modpack.description}
		</p>

		<!-- Stats row -->
		<div class="text-muted-foreground flex flex-wrap items-center gap-4 text-xs">
			<span class="flex items-center gap-1">
				<Download class="h-3 w-3" />
				{formatDownloads(modpack.downloads)}
			</span>
			<span class="flex items-center gap-1">
				<Calendar class="h-3 w-3" />
				{formatDate(modpack.createdAt)}
			</span>
			<span class="flex items-center gap-1">
				<Clock class="h-3 w-3" />
				{formatDate(modpack.updatedAt)}
			</span>
			{#if modpack.mcVersions.length > 0}
				<span class="bg-primary/20 text-primary rounded px-1.5 py-0.5">
					{modpack.mcVersions[0]}
				</span>
			{/if}
		</div>

		<!-- Loaders + Categories -->
		<div class="flex flex-wrap items-center gap-1.5">
			{#each (modpack.loaders || []).filter((l) => l && l !== 'unknown' && l !== 'vanilla') as loader (loader)}
				<span class="rounded px-1.5 py-0.5 text-xs {getLoaderColor(loader)}">
					{loader}
				</span>
			{/each}
			{#each modpack.categories.slice(0, 4) as category (category)}
				<span class="category-tag-with-icon {getCategoryColor(category)}">
					{category}
				</span>
			{/each}
			{#if modpack.categories.length > 4}
				<span class="text-muted-foreground text-xs">+{modpack.categories.length - 4} more</span>
			{/if}
		</div>
	</div>
</button>
