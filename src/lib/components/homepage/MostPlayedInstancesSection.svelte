<script lang="ts">
	import { Gamepad2, Play, Square, Loader2, ChevronLeft, ChevronRight } from '@lucide/svelte';
	import { Button } from '$lib/ui/button';
	import { parseIconPath, getIconUrl } from '$lib/utils/icons';
	import type { Instance, LaunchStatus } from '$lib/types';

	interface Props {
		instances: Instance[];
		launchStatuses: Map<string, LaunchStatus>;
		onLaunch: (instanceId: string) => void;
		onKill: (instanceId: string) => void;
		onCardClick: (instance: Instance) => void;
	}

	let { instances, launchStatuses, onLaunch, onKill, onCardClick }: Props = $props();

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

	function formatPlayTime(seconds: number): string {
		if (seconds < 60) return '< 1m';
		const hours = Math.floor(seconds / 3600);
		const minutes = Math.floor((seconds % 3600) / 60);
		if (hours === 0) return `${minutes}m`;
		return `${hours}h`;
	}

	function getIconSrc(iconPath: string | undefined): string {
		const icon = parseIconPath(iconPath);
		if (icon) {
			return getIconUrl(icon);
		}
		return '/icons/entities/creeper/creeper.png';
	}

	function getStatus(instanceId: string) {
		return launchStatuses.get(instanceId)?.status ?? null;
	}

	function isLaunching(status: string | null): boolean {
		return (
			status === 'checkingAccount' ||
			status === 'refreshingToken' ||
			status === 'loadingVersion' ||
			status === 'verifyingFiles' ||
			status === 'downloading' ||
			status === 'checkingJava' ||
			status === 'downloadingJava' ||
			status === 'buildingClasspath' ||
			status === 'launching' ||
			status === 'running'
		);
	}

	function handlePlayClick(e: MouseEvent, instance: Instance) {
		e.stopPropagation();
		const status = getStatus(instance.id);
		if (status === 'running' || status === 'windowReady') {
			onKill(instance.id);
		} else {
			onLaunch(instance.id);
		}
	}
</script>

{#if instances.length > 0}
	<div class="space-y-2">
		<!-- Header -->
		<div class="flex items-center justify-between">
			<div class="flex items-center gap-2">
				<Gamepad2 class="text-primary h-4 w-4" />
				<h2 class="text-sm font-bold uppercase tracking-wider">Most Played</h2>
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

		<!-- Horizontal scroll strip -->
		<div
			bind:this={scrollContainer}
			class="scrollbar-thin scrollbar-thumb-muted scrollbar-track-transparent -mx-2 flex gap-3 overflow-x-auto px-2 pb-2"
		>
			{#each instances as instance (instance.id)}
				{@const status = getStatus(instance.id)}
				<div
					class="border-border bg-card hover:border-primary/60 group w-48 flex-shrink-0 cursor-pointer border-2 transition-all"
					onclick={() => onCardClick(instance)}
					onkeydown={(e) => e.key === 'Enter' && onCardClick(instance)}
					role="button"
					tabindex="0"
				>
					<!-- Icon -->
					<div class="from-muted/50 to-muted relative h-28 overflow-hidden bg-gradient-to-br">
						<div class="absolute inset-0 flex items-center justify-center">
							<img
								src={getIconSrc(instance.iconPath)}
								alt="{instance.name} icon"
								class="pixelated h-16 w-16 drop-shadow-lg"
							/>
						</div>
						<!-- Play time badge -->
						<div class="absolute right-1.5 bottom-1.5 rounded bg-black/60 px-1.5 py-0.5 text-[10px] text-white/90">
							{formatPlayTime(instance.totalPlayTime)}
						</div>
					</div>

					<!-- Info -->
					<div class="p-3">
						<h3 class="truncate text-sm font-semibold" title={instance.name}>
							{instance.name}
						</h3>
						<p class="text-muted-foreground truncate text-xs">
							{instance.minecraftVersion}
						</p>
					</div>

					<!-- Play button -->
					<div class="border-border border-t p-2">
						<Button
							variant="default"
							size="sm"
							class="h-8 w-full"
							onclick={(e: MouseEvent) => handlePlayClick(e, instance)}
							disabled={status !== null &&
								status !== 'stopped' &&
								status !== 'crashed' &&
								status !== 'running' &&
								status !== 'windowReady'}
						>
							{#if isLaunching(status)}
								<Loader2 class="mr-1 h-4 w-4 animate-spin" />
								Launching
							{:else if status === 'windowReady'}
								<Square class="mr-1 h-4 w-4" />
								Running
							{:else}
								<Play class="mr-1 h-4 w-4" />
								Play
							{/if}
						</Button>
					</div>
				</div>
			{/each}
		</div>
	</div>
{:else}
	<div class="space-y-2">
		<div class="flex items-center gap-2">
			<Gamepad2 class="text-primary h-4 w-4" />
			<h2 class="text-sm font-bold uppercase tracking-wider">Most Played</h2>
		</div>
		<div class="border-border bg-muted/30 border-2 border-dashed p-6 text-center">
			<p class="text-muted-foreground text-sm">No instances played yet</p>
			<p class="text-muted-foreground mt-1 text-xs">Play some Minecraft to see your favorites here!</p>
		</div>
	</div>
{/if}
