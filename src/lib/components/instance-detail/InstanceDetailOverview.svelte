<script lang="ts">
	import {
		Clock,
		Gamepad2,
		Package,
		Globe,
		Play,
		Loader2,
		Image,
		ChevronRight,
	} from '@lucide/svelte';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { Button } from '$lib/ui/button';
	import type { Instance, InstanceDetail, World } from '$lib/types';
	import * as instanceDetailService from '$lib/services/instance-detail';

	interface Props {
		instance: Instance;
		detail: InstanceDetail;
		modCount: number;
		activeAccountId: string | null;
		supportsQuickPlay: boolean;
		screenshotPreviews: Record<string, string>;
		onLoadScreenshotPreview: (filename: string) => void;
		onOpenScreenshotLightbox: (index: number) => void;
		onNavigateToTab: (tab: string) => void;
	}

	let {
		instance,
		detail,
		modCount,
		activeAccountId,
		supportsQuickPlay,
		screenshotPreviews,
		onLoadScreenshotPreview,
		onOpenScreenshotLightbox,
		onNavigateToTab,
	}: Props = $props();

	let launchingWorld = $state<string | null>(null);
	let connectingServer = $state<string | null>(null);
	let error = $state<string | null>(null);

	function formatPlayTime(seconds: number): string {
		if (seconds < 60) return '< 1 min';
		const hours = Math.floor(seconds / 3600);
		const minutes = Math.floor((seconds % 3600) / 60);
		if (hours === 0) return `${minutes}m`;
		if (minutes === 0) return `${hours}h`;
		return `${hours}h ${minutes}m`;
	}

	function worldIconSrc(iconBase64: string | null | undefined): string {
		if (iconBase64) {
			const hasPrefix = iconBase64.startsWith('data:');
			return hasPrefix ? iconBase64 : `data:image/png;base64,${iconBase64}`;
		}
		return '/blocks/grass_block.png';
	}

	function serverIconSrc(iconBase64: string | null | undefined): string {
		if (iconBase64) {
			const hasPrefix = iconBase64.startsWith('data:');
			return hasPrefix ? iconBase64 : `data:image/png;base64,${iconBase64}`;
		}
		return '/icons/entities/creeper/creeper.png';
	}

	async function handleLaunchWorld(world: World) {
		if (!supportsQuickPlay) {
			error = 'Quick Play is only available on Minecraft 1.20+.';
			return;
		}
		if (!activeAccountId) {
			error = 'Select an active account before launching.';
			return;
		}

		launchingWorld = world.folderName;
		error = null;

		try {
			await instanceDetailService.launchIntoWorld(instance.id, activeAccountId, world.folderName);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to launch world';
			console.error('Failed to launch world:', e);
		} finally {
			launchingWorld = null;
		}
	}

	async function handleConnectToServer(serverIp: string) {
		if (!supportsQuickPlay) {
			error = 'Quick Play is only available on Minecraft 1.20+.';
			return;
		}
		if (!activeAccountId) {
			error = 'Select an active account before connecting.';
			return;
		}

		connectingServer = serverIp;
		error = null;

		try {
			await instanceDetailService.launchIntoServer(instance.id, activeAccountId, serverIp);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to connect';
			console.error('Failed to quick-connect:', e);
		} finally {
			connectingServer = null;
		}
	}

	// Get first 3 worlds and servers for quick launch
	const recentWorlds = $derived(detail.recentWorlds.slice(0, 3));
	const recentServers = $derived(detail.savedServers.slice(0, 3));
</script>

<div class="space-y-6">
	{#if error}
		<div
			class="border-destructive/60 bg-destructive/10 text-destructive rounded border px-4 py-3 text-sm"
		>
			{error}
		</div>
	{/if}

	<!-- Stats Cards Grid -->
	<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
		<div class="border-border bg-muted/20 rounded-lg border p-4">
			<div class="flex items-center gap-3">
				<div class="bg-primary/10 rounded-lg p-2.5">
					<Clock class="text-primary h-5 w-5" />
				</div>
				<div>
					<p class="text-muted-foreground text-sm">Play Time</p>
					<p class="text-xl font-bold">{formatPlayTime(detail.totalPlayTime)}</p>
				</div>
			</div>
		</div>

		<div class="border-border bg-muted/20 rounded-lg border p-4">
			<div class="flex items-center gap-3">
				<div class="rounded-lg bg-amber-500/10 p-2.5">
					<Package class="h-5 w-5 text-amber-500" />
				</div>
				<div>
					<p class="text-muted-foreground text-sm">Mods</p>
					<p class="text-xl font-bold">{modCount}</p>
				</div>
			</div>
		</div>

		<div class="border-border bg-muted/20 rounded-lg border p-4">
			<div class="flex items-center gap-3">
				<div class="rounded-lg bg-green-500/10 p-2.5">
					<Gamepad2 class="h-5 w-5 text-green-500" />
				</div>
				<div>
					<p class="text-muted-foreground text-sm">Worlds</p>
					<p class="text-xl font-bold">{detail.recentWorlds.length}</p>
				</div>
			</div>
		</div>

		<div class="border-border bg-muted/20 rounded-lg border p-4">
			<div class="flex items-center gap-3">
				<div class="rounded-lg bg-blue-500/10 p-2.5">
					<Globe class="h-5 w-5 text-blue-500" />
				</div>
				<div>
					<p class="text-muted-foreground text-sm">Servers</p>
					<p class="text-xl font-bold">{detail.savedServers.length}</p>
				</div>
			</div>
		</div>
	</div>

	<!-- Worlds and Screenshots Row -->
	<div class="grid gap-6 lg:grid-cols-2">
		<!-- Quick Launch Worlds -->
		<section class="border-border bg-muted/10 rounded-lg border p-4">
			<div class="mb-3 flex items-center justify-between">
				<h3 class="flex items-center gap-2 font-semibold">
					<Gamepad2 class="h-4 w-4 text-green-500" />
					Quick Launch Worlds
				</h3>
				<Button variant="ghost" size="sm" onclick={() => onNavigateToTab('data')}>
					View All <ChevronRight class="ml-1 h-4 w-4" />
				</Button>
			</div>
			{#if recentWorlds.length === 0}
				<p class="text-muted-foreground text-sm">
					No worlds yet. Create one in-game to see it here.
				</p>
			{:else}
				<div class="space-y-2">
					{#each recentWorlds as world (world.folderName)}
						<div
							class="border-border bg-background/60 flex items-center justify-between rounded-lg border p-3"
						>
							<div class="flex items-center gap-3">
								<img
									src={worldIconSrc(world.iconBase64)}
									alt=""
									class="border-border h-10 w-10 rounded border object-cover"
								/>
								<div class="min-w-0">
									<p class="truncate font-medium">{world.name}</p>
									<p class="text-muted-foreground truncate text-xs">
										{world.gameMode
											? world.gameMode.charAt(0).toUpperCase() + world.gameMode.slice(1)
											: 'Unknown'}
									</p>
								</div>
							</div>
							{#if supportsQuickPlay}
								<Button
									variant="secondary"
									size="sm"
									onclick={() => handleLaunchWorld(world)}
									disabled={!!launchingWorld}
								>
									{#if launchingWorld === world.folderName}
										<Loader2 class="h-4 w-4 animate-spin" />
									{:else}
										<Play class="h-4 w-4" />
									{/if}
								</Button>
							{/if}
						</div>
					{/each}
				</div>
			{/if}
		</section>

		<!-- Recent Screenshots -->
		<section class="border-border bg-muted/10 rounded-lg border p-4">
			<div class="mb-3 flex items-center justify-between">
				<h3 class="flex items-center gap-2 font-semibold">
					<Image class="h-4 w-4 text-purple-500" />
					Recent Screenshots
				</h3>
				<Button variant="ghost" size="sm" onclick={() => onNavigateToTab('gallery')}>
					View All <ChevronRight class="ml-1 h-4 w-4" />
				</Button>
			</div>
			{#if detail.recentScreenshots.length === 0}
				<p class="text-muted-foreground text-sm">
					No screenshots yet. Press F2 in-game to take one.
				</p>
			{:else}
				<div class="grid grid-cols-3 gap-2">
					{#each detail.recentScreenshots.slice(0, 6) as shot, index (shot.filename)}
						<button
							class="border-border bg-muted/40 group aspect-video overflow-hidden rounded border"
							onclick={() => onOpenScreenshotLightbox(index)}
							title={shot.filename}
						>
							<img
								src={screenshotPreviews[shot.filename] ?? convertFileSrc(shot.path)}
								alt={shot.filename}
								class="h-full w-full object-cover transition-transform group-hover:scale-[1.02]"
								loading="lazy"
								onerror={() => onLoadScreenshotPreview(shot.filename)}
							/>
						</button>
					{/each}
				</div>
			{/if}
		</section>
	</div>

	<!-- Quick Connect Servers -->
	{#if recentServers.length > 0}
		<section class="border-border bg-muted/10 rounded-lg border p-4">
			<div class="mb-3 flex items-center justify-between">
				<h3 class="flex items-center gap-2 font-semibold">
					<Globe class="h-4 w-4 text-blue-500" />
					Quick Connect Servers
				</h3>
				<Button variant="ghost" size="sm" onclick={() => onNavigateToTab('data')}>
					View All <ChevronRight class="ml-1 h-4 w-4" />
				</Button>
			</div>
			<div class="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
				{#each recentServers as server (server.ip)}
					<div
						class="border-border bg-background/60 flex items-center justify-between rounded-lg border p-3"
					>
						<div class="flex items-center gap-3">
							<img
								src={serverIconSrc(server.iconBase64)}
								alt=""
								class="border-border bg-muted h-10 w-10 rounded border object-cover"
							/>
							<div class="min-w-0">
								<p class="truncate font-medium">{server.name}</p>
								<p class="text-muted-foreground truncate text-xs">{server.ip}</p>
							</div>
						</div>
						{#if supportsQuickPlay}
							<Button
								variant="secondary"
								size="sm"
								onclick={() => handleConnectToServer(server.ip)}
								disabled={!!connectingServer}
							>
								{#if connectingServer === server.ip}
									<Loader2 class="h-4 w-4 animate-spin" />
								{:else}
									<Play class="h-4 w-4" />
								{/if}
							</Button>
						{/if}
					</div>
				{/each}
			</div>
		</section>
	{/if}

	<!-- Version Info -->
	<section class="border-border bg-muted/10 rounded-lg border p-4">
		<h3 class="mb-3 font-semibold">Version Information</h3>
		<div class="text-muted-foreground grid gap-2 text-sm sm:grid-cols-2">
			<div>
				<span class="text-foreground font-medium">Minecraft:</span>
				{instance.minecraftVersion}
			</div>
			<div>
				<span class="text-foreground font-medium">Loader:</span>
				<span class="capitalize">{instance.loaderType}</span>
				{#if instance.loaderVersion}
					({instance.loaderVersion})
				{/if}
			</div>
			{#if instance.modpackPlatform}
				<div>
					<span class="text-foreground font-medium">Modpack Platform:</span>
					<span class="capitalize">{instance.modpackPlatform}</span>
				</div>
			{/if}
			{#if !supportsQuickPlay}
				<div class="text-amber-500 sm:col-span-2">Quick Play requires Minecraft 1.20 or newer</div>
			{/if}
		</div>
	</section>
</div>
