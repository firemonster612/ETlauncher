<script lang="ts">
	import { Gamepad2 } from '@lucide/svelte';
	import HomepageInstanceCard from './HomepageInstanceCard.svelte';
	import type { Instance, LaunchStatus } from '$lib/types';

	interface Props {
		instances: Instance[];
		launchStatuses: Map<string, LaunchStatus>;
		onLaunch: (instanceId: string) => void;
		onKill: (instanceId: string) => void;
		onCardClick: (instance: Instance) => void;
	}

	let { instances, launchStatuses, onLaunch, onKill, onCardClick }: Props = $props();
</script>

{#if instances.length > 0}
	<div class="space-y-2">
		<!-- Header -->
		<div class="flex items-center gap-2">
			<Gamepad2 class="text-primary h-4 w-4" />
			<h2 class="text-sm font-bold uppercase tracking-wider">Most Played</h2>
		</div>

		<!-- 2x2 Grid -->
		<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
			{#each instances as instance (instance.id)}
				<HomepageInstanceCard
					{instance}
					launchStatus={launchStatuses.get(instance.id)}
					{onLaunch}
					{onKill}
					{onCardClick}
				/>
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
