<script lang="ts">
	import { Globe, Play, AlertCircle, ChevronLeft, ChevronRight, Sword, Paintbrush, Compass, Eye } from '@lucide/svelte';
	import { Button } from '$lib/ui/button';
	import * as Tooltip from '$lib/ui/tooltip';
	import type { HomepageWorld } from '$lib/types';

	interface Props {
		worlds: HomepageWorld[];
		onLaunch: (world: HomepageWorld) => void;
	}

	let { worlds, onLaunch }: Props = $props();

	let scrollContainer = $state<HTMLDivElement | null>(null);
	let canScrollLeft = $state(false);
	let canScrollRight = $state(false);

	function updateScrollButtons() {
		if (!scrollContainer) return;
		canScrollLeft = scrollContainer.scrollLeft > 0;
		canScrollRight =
			scrollContainer.scrollLeft < scrollContainer.scrollWidth - scrollContainer.clientWidth - 1;
	}

	function scrollLeft() {
		scrollContainer?.scrollBy({ left: -200, behavior: 'smooth' });
	}

	function scrollRight() {
		scrollContainer?.scrollBy({ left: 200, behavior: 'smooth' });
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

	function formatLastPlayed(timestamp: number | null): string {
		if (!timestamp) return 'Never';
		const date = new Date(timestamp);
		const now = new Date();
		const diffMs = now.getTime() - date.getTime();
		const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

		if (diffDays === 0) return 'Today';
		if (diffDays === 1) return 'Yesterday';
		if (diffDays < 7) return `${diffDays}d ago`;
		return date.toLocaleDateString();
	}

	function getGameModeInfo(gameMode: string | null): { label: string; icon: typeof Sword; class: string } {
		switch (gameMode?.toLowerCase()) {
			case 'survival':
				return { label: 'Survival', icon: Sword, class: 'bg-red-500/20 text-red-400 border-red-500/50' };
			case 'creative':
				return { label: 'Creative', icon: Paintbrush, class: 'bg-green-500/20 text-green-400 border-green-500/50' };
			case 'adventure':
				return { label: 'Adventure', icon: Compass, class: 'bg-amber-500/20 text-amber-400 border-amber-500/50' };
			case 'spectator':
				return { label: 'Spectator', icon: Eye, class: 'bg-blue-500/20 text-blue-400 border-blue-500/50' };
			default:
				return { label: 'Unknown', icon: Globe, class: 'bg-muted text-muted-foreground border-muted' };
		}
	}

	function handleLaunchClick(e: MouseEvent, world: HomepageWorld) {
		e.stopPropagation();
		if (world.supportsQuickPlay) {
			onLaunch(world);
		}
	}
</script>

{#if worlds.length > 0}
	<div class="space-y-2">
		<!-- Header -->
		<div class="flex items-center justify-between">
			<div class="flex items-center gap-2">
				<Globe class="text-primary h-4 w-4" />
				<h2 class="text-sm font-bold uppercase tracking-wider">Recent Worlds</h2>
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

		<!-- Horizontal scroll -->
		<div
			bind:this={scrollContainer}
			class="scrollbar-thin scrollbar-thumb-muted scrollbar-track-transparent -mx-2 flex gap-3 overflow-x-auto px-2 pb-2"
		>
			{#each worlds as world (world.instanceId + world.folderName)}
				{@const gameModeInfo = getGameModeInfo(world.gameMode)}
				<div class="border-border bg-card w-44 flex-shrink-0 overflow-hidden border-2">
					<!-- World Icon / Banner -->
					<div class="from-muted/50 to-muted relative h-20 overflow-hidden bg-gradient-to-br">
						{#if world.iconBase64}
							<img
								src="data:image/png;base64,{world.iconBase64}"
								alt="{world.name} icon"
								class="pixelated h-full w-full object-cover"
							/>
						{:else}
							<div class="flex h-full w-full items-center justify-center">
								<Globe class="text-muted-foreground/50 h-8 w-8" />
							</div>
						{/if}
						<!-- Instance badge -->
						<div class="absolute top-1 left-1 rounded bg-black/60 px-1 py-0.5 text-[9px] text-white/90">
							{world.instanceName}
						</div>
						<!-- Game mode badge -->
						{#if world.gameMode}
							<div class="absolute right-1 bottom-1 flex items-center gap-0.5 rounded border px-1 py-0.5 text-[9px] {gameModeInfo.class}">
								<svelte:component this={gameModeInfo.icon} class="h-2.5 w-2.5" />
								{gameModeInfo.label}
							</div>
						{/if}
					</div>

					<!-- Info -->
					<div class="p-2">
						<h3 class="truncate text-xs font-semibold" title={world.name}>
							{world.name}
						</h3>
						<div class="text-muted-foreground flex items-center justify-between text-[10px]">
							<span>{world.minecraftVersion}</span>
							<span>{formatLastPlayed(world.lastPlayed)}</span>
						</div>
					</div>

					<!-- Launch Button -->
					<div class="border-border border-t p-1.5">
						{#if world.supportsQuickPlay}
							<Button
								variant="default"
								size="sm"
								class="h-6 w-full text-xs"
								onclick={(e: MouseEvent) => handleLaunchClick(e, world)}
							>
								<Play class="mr-1 h-3 w-3" />
								Play
							</Button>
						{:else}
							<Tooltip.Root>
								<Tooltip.Trigger>
									<Button
										variant="outline"
										size="sm"
										class="h-6 w-full cursor-not-allowed text-xs opacity-50"
										disabled
									>
										<AlertCircle class="mr-1 h-3 w-3" />
										1.20+
									</Button>
								</Tooltip.Trigger>
								<Tooltip.Content>
									<p>Quick play requires Minecraft 1.20+</p>
									<p class="text-muted-foreground text-xs">This world is on {world.minecraftVersion}</p>
								</Tooltip.Content>
							</Tooltip.Root>
						{/if}
					</div>
				</div>
			{/each}
		</div>
	</div>
{:else}
	<div class="space-y-2">
		<div class="flex items-center gap-2">
			<Globe class="text-primary h-4 w-4" />
			<h2 class="text-sm font-bold uppercase tracking-wider">Recent Worlds</h2>
		</div>
		<div class="border-border bg-muted/30 border-2 border-dashed p-6 text-center">
			<p class="text-muted-foreground text-sm">No worlds found</p>
			<p class="text-muted-foreground mt-1 text-xs">Create a world in Minecraft to see it here!</p>
		</div>
	</div>
{/if}
