<script lang="ts">
	import { Globe } from '@lucide/svelte';
	import HomepageWorldCard from './HomepageWorldCard.svelte';
	import type { HomepageWorld } from '$lib/types';

	interface Props {
		worlds: HomepageWorld[];
		onLaunch: (world: HomepageWorld) => void;
	}

	let { worlds, onLaunch }: Props = $props();
</script>

{#if worlds.length > 0}
	<div class="space-y-2">
		<!-- Header -->
		<div class="flex items-center gap-2">
			<Globe class="text-primary h-4 w-4" />
			<h2 class="text-sm font-bold uppercase tracking-wider">Recent Worlds</h2>
		</div>

		<!-- 2x2 Grid -->
		<div class="grid gap-3" style="grid-template-columns: repeat(2, 240px);">
			{#each worlds as world (world.instanceId + world.folderName)}
				<HomepageWorldCard {world} {onLaunch} />
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
