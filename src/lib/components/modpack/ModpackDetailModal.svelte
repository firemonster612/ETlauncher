<script lang="ts">
	import {
		Package,
		Download,
		ExternalLink,
		Loader2,
		X,
		Maximize2,
		StopCircle,
		Calendar,
		User,
		Tag,
		HardDrive,
	} from '@lucide/svelte';
	import { renderMarkdown } from '$lib/utils/markdown';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import { Button } from '$lib/ui/button';
	import * as Select from '$lib/ui/select';
	import DownloadProgress from '$lib/components/DownloadProgress.svelte';
	import ModpackGalleryCarousel from './ModpackGalleryCarousel.svelte';
	import type {
		Modpack,
		ModpackVersion,
		ModpackMod,
		ModpackPlatform,
		LoaderType,
	} from '$lib/types';

	interface Props {
		modpack: Modpack;
		versions: ModpackVersion[];
		selectedVersionId: string | null;
		onVersionChange: (versionId: string) => void;
		onInstall: (versionId: string) => void;
		onClose: () => void;
		onExpandDescription: () => void;
		onOpenLightbox: (index: number) => void;
		// Loading states
		isLoadingDetail?: boolean;
		isLoadingVersions?: boolean;
		detailError?: string | null;
		// Mods
		mods?: ModpackMod[];
		isLoadingMods?: boolean;
		modsError?: string | null;
		// Install progress
		isInstalling?: boolean;
		installProgress?: {
			stage: string;
			progress: number;
			currentItem?: string;
			totalItems: number;
			completedItems: number;
		} | null;
		isCancelling?: boolean;
		onCancelInstall?: () => void;
	}

	let {
		modpack,
		versions,
		selectedVersionId,
		onVersionChange,
		onInstall,
		onClose,
		onExpandDescription,
		onOpenLightbox,
		isLoadingDetail = false,
		isLoadingVersions = false,
		detailError = null,
		mods = [],
		isLoadingMods = false,
		modsError = null,
		isInstalling = false,
		installProgress = null,
		isCancelling = false,
		onCancelInstall,
	}: Props = $props();

	let activeTab = $state<'about' | 'gallery' | 'mods' | 'changelog'>('about');

	// Selected version object
	let selectedVersion = $derived(versions.find((v) => v.id === selectedVersionId) ?? null);
	let totalSize = $derived(selectedVersion?.files.reduce((sum, f) => sum + f.size, 0) ?? 0);

	function formatDownloads(downloads: number): string {
		if (downloads >= 1_000_000) return `${(downloads / 1_000_000).toFixed(1)}M`;
		if (downloads >= 1_000) return `${(downloads / 1_000).toFixed(1)}K`;
		return downloads.toString();
	}

	function formatBytes(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
	}

	function formatDate(timestamp?: number): string {
		if (!timestamp) return 'Unknown';
		return new Date(timestamp * 1000).toLocaleDateString();
	}

	function getPlatformColor(platform: ModpackPlatform): string {
		switch (platform) {
			case 'modrinth':
				return 'bg-green-500/20 text-green-500 border-green-500/50';
			case 'curseforge':
				return 'bg-orange-500/20 text-orange-500 border-orange-500/50';
			case 'ftb':
				return 'bg-blue-500/20 text-blue-500 border-blue-500/50';
			case 'technic':
				return 'bg-yellow-500/20 text-yellow-500 border-yellow-500/50';
			case 'atlauncher':
				return 'bg-purple-500/20 text-purple-500 border-purple-500/50';
			default:
				return 'bg-muted text-muted-foreground border-muted';
		}
	}

	function getLoaderColor(loader: LoaderType): string {
		switch (loader) {
			case 'fabric':
				return 'bg-amber-500/20 text-amber-500';
			case 'forge':
				return 'bg-orange-500/20 text-orange-500';
			case 'neoforge':
				return 'bg-red-500/20 text-red-500';
			case 'quilt':
				return 'bg-purple-500/20 text-purple-500';
			default:
				return 'bg-muted/50 text-muted-foreground';
		}
	}

	function isModListSupported(platform: ModpackPlatform): boolean {
		return ['modrinth', 'curseforge', 'ftb', 'technic'].includes(platform);
	}

	async function handleLinkClick(e: MouseEvent) {
		const target = e.target as HTMLElement | null;
		const anchor = target?.closest('a') as HTMLAnchorElement | null;
		if (!anchor) return;

		const href = anchor.getAttribute('href');
		if (!href || href.startsWith('#')) return;

		e.preventDefault();
		e.stopPropagation();
		try {
			await openUrl(href);
		} catch (err) {
			console.error('Failed to open URL:', href, err);
		}
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="fixed inset-x-0 top-[var(--titlebar-height)] z-50 flex h-[calc(100vh-var(--titlebar-height))] items-center justify-center overflow-hidden bg-black/50 p-4"
	onclick={onClose}
>
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="bg-card border-border flex max-h-[90vh] w-full max-w-6xl flex-col overflow-hidden rounded-lg border-2 shadow-2xl"
		onclick={(e) => e.stopPropagation()}
	>
		<!-- Header banner using modpack image with fade-out effect -->
		{#if modpack.bannerUrl || modpack.iconUrl}
			<div class="relative h-40 flex-shrink-0 overflow-hidden">
				<img
					src={modpack.bannerUrl || modpack.iconUrl}
					alt=""
					class="h-full w-full scale-110 object-cover blur-sm"
				/>
				<div class="from-card via-card/80 absolute inset-0 bg-gradient-to-t to-transparent" />
			</div>
		{/if}

		<!-- Main Header -->
		<div
			class="border-border flex-shrink-0 border-b p-4 {modpack.bannerUrl || modpack.iconUrl
				? 'relative -mt-16'
				: ''}"
		>
			<div class="flex gap-4">
				{#if modpack.iconUrl}
					<img src={modpack.iconUrl} alt={modpack.name} class="h-20 w-20 rounded object-cover" />
				{:else}
					<div class="bg-muted flex h-20 w-20 items-center justify-center rounded">
						<Package class="text-muted-foreground/50 h-10 w-10" />
					</div>
				{/if}
				<div class="flex-1">
					<div class="flex items-start justify-between gap-2">
						<div>
							<h2 class="text-xl font-bold">{modpack.name}</h2>
							<p class="text-muted-foreground text-sm">{modpack.author}</p>
						</div>
						<button class="text-muted-foreground hover:text-foreground" onclick={onClose}>
							<X class="h-5 w-5" />
						</button>
					</div>
					<div class="mt-2 flex flex-wrap items-center gap-2">
						<span class="rounded border px-1.5 py-0.5 text-xs {getPlatformColor(modpack.platform)}">
							{modpack.platform}
						</span>
						{#each (modpack.loaders || []).filter((l) => l && l !== 'unknown' && l !== 'vanilla') as loader (loader)}
							<span class="rounded px-1.5 py-0.5 text-xs {getLoaderColor(loader)}">
								{loader}
							</span>
						{/each}
						<span class="text-muted-foreground flex items-center gap-1 text-xs">
							<Download class="h-3 w-3" />
							{formatDownloads(modpack.downloads)}
						</span>
					</div>
				</div>
			</div>
		</div>

		<!-- Two-column layout -->
		<div class="modpack-detail-modal min-h-0 flex-1 overflow-y-auto p-5">
			<!-- Left Column: Main Content -->
			<div class="space-y-4">
				<!-- Gallery Carousel -->
				{#if (modpack.gallery?.length ?? 0) > 0}
					<ModpackGalleryCarousel images={modpack.gallery ?? []} onImageClick={onOpenLightbox} />
				{/if}

				<!-- Tabs -->
				<div class="flex items-center gap-2">
					<Button
						size="sm"
						variant={activeTab === 'about' ? 'default' : 'secondary'}
						onclick={() => (activeTab = 'about')}
					>
						About
					</Button>
					<Button
						size="sm"
						variant={activeTab === 'gallery' ? 'default' : 'secondary'}
						disabled={(modpack.gallery?.length ?? 0) === 0}
						onclick={() => (activeTab = 'gallery')}
					>
						Gallery
					</Button>
					<Button
						size="sm"
						variant={activeTab === 'mods' ? 'default' : 'secondary'}
						disabled={!selectedVersionId || !isModListSupported(modpack.platform)}
						onclick={() => (activeTab = 'mods')}
					>
						Mods
					</Button>
					<Button
						size="sm"
						variant={activeTab === 'changelog' ? 'default' : 'secondary'}
						disabled={versions.length === 0}
						onclick={() => (activeTab = 'changelog')}
					>
						Changelog
					</Button>
				</div>

				<!-- Tab Content -->
				{#if activeTab === 'about'}
					<div class="border-border bg-background/70 space-y-2 rounded-lg border-2 p-4">
						<div class="flex items-center justify-between gap-2">
							<h3 class="text-sm font-semibold">About</h3>
							<Button
								size="sm"
								variant="secondary"
								onclick={onExpandDescription}
								disabled={!modpack.body && !modpack.description}
							>
								<Maximize2 class="mr-1 h-4 w-4" />
								Expand
							</Button>
						</div>
						{#if isLoadingDetail}
							<div class="text-muted-foreground flex items-center gap-2 text-sm">
								<Loader2 class="h-4 w-4 animate-spin" />
								Loading description...
							</div>
						{:else if detailError}
							<div
								class="bg-destructive/10 border-destructive text-destructive rounded border-2 p-3 text-sm"
							>
								{detailError}
							</div>
						{:else if modpack.body || modpack.description}
							<!-- svelte-ignore a11y_click_events_have_key_events -->
							<!-- svelte-ignore a11y_no_static_element_interactions -->
							<div class="prose-markdown max-h-[400px] overflow-y-auto" onclick={handleLinkClick}>
								<!-- eslint-disable-next-line svelte/no-at-html-tags -->
								{@html renderMarkdown(modpack.body, modpack.description)}
							</div>
						{:else}
							<p class="text-muted-foreground text-sm">No description available.</p>
						{/if}
					</div>
				{:else if activeTab === 'gallery'}
					{#if (modpack.gallery?.length ?? 0) > 0}
						<div class="grid gap-3 sm:grid-cols-2">
							{#each modpack.gallery ?? [] as image, idx (image.rawUrl ?? image.url)}
								<button
									type="button"
									class="border-border bg-muted/50 relative aspect-video cursor-pointer overflow-hidden rounded-lg border-2"
									onclick={() => onOpenLightbox(idx)}
								>
									<img
										src={image.rawUrl ?? image.url}
										alt={image.title ?? modpack.name}
										class="h-full w-full object-cover"
										loading="lazy"
									/>
								</button>
							{/each}
						</div>
					{:else}
						<p class="text-muted-foreground text-sm">No gallery available.</p>
					{/if}
				{:else if activeTab === 'mods'}
					{#if !isModListSupported(modpack.platform)}
						<p class="text-muted-foreground text-sm">Mod list not available for this platform.</p>
					{:else if !selectedVersionId}
						<p class="text-muted-foreground text-sm">Select a version to view its mod list.</p>
					{:else if isLoadingMods}
						<div class="text-muted-foreground flex items-center gap-2 text-sm">
							<Loader2 class="h-4 w-4 animate-spin" />
							Loading mod list...
						</div>
					{:else if modsError}
						<div
							class="bg-destructive/10 border-destructive text-destructive rounded border-2 p-3 text-sm"
						>
							{modsError}
						</div>
					{:else if mods.length === 0}
						<p class="text-muted-foreground text-sm">No mods found for this version.</p>
					{:else}
						<div class="border-border overflow-hidden rounded-lg border-2">
							<div class="max-h-[400px] overflow-y-auto">
								{#each mods as mod (mod.id)}
									<button
										type="button"
										class="hover:bg-muted/50 border-border flex w-full items-center gap-3 border-b p-3 text-left last:border-b-0"
										onclick={() => mod.url && openUrl(mod.url)}
										disabled={!mod.url}
									>
										{#if mod.iconUrl}
											<img
												src={mod.iconUrl}
												alt={mod.name}
												class="h-8 w-8 rounded object-cover"
												loading="lazy"
											/>
										{:else}
											<div
												class="bg-muted text-muted-foreground flex h-8 w-8 items-center justify-center rounded text-xs"
											>
												MOD
											</div>
										{/if}
										<div class="min-w-0">
											<div class="truncate text-sm font-medium">{mod.name}</div>
											{#if mod.author}
												<div class="text-muted-foreground truncate text-xs">{mod.author}</div>
											{/if}
										</div>
									</button>
								{/each}
							</div>
						</div>
					{/if}
				{:else if activeTab === 'changelog'}
					<div
						class="border-border bg-background/70 max-h-[400px] space-y-4 overflow-y-auto rounded-lg border-2 p-4"
					>
						{#each versions as version (version.id)}
							<div class="border-border border-b pb-4 last:border-b-0 last:pb-0">
								<div class="mb-2 flex flex-wrap items-center gap-2">
									<span class="font-medium">{version.name}</span>
									<span class="text-muted-foreground text-xs">
										MC {version.mcVersion} &bull; {version.loaderType}
									</span>
									{#if version.releasedAt}
										<span class="text-muted-foreground text-xs">
											&bull; {formatDate(version.releasedAt)}
										</span>
									{/if}
								</div>
								{#if version.changelog}
									<!-- svelte-ignore a11y_click_events_have_key_events -->
									<!-- svelte-ignore a11y_no_static_element_interactions -->
									<div class="prose-markdown text-sm" onclick={handleLinkClick}>
										<!-- eslint-disable-next-line svelte/no-at-html-tags -->
										{@html renderMarkdown(version.changelog)}
									</div>
								{:else}
									<p class="text-muted-foreground text-sm italic">No changelog available.</p>
								{/if}
							</div>
						{/each}
					</div>
				{/if}
			</div>

			<!-- Right Column: Sidebar Info -->
			<div class="space-y-4">
				<!-- Version Selector -->
				<div class="border-border bg-background/70 space-y-3 rounded-lg border-2 p-4">
					<h3 class="text-sm font-semibold">Version</h3>
					{#if isLoadingVersions}
						<div class="text-muted-foreground flex items-center gap-2 text-sm">
							<Loader2 class="h-4 w-4 animate-spin" />
							Loading...
						</div>
					{:else if versions.length > 0}
						<Select.Root
							type="single"
							value={selectedVersionId ?? ''}
							onValueChange={(v) => onVersionChange(v)}
						>
							<Select.Trigger class="border-border bg-background w-full border-2 text-sm">
								{#if selectedVersion}
									{selectedVersion.name}
								{:else}
									Select version
								{/if}
							</Select.Trigger>
							<Select.Content class="border-border bg-card z-[70] max-h-[200px] border-2">
								{#each versions as version (version.id)}
									<Select.Item value={version.id} label={version.name}>
										<div class="flex flex-col">
											<span class="text-sm">{version.name}</span>
											<span class="text-muted-foreground text-xs">
												MC {version.mcVersion}
											</span>
										</div>
									</Select.Item>
								{/each}
							</Select.Content>
						</Select.Root>
					{:else}
						<p class="text-muted-foreground text-sm">No versions available</p>
					{/if}
				</div>

				<!-- Version Details -->
				{#if selectedVersion}
					<div class="border-border bg-background/70 space-y-2 rounded-lg border-2 p-4">
						<h3 class="text-sm font-semibold">Details</h3>
						<div class="space-y-2 text-sm">
							<div class="flex items-center gap-2">
								<Tag class="text-muted-foreground h-4 w-4" />
								<span class="text-muted-foreground">MC:</span>
								<span>{selectedVersion.mcVersion}</span>
							</div>
							<div class="flex items-center gap-2">
								<Package class="text-muted-foreground h-4 w-4" />
								<span class="text-muted-foreground">Loader:</span>
								<span class="capitalize"
									>{selectedVersion.loaderType}
									{selectedVersion.loaderVersion ?? ''}</span
								>
							</div>
							{#if selectedVersion.releasedAt}
								<div class="flex items-center gap-2">
									<Calendar class="text-muted-foreground h-4 w-4" />
									<span class="text-muted-foreground">Released:</span>
									<span>{formatDate(selectedVersion.releasedAt)}</span>
								</div>
							{/if}
							{#if totalSize > 0}
								<div class="flex items-center gap-2">
									<HardDrive class="text-muted-foreground h-4 w-4" />
									<span class="text-muted-foreground">Size:</span>
									<span>{formatBytes(totalSize)}</span>
								</div>
							{/if}
						</div>
					</div>
				{/if}

				<!-- External Links -->
				{#if modpack.url}
					<div class="border-border bg-background/70 space-y-2 rounded-lg border-2 p-4">
						<h3 class="text-sm font-semibold">Links</h3>
						<a
							href={modpack.url}
							target="_blank"
							rel="noopener noreferrer"
							class="bg-muted hover:bg-muted/80 inline-flex items-center gap-1.5 rounded px-2.5 py-1.5 text-xs transition-colors"
						>
							<ExternalLink class="h-3.5 w-3.5" />
							View on {modpack.platform}
						</a>
					</div>
				{/if}

				<!-- Author & Categories -->
				<div class="border-border bg-background/70 space-y-3 rounded-lg border-2 p-4">
					<div class="flex items-center gap-2">
						<User class="text-muted-foreground h-4 w-4" />
						<span class="text-muted-foreground text-sm">Author:</span>
						<span class="text-sm">{modpack.author}</span>
					</div>
					{#if modpack.categories?.length > 0}
						<div>
							<div class="text-muted-foreground mb-1.5 flex items-center gap-2 text-sm">
								<Tag class="h-4 w-4" />
								Categories:
							</div>
							<div class="flex flex-wrap gap-1.5">
								{#each modpack.categories.slice(0, 6) as category (category)}
									<span class="bg-muted text-muted-foreground rounded px-1.5 py-0.5 text-xs">
										{category}
									</span>
								{/each}
								{#if modpack.categories.length > 6}
									<span class="text-muted-foreground text-xs">+{modpack.categories.length - 6}</span
									>
								{/if}
							</div>
						</div>
					{/if}
				</div>
			</div>
		</div>

		<!-- Footer -->
		{#if !isInstalling}
			<div class="border-border bg-card flex flex-shrink-0 justify-end gap-3 border-t p-4">
				<Button variant="outline" onclick={onClose}>Cancel</Button>
				<Button
					disabled={!selectedVersionId}
					onclick={() => selectedVersionId && onInstall(selectedVersionId)}
				>
					<Download class="mr-2 h-4 w-4" />
					Install
				</Button>
			</div>
		{:else}
			<div class="border-border bg-card flex-shrink-0 border-t p-4">
				<div class="flex items-center gap-3">
					<div class="flex-1">
						{#if installProgress}
							<DownloadProgress
								stage={installProgress.stage}
								progress={installProgress.progress}
								currentItem={installProgress.currentItem}
								totalItems={installProgress.totalItems}
								completedItems={installProgress.completedItems}
							/>
						{:else}
							<div class="flex items-center gap-2 text-sm">
								<Loader2 class="text-primary h-4 w-4 animate-spin" />
								<span>Starting installation...</span>
							</div>
						{/if}
					</div>
					<Button variant="destructive" size="sm" onclick={onCancelInstall} disabled={isCancelling}>
						<StopCircle class="mr-1 h-4 w-4" />
						Cancel
					</Button>
				</div>
			</div>
		{/if}
	</div>
</div>
