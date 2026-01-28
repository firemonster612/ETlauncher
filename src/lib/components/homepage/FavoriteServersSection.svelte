<script lang="ts">
	import { Server, Box } from '@lucide/svelte';
	import type { HomepageServer } from '$lib/types';

	interface Props {
		servers: HomepageServer[];
	}

	let { servers }: Props = $props();

	function getServerIcon(server: HomepageServer): string {
		if (server.iconBase64) {
			return `data:image/png;base64,${server.iconBase64}`;
		}
		return '/icons/entities/creeper/creeper.png';
	}
</script>

<div class="flex h-full flex-col space-y-2">
	<!-- Header -->
	<div class="flex items-center gap-2">
		<Server class="text-primary h-4 w-4" />
		<h2 class="text-sm font-bold uppercase tracking-wider">Favorite Servers</h2>
	</div>

	{#if servers.length > 0}
		<!-- Server list -->
		<div class="border-border bg-card flex-1 overflow-auto border-2">
			{#each servers as server, index (server.ip + server.instanceId)}
				<div
					class="hover:bg-muted/50 flex items-center gap-3 px-3 py-2 transition-colors {index < servers.length - 1 ? 'border-border border-b' : ''}"
				>
					<!-- Server icon -->
					<div class="bg-muted/50 h-8 w-8 flex-shrink-0 overflow-hidden">
						<img
							src={getServerIcon(server)}
							alt=""
							class="pixelated h-full w-full object-cover"
							loading="lazy"
						/>
					</div>

					<!-- Server info -->
					<div class="min-w-0 flex-1">
						<p class="truncate text-sm font-medium" title={server.name}>
							{server.name}
						</p>
						<p class="text-muted-foreground truncate text-xs" title={server.ip}>
							{server.ip}
						</p>
					</div>

					<!-- Instance badge -->
					<div
						class="bg-muted/50 flex items-center gap-1 px-2 py-0.5 text-xs"
						title="From: {server.instanceName}"
					>
						<Box class="h-3 w-3" />
						<span class="max-w-[80px] truncate">{server.instanceName}</span>
					</div>
				</div>
			{/each}
		</div>
	{:else}
		<div class="border-border bg-muted/30 flex flex-1 items-center justify-center border-2 border-dashed p-6 text-center">
			<div>
				<p class="text-muted-foreground text-sm">No servers found</p>
				<p class="text-muted-foreground mt-1 text-xs">Add servers in Minecraft's multiplayer menu</p>
			</div>
		</div>
	{/if}
</div>
