<script lang="ts">
	import { RefreshCw, Package, Loader2, Clock } from '@lucide/svelte';
	import type { Modpack, LoaderType } from '$lib/types';

	interface Props {
		modpacks: Modpack[];
		loading?: boolean;
		onModpackClick?: (modpack: Modpack) => void;
	}

	let { modpacks, loading = false, onModpackClick }: Props = $props();

	function formatTimeSince(timestamp?: number): string {
		if (!timestamp) return 'Unknown';
		const now = Date.now() / 1000;
		const diff = now - timestamp;

		if (diff < 3600) {
			const mins = Math.floor(diff / 60);
			return `${mins}m ago`;
		}
		if (diff < 86400) {
			const hours = Math.floor(diff / 3600);
			return `${hours}h ago`;
		}
		if (diff < 604800) {
			const days = Math.floor(diff / 86400);
			return `${days}d ago`;
		}
		if (diff < 2592000) {
			const weeks = Math.floor(diff / 604800);
			return `${weeks}w ago`;
		}
		const months = Math.floor(diff / 2592000);
		return `${months}mo ago`;
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
		<RefreshCw class="section-header-icon h-6 w-6" />
		<h2 class="section-header-title text-foreground">Recent Updated</h2>
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
				<p class="mt-2 text-sm">No recently updated modpacks</p>
			</div>
		</div>
	{:else}
		<div class="horizontal-scroll">
			{#each modpacks as modpack (`${modpack.platform}-${modpack.id}`)}
				<button
					type="button"
					class="scroll-card border-border hover:border-primary/50 flex flex-col overflow-hidden border-2 text-left transition-colors"
					onclick={() => onModpackClick?.(modpack)}
				>
					<!-- Banner Image -->
					<div class="bg-muted aspect-video w-full overflow-hidden">
						{#if modpack.bannerUrl || (modpack.gallery && modpack.gallery[0])}
							<img
								src={modpack.bannerUrl || modpack.gallery?.[0]?.rawUrl || modpack.gallery?.[0]?.url}
								alt={modpack.name}
								class="h-full w-full object-cover"
								loading="lazy"
							/>
						{:else if modpack.iconUrl}
							<div class="modpack-empty-thumbnail flex h-full w-full items-center justify-center">
								<img
									src={modpack.iconUrl}
									alt={modpack.name}
									class="h-20 w-20 object-contain drop-shadow-lg"
									loading="lazy"
								/>
							</div>
						{:else}
							<div class="modpack-empty-thumbnail flex h-full w-full items-center justify-center">
								<Package class="text-muted-foreground/50 h-16 w-16" />
							</div>
						{/if}
					</div>

					<!-- Content -->
					<div class="flex flex-1 flex-col gap-2 p-3">
						<!-- Title -->
						<h3 class="truncate font-bold">{modpack.name}</h3>

						<!-- Description -->
						<p class="text-muted-foreground line-clamp-2 flex-1 text-xs">
							{modpack.description}
						</p>

						<!-- Time since update + loader -->
						<div class="flex items-center gap-2">
							<span class="text-muted-foreground flex items-center gap-1 text-xs">
								<Clock class="h-3 w-3" />
								{formatTimeSince(modpack.updatedAt)}
							</span>
							{#each (modpack.loaders || [])
								.filter((l) => l && l !== 'unknown' && l !== 'vanilla')
								.slice(0, 1) as loader (loader)}
								<span class="rounded px-1.5 py-0.5 text-xs {getLoaderColor(loader)}">
									{loader}
								</span>
							{/each}
						</div>

						<!-- Category tags (truncated) -->
						<div class="flex flex-wrap gap-1">
							{#each modpack.categories.slice(0, 3) as category (category)}
								<span class="rounded px-1.5 py-0.5 text-[10px] {getCategoryColor(category)}">
									{category}
								</span>
							{/each}
							{#if modpack.categories.length > 3}
								<span class="text-muted-foreground text-[10px]"
									>+{modpack.categories.length - 3}</span
								>
							{/if}
						</div>
					</div>
				</button>
			{/each}
		</div>
	{/if}
</section>
