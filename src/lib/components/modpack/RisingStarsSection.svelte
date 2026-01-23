<script lang="ts">
	import { TrendingUp, Package, Download, Loader2 } from '@lucide/svelte';
	import type { Modpack, LoaderType } from '$lib/types';

	interface Props {
		modpacks: Modpack[];
		loading?: boolean;
		onModpackClick?: (modpack: Modpack) => void;
	}

	let { modpacks, loading = false, onModpackClick }: Props = $props();

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
</script>

<section>
	<!-- Section Header -->
	<div class="section-header">
		<TrendingUp class="section-header-icon h-6 w-6" />
		<h2 class="section-header-title text-foreground">Rising Stars</h2>
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
				<p class="mt-2 text-sm">No rising stars available</p>
			</div>
		</div>
	{:else}
		<div class="rising-stars-list">
			{#each modpacks.slice(0, 5) as modpack, idx (`${modpack.platform}-${modpack.id}`)}
				<button
					type="button"
					class="rising-star-item border-border hover:border-primary/50 overflow-hidden border-2 transition-colors"
					onclick={() => onModpackClick?.(modpack)}
				>
					<!-- Rank number -->
					<div class="rising-star-rank">
						<span class="text-primary text-2xl font-bold">#{idx + 1}</span>
					</div>

					<!-- Icon -->
					{#if modpack.iconUrl}
						<img
							src={modpack.iconUrl}
							alt=""
							class="h-12 w-12 flex-shrink-0 object-cover"
							loading="lazy"
						/>
					{:else}
						<div class="bg-muted flex h-12 w-12 flex-shrink-0 items-center justify-center">
							<Package class="text-muted-foreground/50 h-6 w-6" />
						</div>
					{/if}

					<!-- Info -->
					<div class="min-w-0 flex-1">
						<h3 class="truncate font-bold">{modpack.name}</h3>
						<p class="text-muted-foreground truncate text-sm">{modpack.author}</p>
					</div>

					<!-- Stats -->
					<div class="flex flex-shrink-0 flex-col items-end gap-1">
						<span class="text-muted-foreground flex items-center gap-1 text-sm">
							<Download class="h-4 w-4" />
							{formatDownloads(modpack.downloads)}
						</span>
						<div class="flex gap-1">
							{#each (modpack.loaders || [])
								.filter((l) => l && l !== 'unknown' && l !== 'vanilla')
								.slice(0, 2) as loader (loader)}
								<span class="rounded px-1.5 py-0.5 text-xs {getLoaderColor(loader)}">
									{loader}
								</span>
							{/each}
						</div>
					</div>
				</button>
			{/each}
		</div>
	{/if}
</section>

<style>
	.rising-stars-list {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.rising-star-item {
		display: flex;
		align-items: center;
		gap: 1rem;
		padding: 0.75rem;
		background: var(--card);
		text-align: left;
	}

	.rising-star-rank {
		width: 3rem;
		display: flex;
		justify-content: center;
		flex-shrink: 0;
	}
</style>
