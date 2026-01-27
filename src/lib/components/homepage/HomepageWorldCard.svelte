<script lang="ts">
	import { Play, Globe, AlertCircle } from '@lucide/svelte';
	import { Button } from '$lib/ui/button';
	import type { HomepageWorld } from '$lib/types';
	import * as Tooltip from '$lib/ui/tooltip';

	interface Props {
		world: HomepageWorld;
		onLaunch: (world: HomepageWorld) => void;
	}

	let { world, onLaunch }: Props = $props();

	function formatLastPlayed(timestamp: number | null): string {
		if (!timestamp) return 'Never played';
		const date = new Date(timestamp);
		const now = new Date();
		const diffMs = now.getTime() - date.getTime();
		const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

		if (diffDays === 0) return 'Today';
		if (diffDays === 1) return 'Yesterday';
		if (diffDays < 7) return `${diffDays} days ago`;
		if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks ago`;
		return date.toLocaleDateString();
	}

	function handleLaunchClick(e: MouseEvent) {
		e.stopPropagation();
		if (world.supportsQuickPlay) {
			onLaunch(world);
		}
	}
</script>

<div
	class="border-border bg-card group overflow-hidden border-2 transition-all duration-200"
>
	<!-- World Icon / Banner -->
	<div class="from-muted/50 to-muted relative h-24 overflow-hidden bg-gradient-to-br">
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
	</div>

	<!-- Info -->
	<div class="p-2">
		<h3 class="truncate text-sm font-semibold leading-tight" title={world.name}>
			{world.name}
		</h3>
		<div class="text-muted-foreground mt-0.5 flex items-center justify-between text-[10px]">
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
				class="h-7 w-full text-xs"
				onclick={handleLaunchClick}
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
						class="h-7 w-full cursor-not-allowed text-xs opacity-50"
						disabled
					>
						<AlertCircle class="mr-1 h-3 w-3" />
						1.20+ only
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
