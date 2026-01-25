<script lang="ts">
	import {
		Loader2,
		Play,
		Search,
		FolderOpen,
		Trash2,
		Gamepad2,
		Globe,
		Shield,
		Sparkles,
	} from '@lucide/svelte';
	import { alertDialogStore } from '$lib/stores/alertDialog.svelte';
	import { nestedScroll } from '$lib/utils/scroll';
	import { Button } from '$lib/ui/button';
	import { Input } from '$lib/ui/input';
	import * as instanceDetailService from '$lib/services/instance-detail';
	import type { World, Server } from '$lib/types';

	interface Props {
		instanceId: string;
		minecraftVersion: string;
		activeAccountId: string | null;
	}

	let { instanceId, minecraftVersion, activeAccountId }: Props = $props();

	// Tab state for worlds/servers
	let activeTab = $state<'worlds' | 'servers'>('worlds');

	// Data
	let worlds = $state<World[]>([]);
	let servers = $state<Server[]>([]);

	// Loading states
	let isLoadingWorlds = $state(false);
	let isLoadingServers = $state(false);
	let launching = $state<string | null>(null);
	let connecting = $state<string | null>(null);
	let error = $state<string | null>(null);

	// Search
	let worldSearch = $state('');
	let serverSearch = $state('');

	// Track last loaded instance
	let lastLoadedId = $state<string | null>(null);

	const supportsQuickPlay = $derived(checkQuickPlaySupport(minecraftVersion));

	$effect(() => {
		if (instanceId && instanceId !== lastLoadedId) {
			lastLoadedId = instanceId;
			loadWorlds();
			loadServers();
		}
	});

	function checkQuickPlaySupport(version: string): boolean {
		const parts = version
			.split('.')
			.map((p) => p.replace(/[^0-9].*$/, ''))
			.filter(Boolean)
			.map((p) => parseInt(p, 10))
			.filter((n) => !Number.isNaN(n));
		const major = parts[0] ?? 0;
		const minor = parts[1] ?? 0;
		return major > 1 || (major === 1 && minor >= 20);
	}

	async function loadWorlds() {
		isLoadingWorlds = true;
		error = null;

		try {
			const response = await instanceDetailService.getInstanceWorlds(instanceId);
			worlds = response.worlds;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load worlds';
			console.error('Failed to load worlds:', e);
		} finally {
			isLoadingWorlds = false;
		}
	}

	async function loadServers() {
		isLoadingServers = true;

		try {
			const response = await instanceDetailService.getInstanceServers(instanceId);
			servers = response.servers;
		} catch (e) {
			console.error('Failed to load servers:', e);
		} finally {
			isLoadingServers = false;
		}
	}

	// Filtered lists
	const filteredWorlds = $derived(
		worlds.filter((world) => {
			const query = worldSearch.trim().toLowerCase();
			if (!query) return true;
			return (
				world.name.toLowerCase().includes(query) ||
				world.folderName.toLowerCase().includes(query) ||
				(world.versionName ?? '').toLowerCase().includes(query)
			);
		})
	);

	const filteredServers = $derived(
		servers.filter((server) => {
			const query = serverSearch.trim().toLowerCase();
			if (!query) return true;
			return server.name.toLowerCase().includes(query) || server.ip.toLowerCase().includes(query);
		})
	);

	function worldIconSrc(world: World): string {
		if (world.iconBase64) {
			const hasPrefix = world.iconBase64.startsWith('data:');
			return hasPrefix ? world.iconBase64 : `data:image/png;base64,${world.iconBase64}`;
		}
		return '/blocks/grass_block.png';
	}

	function serverIconSrc(server: Server): string {
		if (server.iconBase64) {
			const hasPrefix = server.iconBase64.startsWith('data:');
			return hasPrefix ? server.iconBase64 : `data:image/png;base64,${server.iconBase64}`;
		}
		return '/icons/entities/creeper/creeper.png';
	}

	function formatDate(ms: number | null): string {
		if (!ms) return 'Never';
		const date = new Date(ms);
		return isNaN(date.getTime()) ? 'Unknown' : date.toLocaleString();
	}

	function formatSize(size: number): string {
		if (size === 0) return '0 B';
		const units = ['B', 'KB', 'MB', 'GB'];
		const i = Math.min(Math.floor(Math.log(size) / Math.log(1024)), units.length - 1);
		return `${(size / 1024 ** i).toFixed(1)} ${units[i]}`;
	}

	function gameModeLabel(mode: string | null): string {
		if (!mode) return 'Unknown';
		return mode.charAt(0).toUpperCase() + mode.slice(1);
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

		launching = world.folderName;
		error = null;

		try {
			await instanceDetailService.launchIntoWorld(instanceId, activeAccountId, world.folderName);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to launch world';
			console.error('Failed to launch world:', e);
		} finally {
			launching = null;
		}
	}

	async function handleConnectToServer(server: Server) {
		if (!supportsQuickPlay) {
			error = 'Quick Play is only available on Minecraft 1.20+.';
			return;
		}
		if (!activeAccountId) {
			error = 'Select an active account before connecting.';
			return;
		}

		connecting = server.ip;
		error = null;

		try {
			await instanceDetailService.launchIntoServer(instanceId, activeAccountId, server.ip);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to connect';
			console.error('Failed to quick-connect:', e);
		} finally {
			connecting = null;
		}
	}

	async function openWorldFolder(world: World) {
		try {
			await instanceDetailService.openWorldFolder(instanceId, world.folderName);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to open folder';
			console.error('Failed to open world folder:', e);
		}
	}

	async function handleDeleteWorld(world: World) {
		const confirmed = await alertDialogStore.confirm({
			title: 'Delete World',
			message: `Delete world "${world.name}"? This action cannot be undone.`,
			type: 'warning',
			confirmText: 'Delete',
			cancelText: 'Cancel',
		});

		if (!confirmed) return;

		try {
			await instanceDetailService.deleteWorld(instanceId, world.folderName);
			worlds = worlds.filter((w) => w.folderName !== world.folderName);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to delete world';
			console.error('Failed to delete world:', e);
		}
	}
</script>

<div class="flex h-full flex-col gap-4">
	<!-- Tab Switcher -->
	<div class="flex items-center justify-between">
		<div class="flex gap-2">
			<Button
				size="sm"
				variant={activeTab === 'worlds' ? 'default' : 'secondary'}
				onclick={() => (activeTab = 'worlds')}
			>
				<Gamepad2 class="mr-1.5 h-4 w-4" />
				Worlds ({worlds.length})
			</Button>
			<Button
				size="sm"
				variant={activeTab === 'servers' ? 'default' : 'secondary'}
				onclick={() => (activeTab = 'servers')}
			>
				<Globe class="mr-1.5 h-4 w-4" />
				Servers ({servers.length})
			</Button>
		</div>

		{#if !supportsQuickPlay}
			<p class="text-muted-foreground text-sm">Quick Play requires Minecraft 1.20+</p>
		{/if}
	</div>

	<!-- Error Display -->
	{#if error}
		<div
			class="border-destructive/60 bg-destructive/10 text-destructive rounded border px-4 py-3 text-sm"
		>
			{error}
		</div>
	{/if}

	<!-- Worlds Tab -->
	{#if activeTab === 'worlds'}
		<!-- Search Bar -->
		<div class="relative">
			<Input
				placeholder="Search worlds..."
				value={worldSearch}
				oninput={(e) => (worldSearch = e.currentTarget.value)}
				class="pl-10"
			/>
			<Search class="text-muted-foreground absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2" />
		</div>

		<!-- Worlds List -->
		<div class="flex-1 overflow-y-auto" use:nestedScroll>
			{#if isLoadingWorlds}
				<div class="text-muted-foreground flex items-center justify-center gap-3 py-12">
					<Loader2 class="h-6 w-6 animate-spin" />
					<span>Loading worlds...</span>
				</div>
			{:else if filteredWorlds.length === 0}
				<div class="text-muted-foreground flex flex-col items-center justify-center gap-3 py-12">
					<Gamepad2 class="h-12 w-12 opacity-50" />
					{#if worldSearch}
						<p>No worlds matching "{worldSearch}"</p>
					{:else}
						<p>No worlds found</p>
						<p class="text-sm">Create a new world in-game to see it here</p>
					{/if}
				</div>
			{:else}
				<div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
					{#each filteredWorlds as world (world.folderName)}
						<!-- World Card with background -->
						<div class="border-border bg-background/80 relative overflow-hidden rounded-lg border">
							<!-- Subtle background icon -->
							{#if world.iconBase64}
								<div class="absolute inset-0 flex items-center justify-center opacity-5">
									<img
										src={worldIconSrc(world)}
										alt=""
										class="h-32 w-32 blur-sm"
										aria-hidden="true"
									/>
								</div>
							{/if}

							<div class="relative flex flex-col gap-3 p-4">
								<!-- World Info -->
								<div class="flex items-start gap-3">
									<img
										src={worldIconSrc(world)}
										alt=""
										class="border-border h-14 w-14 flex-shrink-0 rounded-lg border object-cover"
									/>
									<div class="min-w-0 flex-1">
										<p class="truncate font-semibold">{world.name}</p>
										<p class="text-muted-foreground truncate text-xs">{world.folderName}</p>
										<div class="mt-2 flex flex-wrap gap-1.5">
											<span class="bg-muted rounded px-2 py-0.5 text-[11px] font-medium capitalize">
												{gameModeLabel(world.gameMode)}
											</span>
											{#if world.versionName}
												<span
													class="text-muted-foreground bg-muted/60 rounded px-2 py-0.5 text-[11px]"
												>
													{world.versionName}
												</span>
											{/if}
											{#if world.cheatsEnabled}
												<span
													class="flex items-center gap-0.5 rounded bg-amber-500/20 px-2 py-0.5 text-[11px] text-amber-500"
												>
													<Sparkles class="h-3 w-3" />
													Cheats
												</span>
											{/if}
										</div>
									</div>
								</div>

								<!-- Stats -->
								<div class="text-muted-foreground grid grid-cols-2 gap-2 text-xs">
									<div>
										<span class="text-foreground font-medium">Last played:</span><br />
										{formatDate(world.lastPlayed)}
									</div>
									<div>
										<span class="text-foreground font-medium">Size:</span><br />
										{formatSize(world.size)}
									</div>
								</div>

								<!-- Actions -->
								<div class="flex gap-2">
									{#if supportsQuickPlay}
										<Button
											class="flex-1 justify-center"
											variant="secondary"
											size="sm"
											onclick={() => handleLaunchWorld(world)}
											disabled={!!launching}
										>
											{#if launching === world.folderName}
												<Loader2 class="mr-2 h-4 w-4 animate-spin" />
												Launching...
											{:else}
												<Play class="mr-2 h-4 w-4" />
												Play
											{/if}
										</Button>
									{/if}
									<Button
										variant="outline"
										size="sm"
										onclick={() => openWorldFolder(world)}
										title="Open folder"
									>
										<FolderOpen class="h-4 w-4" />
									</Button>
									<Button
										variant="destructive"
										size="sm"
										onclick={() => handleDeleteWorld(world)}
										title="Delete world"
									>
										<Trash2 class="h-4 w-4" />
									</Button>
								</div>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</div>
	{/if}

	<!-- Servers Tab -->
	{#if activeTab === 'servers'}
		<!-- Search Bar -->
		<div class="relative">
			<Input
				placeholder="Search servers..."
				value={serverSearch}
				oninput={(e) => (serverSearch = e.currentTarget.value)}
				class="pl-10"
			/>
			<Search class="text-muted-foreground absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2" />
		</div>

		<!-- Servers List -->
		<div class="flex-1 overflow-y-auto" use:nestedScroll>
			{#if isLoadingServers}
				<div class="text-muted-foreground flex items-center justify-center gap-3 py-12">
					<Loader2 class="h-6 w-6 animate-spin" />
					<span>Loading servers...</span>
				</div>
			{:else if filteredServers.length === 0}
				<div class="text-muted-foreground flex flex-col items-center justify-center gap-3 py-12">
					<Globe class="h-12 w-12 opacity-50" />
					{#if serverSearch}
						<p>No servers matching "{serverSearch}"</p>
					{:else}
						<p>No saved servers</p>
						<p class="text-sm">Add servers in-game to see them here</p>
					{/if}
				</div>
			{:else}
				<div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
					{#each filteredServers as server (server.ip)}
						<div class="border-border bg-background/80 flex flex-col gap-3 rounded-lg border p-4">
							<!-- Server Info -->
							<div class="flex items-start gap-3">
								<img
									src={serverIconSrc(server)}
									alt=""
									class="border-border bg-muted h-14 w-14 flex-shrink-0 rounded-lg border object-cover"
								/>
								<div class="min-w-0 flex-1">
									<p class="truncate font-semibold">{server.name}</p>
									<p class="text-muted-foreground truncate text-xs">{server.ip}</p>
									<div class="mt-2 flex flex-wrap items-center gap-1.5">
										{#if server.hidden}
											<span
												class="bg-muted/60 flex items-center gap-1 rounded px-2 py-0.5 text-[11px]"
											>
												<Shield class="h-3 w-3" />
												Hidden
											</span>
										{/if}
										{#if server.acceptTextures}
											<span
												class="rounded bg-emerald-500/20 px-2 py-0.5 text-[11px] text-emerald-500"
											>
												Server textures
											</span>
										{/if}
									</div>
								</div>
							</div>

							<!-- Connect Button -->
							{#if supportsQuickPlay}
								<Button
									variant="secondary"
									size="sm"
									class="justify-center"
									onclick={() => handleConnectToServer(server)}
									disabled={!!connecting}
								>
									{#if connecting === server.ip}
										<Loader2 class="mr-2 h-4 w-4 animate-spin" />
										Connecting...
									{:else}
										<Play class="mr-2 h-4 w-4" />
										Connect
									{/if}
								</Button>
							{/if}
						</div>
					{/each}
				</div>
			{/if}
		</div>
	{/if}
</div>
