<script lang="ts">
	import {
		X,
		Download,
		Users,
		Calendar,
		Clock,
		ExternalLink,
		BookOpen,
		Bug,
		Github,
		Copy,
		Check,
		Package,
		Loader2,
		StopCircle,
		Monitor,
		Server,
		MessageCircle,
		Box,
		Sparkles,
		Palette,
		Database,
	} from '@lucide/svelte';
	import { Button } from '$lib/ui/button';
	import * as Select from '$lib/ui/select';
	import { renderMarkdown } from '$lib/utils/markdown';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import DownloadProgress from '$lib/components/DownloadProgress.svelte';
	import ScreenshotLightbox from '$lib/components/ScreenshotLightbox.svelte';
	import TeamMembersSection from '$lib/components/modpack/TeamMembersSection.svelte';
	import type {
		Modpack,
		ModpackVersion,
		ModpackMod,
		ModpackContentType,
		LoaderType,
	} from '$lib/types';

	interface Props {
		modpack: Modpack;
		versions: ModpackVersion[];
		mods?: ModpackMod[];
		isLoadingVersions?: boolean;
		isLoadingDetail?: boolean;
		isLoadingMods?: boolean;
		detailError?: string | null;
		modsError?: string | null;
		installProgress?: {
			stage: string;
			progress: number;
			currentItem?: string;
			totalItems: number;
			completedItems: number;
		} | null;
		isInstalling?: boolean;
		isCancelling?: boolean;
		onClose: () => void;
		onInstall: (versionId: string) => void;
		onCancelInstall?: () => void;
		onLoadMods?: (versionId: string) => void;
	}

	let {
		modpack,
		versions,
		mods = [],
		isLoadingVersions = false,
		isLoadingDetail = false,
		isLoadingMods = false,
		detailError = null,
		modsError = null,
		installProgress = null,
		isInstalling = false,
		isCancelling = false,
		onClose,
		onInstall,
		onCancelInstall,
		onLoadMods,
	}: Props = $props();

	let selectedVersionId = $state<string | null>(null);
	let lightboxIndex = $state<number | null>(null);
	let slugCopied = $state(false);
	let contentTab = $state<'about' | 'changelog' | 'contents'>('about');
	let contentTypeFilter = $state<ModpackContentType | 'all'>('all');

	// Set initial version when versions load
	$effect(() => {
		if (versions.length > 0 && !selectedVersionId) {
			selectedVersionId = versions[0].id;
		}
	});

	let gallery = $derived(modpack.gallery ?? []);
	let selectedVersion = $derived(versions.find((v) => v.id === selectedVersionId) ?? null);

	// Content type counts
	let modCount = $derived(mods.filter((m) => m.contentType === 'mod' || !m.contentType).length);
	let shaderCount = $derived(mods.filter((m) => m.contentType === 'shader').length);
	let resourcePackCount = $derived(mods.filter((m) => m.contentType === 'resourcePack').length);
	let dataPackCount = $derived(mods.filter((m) => m.contentType === 'dataPack').length);

	// Filtered mods based on content type filter
	let filteredMods = $derived.by(() => {
		if (contentTypeFilter === 'all') return mods;
		if (contentTypeFilter === 'mod')
			return mods.filter((m) => m.contentType === 'mod' || !m.contentType);
		return mods.filter((m) => m.contentType === contentTypeFilter);
	});

	// Helper to get icon for content type
	function getContentTypeIcon(type: ModpackContentType | undefined) {
		switch (type) {
			case 'shader':
				return Sparkles;
			case 'resourcePack':
				return Palette;
			case 'dataPack':
				return Database;
			default:
				return Box;
		}
	}

	function formatDownloads(downloads: number): string {
		if (downloads >= 1_000_000) return `${(downloads / 1_000_000).toFixed(1)}M`;
		if (downloads >= 1_000) return `${(downloads / 1_000).toFixed(1)}K`;
		return downloads.toString();
	}

	function formatDate(timestamp?: number): string {
		if (!timestamp) return 'Unknown';
		return new Date(timestamp * 1000).toLocaleDateString();
	}

	function formatBytes(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
	}

	function getLoaderColor(loader: LoaderType): string {
		switch (loader) {
			case 'fabric':
				return 'bg-amber-500/20 text-amber-400';
			case 'forge':
				return 'bg-orange-500/20 text-orange-400';
			case 'neoforge':
				return 'bg-red-500/20 text-red-400';
			case 'quilt':
				return 'bg-purple-500/20 text-purple-400';
			default:
				return 'bg-muted/50 text-muted-foreground';
		}
	}

	function getCategoryColor(category: string): string {
		const colors: Record<string, string> = {
			adventure: 'bg-emerald-500/20 text-emerald-400',
			technology: 'bg-blue-500/20 text-blue-400',
			tech: 'bg-blue-500/20 text-blue-400',
			magic: 'bg-purple-500/20 text-purple-400',
			quests: 'bg-yellow-500/20 text-yellow-400',
			challenging: 'bg-red-500/20 text-red-400',
			hardcore: 'bg-red-500/20 text-red-400',
			lightweight: 'bg-cyan-500/20 text-cyan-400',
			optimization: 'bg-cyan-500/20 text-cyan-400',
			combat: 'bg-orange-500/20 text-orange-400',
			multiplayer: 'bg-indigo-500/20 text-indigo-400',
			'kitchen-sink': 'bg-pink-500/20 text-pink-400',
		};
		const key = category.toLowerCase().replace(/[^a-z-]/g, '');
		return colors[key] || 'bg-muted/50 text-muted-foreground';
	}

	function openLightbox(index: number) {
		lightboxIndex = index;
	}

	function closeLightbox() {
		lightboxIndex = null;
	}

	async function copySlug() {
		await navigator.clipboard.writeText(modpack.slug);
		slugCopied = true;
		setTimeout(() => (slugCopied = false), 2000);
	}

	function formatSideSupport(side?: 'required' | 'optional' | 'unsupported'): {
		label: string;
		color: string;
	} {
		switch (side) {
			case 'required':
				return { label: 'Required', color: 'text-green-400' };
			case 'optional':
				return { label: 'Optional', color: 'text-yellow-400' };
			case 'unsupported':
				return { label: 'Unsupported', color: 'text-red-400' };
			default:
				return { label: 'Unknown', color: 'text-muted-foreground' };
		}
	}

	async function handleDescriptionLinkClick(e: MouseEvent) {
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

	function handleInstall() {
		if (selectedVersionId) {
			onInstall(selectedVersionId);
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
		class="bg-card border-border relative flex max-h-[90vh] w-full max-w-7xl flex-col overflow-hidden border-2 shadow-2xl"
		onclick={(e) => e.stopPropagation()}
	>
		<!-- Close button -->
		<button
			type="button"
			class="bg-card/80 hover:bg-card text-muted-foreground hover:text-foreground border-border absolute top-3 right-3 z-20 border-2 p-1.5 backdrop-blur-sm transition-colors"
			onclick={onClose}
			aria-label="Close modal"
		>
			<X class="h-5 w-5" />
		</button>

		<!-- Scrollable content -->
		<div class="flex-1 overflow-x-hidden overflow-y-auto p-6">
			<!-- Top Section - Single Column Info -->
			<div class="detail-modal-v2-top mb-6">
				<!-- Modpack Info -->
				<div class="flex min-w-0 flex-col gap-4">
					<!-- Icon -->
					<div class="flex justify-center">
						{#if modpack.iconUrl}
							<img
								src={modpack.iconUrl}
								alt={modpack.name}
								class="border-border h-28 w-28 border-2 object-cover"
							/>
						{:else}
							<div
								class="border-border bg-muted flex h-28 w-28 items-center justify-center border-2"
							>
								<Package class="text-muted-foreground/50 h-14 w-14" />
							</div>
						{/if}
					</div>

					<!-- Title as link -->
					{#if modpack.url}
						<button
							type="button"
							class="text-primary text-center text-xl font-bold hover:underline"
							onclick={() => modpack.url && openUrl(modpack.url)}
						>
							{modpack.name}
						</button>
					{:else}
						<h2 class="text-center text-xl font-bold">{modpack.name}</h2>
					{/if}

					<!-- Short description -->
					<p class="text-muted-foreground text-center text-sm">
						{modpack.description}
					</p>

					<!-- Category tags -->
					{#if (modpack.loaders || []).filter((l) => l && l !== 'unknown' && l !== 'vanilla').length > 0 || modpack.categories.length > 0}
						<div class="border-border flex flex-wrap justify-center gap-1.5 border-t pt-4">
							{#each (modpack.loaders || []).filter((l) => l && l !== 'unknown' && l !== 'vanilla') as loader (loader)}
								<span class="category-tag-with-icon {getLoaderColor(loader)}">
									{loader}
								</span>
							{/each}
							{#each modpack.categories.slice(0, 6) as category (category)}
								<span class="category-tag-with-icon {getCategoryColor(category)}">
									{category}
								</span>
							{/each}
						</div>
					{/if}

					<!-- Version selector and Install -->
					<div class="border-border space-y-4 border-t pt-4">
						{#if isLoadingVersions}
							<div class="text-muted-foreground flex items-center justify-center gap-2 text-sm">
								<Loader2 class="h-4 w-4 animate-spin" />
								Loading versions...
							</div>
						{:else if versions.length > 0}
							<Select.Root
								type="single"
								value={selectedVersionId ?? ''}
								onValueChange={(v) => (selectedVersionId = v)}
							>
								<Select.Trigger
									class="border-border bg-background mx-auto h-9 w-full max-w-[280px] overflow-hidden border-2"
								>
									<span class="truncate">
										{#if selectedVersion}
											{selectedVersion.name} - MC {selectedVersion.mcVersion}
										{:else}
											Select version
										{/if}
									</span>
								</Select.Trigger>
								<Select.Content class="border-border bg-card z-[70] max-h-[300px] border-2">
									{#each versions as version (version.id)}
										<Select.Item value={version.id} label={version.name}>
											<div class="flex flex-col">
												<span class="text-sm">{version.name}</span>
												<span class="text-muted-foreground text-xs">
													MC {version.mcVersion} &bull; {version.loaderType}
													{#if version.releasedAt}
														&bull; {formatDate(version.releasedAt)}
													{/if}
												</span>
											</div>
										</Select.Item>
									{/each}
								</Select.Content>
							</Select.Root>
						{:else}
							<p class="text-muted-foreground text-center text-sm">No versions available</p>
						{/if}

						<!-- Install Button -->
						{#if !isInstalling}
							<div class="flex justify-center">
								<Button
									class="install-button-large"
									disabled={!selectedVersionId || isInstalling}
									onclick={handleInstall}
								>
									<Download class="mr-2 h-5 w-5" />
									INSTALL
								</Button>
							</div>
						{/if}
					</div>

					<!-- Stats row -->
					<div class="stats-grid">
						<div class="flex flex-col items-center gap-1">
							<Download class="text-muted-foreground h-4 w-4" />
							<span class="text-sm font-bold">{formatDownloads(modpack.downloads)}</span>
							<span class="text-muted-foreground text-xs">Downloads</span>
						</div>
						<div class="flex flex-col items-center gap-1">
							<Users class="text-muted-foreground h-4 w-4" />
							<span class="text-sm font-bold"
								>{modpack.followers != null ? formatDownloads(modpack.followers) : '-'}</span
							>
							<span class="text-muted-foreground text-xs">Followers</span>
						</div>
						<div class="flex flex-col items-center gap-1">
							<Calendar class="text-muted-foreground h-4 w-4" />
							<span class="text-sm font-bold">{formatDate(modpack.createdAt)}</span>
							<span class="text-muted-foreground text-xs">Created</span>
						</div>
						<div class="flex flex-col items-center gap-1">
							<Clock class="text-muted-foreground h-4 w-4" />
							<span class="text-sm font-bold">{formatDate(modpack.updatedAt)}</span>
							<span class="text-muted-foreground text-xs">Updated</span>
						</div>
					</div>
				</div>

				<!-- Horizontal Gallery Carousel -->
				{#if gallery.length > 0}
					<div class="gallery-carousel-container mt-4">
						<div class="gallery-carousel-scroll">
							{#each gallery as image, idx (image.rawUrl ?? image.url)}
								<button
									type="button"
									class="gallery-carousel-item"
									onclick={() => openLightbox(idx)}
								>
									<img
										src={image.rawUrl ?? image.url}
										alt={image.title ?? `Gallery image ${idx + 1}`}
										loading={idx < 3 ? 'eager' : 'lazy'}
									/>
								</button>
							{/each}
						</div>
					</div>
				{/if}
			</div>

			<!-- Bottom Section - Two Columns -->
			<div class="detail-modal-v2-bottom">
				<!-- Left Sidebar -->
				<div class="min-w-0 space-y-6">
					<!-- External Resources -->
					<div>
						<h3 class="mb-3 text-sm font-bold tracking-wide uppercase">External Resources</h3>
						<div class="external-resources-list">
							{#if modpack.url}
								<button
									type="button"
									class="external-resource-link"
									onclick={() => modpack.url && openUrl(modpack.url)}
								>
									<ExternalLink class="h-4 w-4" />
									<span class="text-sm">View on {modpack.platform}</span>
								</button>
							{/if}
							{#if modpack.externalLinks?.discordUrl}
								<button
									type="button"
									class="external-resource-link"
									onclick={() =>
										modpack.externalLinks?.discordUrl && openUrl(modpack.externalLinks.discordUrl)}
								>
									<MessageCircle class="h-4 w-4" />
									<span class="text-sm">Discord</span>
								</button>
							{/if}
							{#if modpack.externalLinks?.wikiUrl}
								<button
									type="button"
									class="external-resource-link"
									onclick={() =>
										modpack.externalLinks?.wikiUrl && openUrl(modpack.externalLinks.wikiUrl)}
								>
									<BookOpen class="h-4 w-4" />
									<span class="text-sm">Wiki</span>
								</button>
							{:else}
								<button type="button" class="external-resource-link opacity-50" disabled>
									<BookOpen class="h-4 w-4" />
									<span class="text-sm">Wiki</span>
								</button>
							{/if}
							{#if modpack.externalLinks?.issuesUrl}
								<button
									type="button"
									class="external-resource-link"
									onclick={() =>
										modpack.externalLinks?.issuesUrl && openUrl(modpack.externalLinks.issuesUrl)}
								>
									<Bug class="h-4 w-4" />
									<span class="text-sm">Issues</span>
								</button>
							{:else}
								<button type="button" class="external-resource-link opacity-50" disabled>
									<Bug class="h-4 w-4" />
									<span class="text-sm">Issues</span>
								</button>
							{/if}
							{#if modpack.externalLinks?.sourceUrl}
								<button
									type="button"
									class="external-resource-link"
									onclick={() =>
										modpack.externalLinks?.sourceUrl && openUrl(modpack.externalLinks.sourceUrl)}
								>
									<Github class="h-4 w-4" />
									<span class="text-sm">Source</span>
								</button>
							{:else}
								<button type="button" class="external-resource-link opacity-50" disabled>
									<Github class="h-4 w-4" />
									<span class="text-sm">Source</span>
								</button>
							{/if}
						</div>
					</div>

					<!-- Project Members -->
					<div>
						<h3 class="mb-3 text-sm font-bold tracking-wide uppercase">Project Members</h3>
						<TeamMembersSection
							members={modpack.teamMembers ?? []}
							authorFallback={modpack.author}
						/>
					</div>

					<!-- Environment Support -->
					{#if modpack.clientSide || modpack.serverSide}
						<div>
							<h3 class="mb-3 text-sm font-bold tracking-wide uppercase">Environment</h3>
							<div class="space-y-2">
								<div class="flex items-center gap-2">
									<Monitor class="text-muted-foreground h-4 w-4" />
									<span class="text-sm">Client:</span>
									<span class="text-sm {formatSideSupport(modpack.clientSide).color}">
										{formatSideSupport(modpack.clientSide).label}
									</span>
								</div>
								<div class="flex items-center gap-2">
									<Server class="text-muted-foreground h-4 w-4" />
									<span class="text-sm">Server:</span>
									<span class="text-sm {formatSideSupport(modpack.serverSide).color}">
										{formatSideSupport(modpack.serverSide).label}
									</span>
								</div>
							</div>
						</div>
					{/if}

					<!-- Technical Information -->
					<div>
						<h3 class="mb-3 text-sm font-bold tracking-wide uppercase">Technical Information</h3>
						<div class="technical-info-grid">
							<span class="technical-info-label">Created</span>
							<span class="technical-info-value">{formatDate(modpack.createdAt)}</span>

							<span class="technical-info-label">Updated</span>
							<span class="technical-info-value">{formatDate(modpack.updatedAt)}</span>

							{#if selectedVersion}
								<span class="technical-info-label">Released</span>
								<span class="technical-info-value">{formatDate(selectedVersion.releasedAt)}</span>

								{#if selectedVersion.files.length > 0}
									<span class="technical-info-label">Size</span>
									<span class="technical-info-value">
										{formatBytes(selectedVersion.files.reduce((sum, f) => sum + f.size, 0))}
									</span>
								{/if}
							{/if}

							<span class="technical-info-label">Slug</span>
							<span class="technical-info-value flex items-center gap-1">
								<span class="truncate">{modpack.slug}</span>
								<button
									type="button"
									class="text-muted-foreground hover:text-foreground flex-shrink-0"
									onclick={copySlug}
									aria-label="Copy slug"
								>
									{#if slugCopied}
										<Check class="h-3 w-3" />
									{:else}
										<Copy class="h-3 w-3" />
									{/if}
								</button>
							</span>
						</div>
					</div>
				</div>

				<!-- Right: Description/Changelog Content -->
				<div class="flex min-w-0 flex-col">
					<!-- Tab Buttons -->
					<div class="mb-3 flex gap-2">
						<Button
							variant={contentTab === 'about' ? 'default' : 'secondary'}
							size="sm"
							onclick={() => (contentTab = 'about')}
						>
							About
						</Button>
						<Button
							variant={contentTab === 'changelog' ? 'default' : 'secondary'}
							size="sm"
							onclick={() => (contentTab = 'changelog')}
						>
							Changelog
						</Button>
						<Button
							variant={contentTab === 'contents' ? 'default' : 'secondary'}
							size="sm"
							onclick={() => {
								contentTab = 'contents';
								if (selectedVersionId && onLoadMods && mods.length === 0 && !isLoadingMods) {
									onLoadMods(selectedVersionId);
								}
							}}
						>
							Contents {mods.length > 0 ? `(${mods.length})` : ''}
						</Button>
					</div>

					<!-- Tab Content -->
					<div class="border-border bg-background/50 min-h-[300px] border-2 p-4">
						{#if contentTab === 'about'}
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
								<div class="prose-markdown" onclick={handleDescriptionLinkClick}>
									<!-- eslint-disable-next-line svelte/no-at-html-tags -->
									{@html renderMarkdown(modpack.body, modpack.description)}
								</div>
							{:else}
								<p class="text-muted-foreground text-sm">No description available.</p>
							{/if}
						{:else if contentTab === 'changelog'}
							<!-- Changelog Tab -->
							{#if isLoadingVersions}
								<div class="text-muted-foreground flex items-center gap-2 text-sm">
									<Loader2 class="h-4 w-4 animate-spin" />
									Loading changelog...
								</div>
							{:else if versions.length === 0}
								<p class="text-muted-foreground text-sm">No version history available.</p>
							{:else}
								<div class="space-y-4">
									{#each versions.slice(0, 10) as version (version.id)}
										<div class="border-border border-b pb-4 last:border-b-0">
											<div class="mb-2 flex items-center gap-2">
												<span class="font-bold">{version.name}</span>
												<span class="bg-muted text-muted-foreground rounded px-2 py-0.5 text-xs">
													MC {version.mcVersion}
												</span>
												<span
													class="rounded px-2 py-0.5 text-xs {getLoaderColor(version.loaderType)}"
												>
													{version.loaderType}
												</span>
											</div>
											{#if version.releasedAt}
												<p class="text-muted-foreground mb-2 text-xs">
													Released {formatDate(version.releasedAt)}
												</p>
											{/if}
											{#if version.changelog}
												<!-- svelte-ignore a11y_click_events_have_key_events -->
												<!-- svelte-ignore a11y_no_static_element_interactions -->
												<div class="prose-markdown text-sm" onclick={handleDescriptionLinkClick}>
													<!-- eslint-disable-next-line svelte/no-at-html-tags -->
													{@html renderMarkdown(version.changelog)}
												</div>
											{:else}
												<p class="text-muted-foreground text-sm italic">No changelog provided</p>
											{/if}
										</div>
									{/each}
									{#if versions.length > 10}
										<p class="text-muted-foreground text-center text-sm">
											Showing 10 of {versions.length} versions
										</p>
									{/if}
								</div>
							{/if}
						{:else}
							<!-- Contents Tab -->
							{#if isLoadingMods}
								<div class="text-muted-foreground flex items-center gap-2 text-sm">
									<Loader2 class="h-4 w-4 animate-spin" />
									Loading contents...
								</div>
							{:else if modsError}
								<div
									class="bg-destructive/10 border-destructive text-destructive rounded border-2 p-3 text-sm"
								>
									{modsError}
								</div>
							{:else if mods.length === 0}
								<div class="flex flex-col items-center justify-center py-12 text-center">
									<Package class="text-muted-foreground/30 mb-4 h-12 w-12" />
									<p class="text-muted-foreground text-sm">
										{#if !selectedVersionId}
											Select a version to view contents
										{:else}
											No content information available for this modpack
										{/if}
									</p>
									{#if selectedVersionId && onLoadMods}
										<Button
											variant="secondary"
											size="sm"
											class="mt-4"
											onclick={() =>
												selectedVersionId && onLoadMods && onLoadMods(selectedVersionId)}
										>
											Load Contents
										</Button>
									{/if}
								</div>
							{:else}
								<!-- Content Type Filter -->
								<div class="mb-4 flex flex-wrap gap-2">
									<button
										type="button"
										class="content-filter-btn"
										class:active={contentTypeFilter === 'all'}
										onclick={() => (contentTypeFilter = 'all')}
									>
										All ({mods.length})
									</button>
									{#if modCount > 0}
										<button
											type="button"
											class="content-filter-btn"
											class:active={contentTypeFilter === 'mod'}
											onclick={() => (contentTypeFilter = 'mod')}
										>
											<Box class="h-3.5 w-3.5" />
											Mods ({modCount})
										</button>
									{/if}
									{#if shaderCount > 0}
										<button
											type="button"
											class="content-filter-btn"
											class:active={contentTypeFilter === 'shader'}
											onclick={() => (contentTypeFilter = 'shader')}
										>
											<Sparkles class="h-3.5 w-3.5" />
											Shaders ({shaderCount})
										</button>
									{/if}
									{#if resourcePackCount > 0}
										<button
											type="button"
											class="content-filter-btn"
											class:active={contentTypeFilter === 'resourcePack'}
											onclick={() => (contentTypeFilter = 'resourcePack')}
										>
											<Palette class="h-3.5 w-3.5" />
											Resource Packs ({resourcePackCount})
										</button>
									{/if}
									{#if dataPackCount > 0}
										<button
											type="button"
											class="content-filter-btn"
											class:active={contentTypeFilter === 'dataPack'}
											onclick={() => (contentTypeFilter = 'dataPack')}
										>
											<Database class="h-3.5 w-3.5" />
											Data Packs ({dataPackCount})
										</button>
									{/if}
								</div>

								<!-- Contents List -->
								<div class="contents-list">
									{#each filteredMods as mod (mod.id)}
										{@const ContentIcon = getContentTypeIcon(mod.contentType)}
										<div class="content-item">
											{#if mod.iconUrl}
												<img src={mod.iconUrl} alt="" class="content-item-icon" />
											{:else}
												<div class="content-item-icon-placeholder">
													<ContentIcon class="text-muted-foreground h-5 w-5" />
												</div>
											{/if}
											<div class="content-item-info">
												{#if mod.url}
													<button
														type="button"
														class="content-item-name hover:text-primary"
														onclick={() => mod.url && openUrl(mod.url)}
													>
														{mod.name}
													</button>
												{:else}
													<span class="content-item-name">{mod.name}</span>
												{/if}
												{#if mod.author}
													<span class="content-item-author">by {mod.author}</span>
												{/if}
											</div>
										</div>
									{/each}
								</div>

								{#if filteredMods.length > 0}
									<p class="text-muted-foreground mt-4 text-center text-xs">
										Showing {filteredMods.length} of {mods.length} items
									</p>
								{/if}
							{/if}
						{/if}
					</div>
				</div>
			</div>
		</div>

		<!-- Sticky Install Progress -->
		{#if isInstalling}
			<div
				class="border-border bg-card flex-shrink-0 border-t p-4 shadow-[0_-4px_6px_-1px_rgba(0,0,0,0.1)]"
			>
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
								<Loader2 class="text-primary h-4 w-4 flex-shrink-0 animate-spin" />
								<span class="font-medium">Starting installation...</span>
							</div>
						{/if}
					</div>
					{#if onCancelInstall}
						<Button
							variant="destructive"
							size="sm"
							onclick={onCancelInstall}
							disabled={isCancelling}
						>
							<StopCircle class="mr-1 h-4 w-4" />
							Cancel
						</Button>
					{/if}
				</div>
			</div>
		{/if}
	</div>
</div>

<!-- Lightbox -->
<ScreenshotLightbox
	open={lightboxIndex !== null}
	src={lightboxIndex !== null
		? (gallery[lightboxIndex]?.rawUrl ?? gallery[lightboxIndex]?.url ?? null)
		: null}
	filename={lightboxIndex !== null
		? (gallery[lightboxIndex]?.title ?? `Image ${lightboxIndex + 1}`)
		: undefined}
	canPrev={lightboxIndex !== null && lightboxIndex > 0}
	canNext={lightboxIndex !== null && lightboxIndex < gallery.length - 1}
	onClose={closeLightbox}
	onPrev={() => lightboxIndex !== null && (lightboxIndex = Math.max(0, lightboxIndex - 1))}
	onNext={() =>
		lightboxIndex !== null && (lightboxIndex = Math.min(gallery.length - 1, lightboxIndex + 1))}
/>
