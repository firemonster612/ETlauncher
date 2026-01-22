<script lang="ts">
	import { ArrowLeft, FolderOpen, Loader2, Play, Search, Trash2 } from '@lucide/svelte';
	import { alertDialogStore } from '$lib/stores/alertDialog.svelte';
	import { Button } from '$lib/ui/button';
	import { Input } from '$lib/ui/input';
	import * as instanceDetailService from '$lib/services/instance-detail';
	import type { World } from '$lib/types';

	interface Props {
		instanceId: string;
		minecraftVersion: string;
		activeAccountId: string | null;
		onBack: () => void;
	}

	let { instanceId, minecraftVersion, activeAccountId, onBack }: Props = $props();

	let worlds = $state<World[]>([]);
	let isLoading = $state(false);
	let search = $state('');
	let error = $state<string | null>(null);
	let launching = $state<string | null>(null);

	let lastLoadedId = $state<string | null>(null);

	$effect(() => {
		if (instanceId && instanceId !== lastLoadedId) {
			loadWorlds();
		}
	});

	const filteredWorlds = $derived(
		worlds.filter((world) => {
			const query = search.trim().toLowerCase();
			if (!query) return true;
			return (
				world.name.toLowerCase().includes(query) ||
				world.folderName.toLowerCase().includes(query) ||
				(world.versionName ?? '').toLowerCase().includes(query)
			);
		})
	);

	const supportsQuickPlay = $derived(checkQuickPlaySupport(minecraftVersion));

	async function loadWorlds() {
		isLoading = true;
		error = null;

		try {
			const response = await instanceDetailService.getInstanceWorlds(instanceId);
			worlds = response.worlds;
			lastLoadedId = instanceId;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load worlds';
			console.error('Failed to load worlds:', e);
		} finally {
			isLoading = false;
		}
	}

	async function handleLaunch(world: World) {
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

	async function openFolder(world: World) {
		try {
			await instanceDetailService.openWorldFolder(instanceId, world.folderName);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to open folder';
			console.error('Failed to open world folder:', e);
		}
	}

	function formatDate(ms: number | null): string {
		if (!ms) return 'Unknown';
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

	function worldIconSrc(world: World): string {
		if (world.iconBase64) {
			const hasPrefix = world.iconBase64.startsWith('data:');
			return hasPrefix ? world.iconBase64 : `data:image/png;base64,${world.iconBase64}`;
		}
		return '/blocks/grass_block.png';
	}

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

	async function handleDelete(world: World) {
		const confirmed = await alertDialogStore.confirm({
			title: 'Delete World',
			message: `Delete world "${world.name}"? This action cannot be undone.`,
			type: 'warning',
			confirmText: 'Delete',
			cancelText: 'Cancel',
		});

		if (!confirmed) {
			return;
		}

		try {
			await instanceDetailService.deleteWorld(instanceId, world.folderName);
			// Remove from local state
			worlds = worlds.filter((w) => w.folderName !== world.folderName);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to delete world';
			console.error('Failed to delete world:', e);
		}
	}
</script>

<div class="flex h-full flex-col">
	<div class="border-border flex items-center justify-between border-b px-6 py-4">
		<div class="flex items-center gap-3">
			<Button variant="ghost" size="icon" onclick={onBack} aria-label="Back">
				<ArrowLeft class="h-5 w-5" />
			</Button>
			<div>
				<p class="text-muted-foreground text-xs tracking-wide uppercase">Worlds</p>
				<h2 class="text-xl font-semibold">Singleplayer saves</h2>
			</div>
		</div>
		{#if !supportsQuickPlay}
			<p class="text-muted-foreground text-xs">Quick Play requires Minecraft 1.20+</p>
		{/if}
	</div>

	<div class="flex flex-1 flex-col gap-4 overflow-y-auto p-6">
		<div class="flex flex-col gap-3 md:flex-row md:items-center">
			<div class="relative flex-1">
				<Input
					placeholder="Search worlds..."
					value={search}
					oninput={(e) => (search = e.currentTarget.value)}
					class="pl-10"
				/>
				<Search class="text-muted-foreground absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2" />
			</div>
		</div>

		{#if error}
			<div
				class="border-destructive/60 bg-destructive/10 text-destructive rounded border px-4 py-3 text-sm"
			>
				{error}
			</div>
		{/if}

		{#if isLoading}
			<div class="text-muted-foreground flex items-center gap-3">
				<Loader2 class="h-5 w-5 animate-spin" />
				<span>Loading worlds...</span>
			</div>
		{:else if filteredWorlds.length === 0}
			<p class="text-muted-foreground text-sm">No worlds found.</p>
		{:else}
			<div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
				{#each filteredWorlds as world (world.folderName)}
					<div class="border-border bg-background/80 flex flex-col gap-3 rounded-lg border p-3">
						<div class="flex items-start gap-3">
							<img
								src={worldIconSrc(world)}
								alt={`${world.name} icon`}
								class="border-border h-12 w-12 rounded border object-cover"
							/>
							<div class="min-w-0 flex-1">
								<p class="truncate font-semibold">{world.name}</p>
								<p class="text-muted-foreground truncate text-xs">Folder: {world.folderName}</p>
								<div class="mt-1 flex flex-wrap gap-2">
									<span class="rounded border px-2 py-0.5 text-[11px] capitalize">
										{gameModeLabel(world.gameMode)}
									</span>
									{#if world.versionName}
										<span class="text-muted-foreground rounded border px-2 py-0.5 text-[11px]">
											{world.versionName}
										</span>
									{/if}
									{#if world.cheatsEnabled}
										<span
											class="rounded border bg-amber-500/20 px-2 py-0.5 text-[11px] text-amber-500"
										>
											Cheats on
										</span>
									{/if}
								</div>
							</div>
						</div>

						<div class="text-muted-foreground text-xs">
							<div>Last played: {formatDate(world.lastPlayed)}</div>
							<div>Size: {formatSize(world.size)}</div>
						</div>

						<div class="flex gap-2">
							{#if supportsQuickPlay}
								<Button
									class="flex-1 justify-center"
									variant="secondary"
									size="sm"
									onclick={() => handleLaunch(world)}
									disabled={!!launching}
								>
									{#if launching === world.folderName}
										<Loader2 class="mr-2 h-4 w-4 animate-spin" />
										Launching...
									{:else}
										<Play class="mr-2 h-4 w-4" />
										Launch
									{/if}
								</Button>
							{/if}
							<Button
								variant="outline"
								size="sm"
								onclick={() => openFolder(world)}
								title="Open folder"
							>
								<FolderOpen class="h-4 w-4" />
							</Button>
							<Button
								variant="destructive"
								size="sm"
								onclick={() => handleDelete(world)}
								title="Delete world"
							>
								<Trash2 class="h-4 w-4" />
							</Button>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</div>
</div>
