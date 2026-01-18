<script lang="ts">
	import { ChevronLeft, Loader2, Play, X } from '@lucide/svelte';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { Button } from '$lib/ui/button';
	import InstanceDetailScreenshots from '$lib/components/InstanceDetailScreenshots.svelte';
	import InstanceDetailWorlds from '$lib/components/InstanceDetailWorlds.svelte';
	import InstanceDetailServers from '$lib/components/InstanceDetailServers.svelte';
	import ScreenshotLightbox from '$lib/components/ScreenshotLightbox.svelte';
	import * as instanceDetailService from '$lib/services/instance-detail';
	import { getAvatarUrl } from '$lib/services/account';
	import { accountsStore } from '$lib/stores/accounts.svelte';
	import { getIconUrl, parseIconPath } from '$lib/utils/icons';
	import type { Instance, InstanceDetail } from '$lib/types';

	interface Props {
		instance: Instance | null;
		open: boolean;
		onClose: () => void;
	}

	let { instance, open, onClose }: Props = $props();

	let currentView = $state<'dashboard' | 'screenshots' | 'worlds' | 'servers'>('dashboard');
	let detail = $state<InstanceDetail | null>(null);
	let isLoading = $state(false);
	let error = $state<string | null>(null);

	let lightboxIndex = $state<number | null>(null);
	let lightboxData = $state<string | null>(null);
	let lightboxLoading = $state(false);
	let screenshotPreviews = $state<Record<string, string>>({});

	const activeAccountId = $derived(accountsStore.activeAccount?.id ?? null);
	const activeAccountName = $derived(accountsStore.activeAccount?.username ?? null);

	$effect(() => {
		if (open && instance) {
			currentView = 'dashboard';
			screenshotPreviews = {};
			loadDetail();
		} else {
			detail = null;
			error = null;
			screenshotPreviews = {};
		}
	});

	async function loadDetail() {
		if (!instance) return;
		isLoading = true;
		error = null;

		try {
			detail = await instanceDetailService.getInstanceDetail(instance.id);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load instance details';
			console.error('Failed to load instance detail:', e);
		} finally {
			isLoading = false;
		}
	}

	async function loadScreenshotPreview(filename: string) {
		if (!instance || screenshotPreviews[filename]) return;
		try {
			const data = await instanceDetailService.getScreenshotData(instance.id, filename);
			screenshotPreviews = {
				...screenshotPreviews,
				[filename]: `data:image/png;base64,${data}`,
			};
		} catch (e) {
			console.error('Failed to load screenshot preview:', e);
		}
	}

	function handleBackToDashboard() {
		currentView = 'dashboard';
		// Reload detail to reflect any changes made in sub-pages
		screenshotPreviews = {};
		loadDetail();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (!open) return;
		if (e.key === 'Escape') {
			if (currentView !== 'dashboard') {
				handleBackToDashboard();
			} else {
				onClose();
			}
		}
	}

	function getIconSrc(iconPath: string | undefined): string {
		const icon = parseIconPath(iconPath);
		if (icon) return getIconUrl(icon);
		return '/icons/entities/creeper/creeper.png';
	}

	function formatPlayTime(seconds: number): string {
		if (seconds < 60) return '< 1 min';
		const hours = Math.floor(seconds / 3600);
		const minutes = Math.floor((seconds % 3600) / 60);
		if (hours === 0) return `${minutes}m`;
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

	let connectingServer = $state<string | null>(null);
	let launchingWorld = $state<string | null>(null);
	const supportsQuickPlay = $derived(
		instance ? checkQuickPlaySupport(instance.minecraftVersion) : false
	);

	async function handleConnectToServer(serverIp: string) {
		if (!instance) return;
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

	async function handleLaunchWorld(world: { folderName: string }) {
		if (!instance) return;
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

	async function openScreenshotLightbox(index: number) {
		if (!detail || !instance) return;
		lightboxIndex = index;
		await loadLightboxImage(index);
	}

	async function loadLightboxImage(index: number) {
		if (!detail || !instance) return;
		const shot = detail.recentScreenshots[index];
		if (!shot) return;

		lightboxLoading = true;
		try {
			const data = await instanceDetailService.getScreenshotData(instance.id, shot.filename);
			lightboxData = `data:image/png;base64,${data}`;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load screenshot';
			console.error('Failed to load screenshot data:', e);
		} finally {
			lightboxLoading = false;
		}
	}

	function closeScreenshotLightbox() {
		lightboxIndex = null;
		lightboxData = null;
	}

	function goPrevScreenshot() {
		if (!detail || lightboxIndex === null || lightboxIndex === 0) return;
		const nextIndex = lightboxIndex - 1;
		lightboxIndex = nextIndex;
		loadLightboxImage(nextIndex);
	}

	function goNextScreenshot() {
		if (!detail || lightboxIndex === null) return;
		const nextIndex = lightboxIndex + 1;
		if (nextIndex >= detail.recentScreenshots.length) return;
		lightboxIndex = nextIndex;
		loadLightboxImage(nextIndex);
	}

	const canPrevScreenshot = $derived(lightboxIndex !== null && lightboxIndex > 0);
	const canNextScreenshot = $derived(
		detail && lightboxIndex !== null && lightboxIndex < detail.recentScreenshots.length - 1
	);
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open && instance}
	<!-- Backdrop (click to close) -->
	<button
		class="fixed inset-x-0 top-[var(--titlebar-height)] z-50 h-[calc(100vh-var(--titlebar-height))] bg-black/50"
		onclick={onClose}
		aria-label="Close instance detail"
	></button>

	<!-- Full-page panel (matches ContentBrowser sizing) -->
	<div
		class="bg-card border-border fixed inset-x-0 top-[var(--titlebar-height)] z-50 flex h-[calc(100vh-var(--titlebar-height))] w-full max-w-none flex-col overflow-hidden border-l-2 shadow-2xl"
	>
		<div class="border-border flex items-center justify-between border-b px-6 py-4">
			<div class="flex items-center gap-3">
				{#if currentView !== 'dashboard'}
					<Button variant="ghost" size="icon" onclick={handleBackToDashboard} aria-label="Back">
						<ChevronLeft class="h-5 w-5" />
					</Button>
				{/if}
				<img
					src={getIconSrc(instance.iconPath)}
					alt="{instance.name} icon"
					class="pixelated h-12 w-12"
				/>
				<div>
					<p class="text-muted-foreground text-xs tracking-wide uppercase">Instance</p>
					<h2 class="text-xl leading-tight font-bold">{instance.name}</h2>
					{#if activeAccountName}
						<p class="text-muted-foreground mt-0.5 text-xs">Active account: {activeAccountName}</p>
					{/if}
				</div>
			</div>

			<Button variant="ghost" size="icon" onclick={onClose} aria-label="Close">
				<X class="h-5 w-5" />
			</Button>
		</div>

		{#if currentView === 'dashboard'}
			<div class="flex-1 space-y-6 overflow-y-auto p-6">
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
						<span>Loading instance details...</span>
					</div>
				{:else if detail}
					<div class="grid gap-4 md:grid-cols-3">
						<div class="border-border bg-muted/20 rounded-lg border p-4 md:col-span-1">
							<p class="text-muted-foreground text-sm">Total Play Time</p>
							<p class="text-2xl font-semibold">{formatPlayTime(detail.totalPlayTime)}</p>
						</div>

						<div class="border-border bg-muted/10 rounded-lg border p-4 md:col-span-2">
							<p class="text-muted-foreground mb-2 text-sm">Active Account</p>
							{#if activeAccountId && activeAccountName}
								<div class="flex items-center gap-2">
									<img
										src={getAvatarUrl(activeAccountName, 32)}
										alt="{activeAccountName}'s avatar"
										class="pixelated h-8 w-8 rounded"
									/>
									<p class="font-medium">{activeAccountName}</p>
								</div>
							{:else}
								<p class="text-muted-foreground text-sm">
									No active account selected. Select one before launching.
								</p>
							{/if}
						</div>
					</div>

					<!-- Recent Screenshots -->
					<section class="border-border bg-muted/10 rounded-lg border p-4">
						<div class="mb-3 flex items-center justify-between">
							<h3 class="font-semibold">Recent Screenshots</h3>
							<Button variant="ghost" size="sm" onclick={() => (currentView = 'screenshots')}>
								View All →
							</Button>
						</div>
						{#if detail.recentScreenshots.length === 0}
							<p class="text-muted-foreground text-sm">No screenshots yet.</p>
						{:else}
							<div class="grid grid-cols-2 gap-2 md:grid-cols-3 lg:grid-cols-6">
								{#each detail.recentScreenshots as shot, index (shot.filename)}
									<button
										class="border-border bg-muted/40 group h-24 overflow-hidden rounded border"
										onclick={() => openScreenshotLightbox(index)}
										title="Open screenshot"
									>
										<img
											src={screenshotPreviews[shot.filename] ?? convertFileSrc(shot.path)}
											alt={shot.filename}
											class="h-full w-full object-cover transition-transform group-hover:scale-[1.02]"
											loading="lazy"
											onerror={() => loadScreenshotPreview(shot.filename)}
										/>
									</button>
								{/each}
							</div>
						{/if}
					</section>

					<!-- Worlds -->
					<section class="border-border bg-muted/10 rounded-lg border p-4">
						<div class="mb-3 flex items-center justify-between">
							<h3 class="font-semibold">Worlds</h3>
							<Button variant="ghost" size="sm" onclick={() => (currentView = 'worlds')}>
								View All →
							</Button>
						</div>

						{#if detail.recentWorlds.length === 0}
							<p class="text-muted-foreground text-sm">No worlds found.</p>
						{:else}
							<div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
								{#each detail.recentWorlds as world (world.folderName)}
									<div
										class="border-border bg-background/80 flex flex-col gap-2 rounded-lg border p-3"
									>
										<div class="flex items-center gap-2">
											<img
												src={worldIconSrc(world.iconBase64)}
												alt="{world.name} icon"
												class="border-border h-10 w-10 rounded border object-cover"
											/>
											<div class="min-w-0">
												<p class="truncate font-semibold">{world.name}</p>
												<p class="text-muted-foreground truncate text-xs">
													Last played: {world.lastPlayed
														? new Date(world.lastPlayed).toLocaleDateString()
														: 'Unknown'}
												</p>
											</div>
										</div>
										{#if supportsQuickPlay}
											<Button
												variant="secondary"
												size="sm"
												class="justify-center"
												onclick={() => handleLaunchWorld(world)}
												disabled={!!launchingWorld}
											>
												{#if launchingWorld === world.folderName}
													<Loader2 class="mr-1 h-4 w-4 animate-spin" /> Launching...
												{:else}
													<Play class="mr-1 h-4 w-4" /> Launch
												{/if}
											</Button>
										{/if}
									</div>
								{/each}
							</div>
						{/if}
					</section>

					<!-- Servers -->
					<section class="border-border bg-muted/10 rounded-lg border p-4">
						<div class="mb-3 flex items-center justify-between">
							<h3 class="font-semibold">Servers</h3>
							<Button variant="ghost" size="sm" onclick={() => (currentView = 'servers')}>
								View All →
							</Button>
						</div>

						{#if detail.savedServers.length === 0}
							<p class="text-muted-foreground text-sm">No saved servers yet.</p>
						{:else}
							<div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
								{#each detail.savedServers as server (server.ip)}
									<div
										class="border-border bg-background/80 flex flex-col gap-2 rounded-lg border p-3"
									>
										<div class="flex items-center gap-2">
											<img
												src={serverIconSrc(server.iconBase64)}
												alt="{server.name} icon"
												class="border-border bg-muted h-10 w-10 rounded border object-cover"
											/>
											<div class="min-w-0">
												<p class="truncate font-semibold">{server.name}</p>
												<p class="text-muted-foreground truncate text-xs">{server.ip}</p>
											</div>
										</div>
										{#if supportsQuickPlay}
											<Button
												variant="secondary"
												size="sm"
												class="justify-center"
												onclick={() => handleConnectToServer(server.ip)}
												disabled={!!connectingServer}
											>
												{#if connectingServer === server.ip}
													<Loader2 class="mr-1 h-4 w-4 animate-spin" /> Connecting...
												{:else}
													<Play class="mr-1 h-4 w-4" /> Connect
												{/if}
											</Button>
										{/if}
									</div>
								{/each}
							</div>
						{/if}
					</section>
				{/if}
			</div>
		{:else if currentView === 'screenshots'}
			<InstanceDetailScreenshots instanceId={instance.id} onBack={handleBackToDashboard} />
		{:else if currentView === 'worlds'}
			<InstanceDetailWorlds
				instanceId={instance.id}
				minecraftVersion={instance.minecraftVersion}
				{activeAccountId}
				onBack={handleBackToDashboard}
			/>
		{:else if currentView === 'servers'}
			<InstanceDetailServers
				instanceId={instance.id}
				minecraftVersion={instance.minecraftVersion}
				{activeAccountId}
				onBack={handleBackToDashboard}
			/>
		{/if}
	</div>

	<ScreenshotLightbox
		open={lightboxIndex !== null}
		src={lightboxData}
		filename={lightboxIndex !== null && detail
			? detail.recentScreenshots[lightboxIndex]?.filename
			: undefined}
		isLoading={lightboxLoading}
		canPrev={!!canPrevScreenshot}
		canNext={!!canNextScreenshot}
		onClose={closeScreenshotLightbox}
		onPrev={goPrevScreenshot}
		onNext={goNextScreenshot}
	/>
{/if}
