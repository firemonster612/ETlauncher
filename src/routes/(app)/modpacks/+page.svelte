<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import {
		Package,
		Search,
		Download,
		ExternalLink,
		Loader2,
		ArrowUpDown,
		Filter,
		X,
		ChevronDown,
		Maximize2,
		StopCircle,
		Check,
	} from '@lucide/svelte';
	import { renderMarkdown } from '$lib/utils/markdown';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import { ask } from '@tauri-apps/plugin-dialog';
	import { Button } from '$lib/ui/button';
	import { Checkbox } from '$lib/ui/checkbox';
	import { Input } from '$lib/ui/input';
	import { Skeleton } from '$lib/ui/skeleton';
	import * as Select from '$lib/ui/select';
	import { modpacksStore } from '$lib/stores/modpacks.svelte';
	import { modpackInstallStore } from '$lib/stores/modpackInstall.svelte';
	import { versionsStore } from '$lib/stores/versions.svelte';
	import DownloadProgress from '$lib/components/DownloadProgress.svelte';
	import ScreenshotLightbox from '$lib/components/ScreenshotLightbox.svelte';
	import DescriptionModal from '$lib/components/DescriptionModal.svelte';
	import * as modpackService from '$lib/services/modpack';
	import type { Modpack, ModpackMod, ModpackPlatform, ModpackSortBy, LoaderType } from '$lib/types';

	// Category options per platform
	const modrinthCategories = [
		'adventure',
		'challenging',
		'combat',
		'kitchen-sink',
		'lightweight',
		'magic',
		'multiplayer',
		'optimization',
		'quests',
		'technology',
	];

	const curseforgeCategories = [
		'Adventure and RPG',
		'Combat / PvP',
		'Expert',
		'Exploration',
		'Extra Large',
		'FTB Official Pack',
		'Hardcore',
		'Horror',
		'Magic',
		'Map Based',
		'Mini Game',
		'Multiplayer',
		'Quests',
		'Sci-Fi',
		'Skyblock',
		'Small / Light',
		'Tech',
		'Vanilla+',
	];

	let searchInput = $state('');
	let showFilters = $state(false);
	let selectedModpackDetail = $state<Modpack | null>(null);
	let modpackDetailTab = $state<'about' | 'gallery' | 'mods'>('about');
	let modpackLightboxIndex = $state<number | null>(null);
	let descriptionExpanded = $state(false);
	let selectedVersionId = $state<string | null>(null);
	let isLoadingMods = $state(false);
	let modsError = $state<string | null>(null);
	let modListCache = $state<Record<string, ModpackMod[]>>({});
	let loadMoreSentinel: HTMLDivElement | undefined = $state();
	let selectedCategories = $state<string[]>([]);
	let categoriesOpen = $state(false);
	let hasUserScrolled = $state(false);
	let currentModpackGallery = $derived(selectedModpackDetail?.gallery ?? []);

	// Get available categories based on platform
	let availableCategories = $derived(
		modpacksStore.platform === 'modrinth'
			? modrinthCategories
			: modpacksStore.platform === 'curseforge'
				? curseforgeCategories
				: []
	);

	// Platforms that support category filtering
	const platformsWithCategoryFilter = ['modrinth', 'curseforge'];
	let showCategoryFilter = $derived(
		modpacksStore.platform && platformsWithCategoryFilter.includes(modpacksStore.platform)
	);

	onMount(() => {
		versionsStore.load();
		// Initial search
		modpacksStore.search();
	});

	// Track if the user has manually scrolled (mouse wheel or touch)
	const markScrollListener = () => markUserScrolled();
	onMount(() => {
		window.addEventListener('wheel', markScrollListener, { passive: true });
		window.addEventListener('touchmove', markScrollListener, { passive: true });
	});

	onDestroy(() => {
		window.removeEventListener('wheel', markScrollListener);
		window.removeEventListener('touchmove', markScrollListener);
	});

	function markUserScrolled() {
		hasUserScrolled = true;
	}

	// Set up infinite scroll observer when sentinel is available and the user has scrolled
	$effect(() => {
		if (!loadMoreSentinel || !hasUserScrolled) return;

		const observer = new IntersectionObserver(
			(entries) => {
				if (entries[0].isIntersecting && modpacksStore.hasMore && !modpacksStore.isSearching) {
					modpacksStore.loadMore();
				}
			},
			{ threshold: 0.1 }
		);

		observer.observe(loadMoreSentinel);
		return () => observer.disconnect();
	});

	// Debounced search on query change
	let searchTimeout: ReturnType<typeof setTimeout>;
	function handleSearchInput(value: string) {
		searchInput = value;
		clearTimeout(searchTimeout);
		searchTimeout = setTimeout(() => {
			modpacksStore.setQuery(value);
			modpacksStore.search();
		}, 300);
	}

	// Platforms that support version filtering
	const platformsWithVersionFilter = ['modrinth', 'curseforge', 'atlauncher'];
	// Platforms that support loader filtering
	const platformsWithLoaderFilter = ['modrinth', 'curseforge'];

	// Computed filter visibility - requires a specific platform to be selected
	let showVersionFilter = $derived(
		modpacksStore.platform && platformsWithVersionFilter.includes(modpacksStore.platform)
	);
	let showLoaderFilter = $derived(
		modpacksStore.platform && platformsWithLoaderFilter.includes(modpacksStore.platform)
	);

	// Show filter button only for platforms that have at least one filter type
	let hasAnyFilters = $derived(
		showVersionFilter || showLoaderFilter || (showCategoryFilter && availableCategories.length > 0)
	);

	function handlePlatformChange(platform: ModpackPlatform | 'all') {
		modpacksStore.setPlatform(platform === 'all' ? null : platform);
		// Clear filters that don't work on the selected platform
		if (platform !== 'all') {
			if (!platformsWithVersionFilter.includes(platform)) {
				modpacksStore.setMcVersion(null);
			}
			if (!platformsWithLoaderFilter.includes(platform)) {
				modpacksStore.setLoader(null);
			}
			if (!platformsWithCategoryFilter.includes(platform)) {
				selectedCategories = [];
				modpacksStore.setCategory(null);
			}
		}
		// Clear categories when switching platforms since they're different
		selectedCategories = [];
		modpacksStore.setCategory(null);
		// Only search if cache wasn't used (isSearching means cache miss)
		if (modpacksStore.isSearching) {
			modpacksStore.search();
		}
	}

	function toggleCategory(category: string) {
		if (selectedCategories.includes(category)) {
			selectedCategories = selectedCategories.filter((c) => c !== category);
		} else {
			selectedCategories = [...selectedCategories, category];
		}
		// Send first category to backend (backend currently only supports one)
		modpacksStore.setCategory(selectedCategories.length > 0 ? selectedCategories[0] : null);
		modpacksStore.search();
	}

	function clearCategories() {
		selectedCategories = [];
		modpacksStore.setCategory(null);
		modpacksStore.search();
	}

	function handleSortChange(sort: ModpackSortBy) {
		modpacksStore.setSortBy(sort);
		modpacksStore.search();
	}

	function handleVersionChange(version: string) {
		modpacksStore.setMcVersion(version || null);
		modpacksStore.search();
	}

	function handleLoaderChange(loader: string) {
		modpacksStore.setLoader(loader === 'any' ? null : (loader as LoaderType));
		modpacksStore.search();
	}

	function clearFilters() {
		modpacksStore.clearFilters();
		searchInput = '';
		selectedCategories = [];
		modpacksStore.search();
	}

	function formatDownloads(downloads: number): string {
		if (downloads >= 1_000_000) {
			return `${(downloads / 1_000_000).toFixed(1)}M`;
		}
		if (downloads >= 1_000) {
			return `${(downloads / 1_000).toFixed(1)}K`;
		}
		return downloads.toString();
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

	function handleModpackClick(modpack: Modpack) {
		modpackDetailTab = 'about';
		modpackLightboxIndex = null;
		descriptionExpanded = false;
		selectedVersionId = null;
		isLoadingMods = false;
		modsError = null;
		modListCache = {};
		// Show basic modpack immediately
		selectedModpackDetail = modpack;
		// Start loading details + versions in parallel (non-blocking)
		modpacksStore.selectModpack(modpack).then((detailed) => {
			selectedModpackDetail = detailed;
		});
	}

	// Auto-select first version when versions load
	$effect(() => {
		if (modpacksStore.selectedModpackVersions.length > 0 && !selectedVersionId) {
			selectedVersionId = modpacksStore.selectedModpackVersions[0]?.id ?? null;
		}
	});

	function closeModpackDetail() {
		selectedModpackDetail = null;
		modpackDetailTab = 'about';
		modpackLightboxIndex = null;
		descriptionExpanded = false;
		selectedVersionId = null;
		isLoadingMods = false;
		modsError = null;
		modListCache = {};
		modpacksStore.clearSelection();
	}

	function openModpackLightbox(index: number) {
		modpackLightboxIndex = index;
	}

	function closeModpackLightbox() {
		modpackLightboxIndex = null;
	}

	function prevModpackLightbox() {
		if (modpackLightboxIndex === null) return;
		modpackLightboxIndex = Math.max(0, modpackLightboxIndex - 1);
	}

	function nextModpackLightbox() {
		if (modpackLightboxIndex === null) return;
		modpackLightboxIndex = Math.min(currentModpackGallery.length - 1, modpackLightboxIndex + 1);
	}

	function isModListSupported(platform: ModpackPlatform): boolean {
		return (
			platform === 'modrinth' ||
			platform === 'curseforge' ||
			platform === 'ftb' ||
			platform === 'technic'
		);
	}

	async function loadModList() {
		if (!selectedModpackDetail || !selectedVersionId) return;
		if (!isModListSupported(selectedModpackDetail.platform)) return;
		if (modListCache[selectedVersionId]) return;

		isLoadingMods = true;
		modsError = null;
		try {
			const mods = await modpackService.getModpackMods(
				selectedModpackDetail.platform,
				selectedModpackDetail.id,
				selectedVersionId
			);
			modListCache = { ...modListCache, [selectedVersionId]: mods };
		} catch (e: unknown) {
			console.error('Failed to load mod list:', e);
			modsError = e instanceof Error ? e.message : typeof e === 'string' ? e : JSON.stringify(e);
		} finally {
			isLoadingMods = false;
		}
	}

	// Preload mod list as soon as a version is selected (don't wait for mods tab)
	$effect(() => {
		// Track dependencies for re-run when selection changes
		void selectedVersionId;
		void selectedModpackDetail?.platform;
		// Load mod list in background - will be ready when user clicks mods tab
		void loadModList();
	});

	async function handleInstall(versionId: string) {
		if (!selectedModpackDetail) return;

		console.log('[modpacks] Installing modpack:', {
			name: selectedModpackDetail.name,
			platform: selectedModpackDetail.platform,
			modpackId: selectedModpackDetail.id,
			versionId,
		});

		// The global modpackInstallStore will handle progress events
		// We just need to call the install function
		const instance = await modpacksStore.installModpack(
			selectedModpackDetail.platform,
			selectedModpackDetail.id,
			versionId,
			selectedModpackDetail.name
		);

		if (instance) {
			console.log('[modpacks] Install successful, closing modal');
			closeModpackDetail();
			alert(`Successfully installed ${selectedModpackDetail.name}!`);
		} else if (modpacksStore.installError && !modpacksStore.installError.includes('CANCELLED')) {
			console.error('[modpacks] Install failed:', modpacksStore.installError);
			alert(`Failed to install: ${modpacksStore.installError}`);
		}
		// If cancelled, the global store handles the UI feedback
	}

	async function handleCancelInstall() {
		if (!modpackInstallStore.modpackName || modpackInstallStore.isCancelling) return;

		const confirmed = await ask(
			`This will stop the download and remove any partially installed files.`,
			{
				title: `Cancel installation of "${modpackInstallStore.modpackName}"?`,
				kind: 'warning',
			}
		);

		if (confirmed) {
			modpackInstallStore.cancel();
		}
	}

	const platforms: { value: ModpackPlatform | 'all'; label: string }[] = [
		{ value: 'all', label: 'All' },
		{ value: 'modrinth', label: 'Modrinth' },
		{ value: 'curseforge', label: 'CurseForge' },
		{ value: 'ftb', label: 'FTB' },
		{ value: 'technic', label: 'Technic' },
		{ value: 'atlauncher', label: 'ATLauncher' },
	];

	const sortOptions: { value: ModpackSortBy; label: string }[] = [
		{ value: 'downloads', label: 'Downloads' },
		{ value: 'recentlyUpdated', label: 'Recently Updated' },
		{ value: 'relevance', label: 'Relevance' },
		{ value: 'name', label: 'Name' },
	];
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between gap-4">
		<h1 class="text-2xl">Modpacks</h1>
		{#if hasAnyFilters}
			<div class="flex items-center gap-2">
				<Button variant="outline" size="sm" onclick={() => (showFilters = !showFilters)}>
					<Filter class="mr-2 h-4 w-4" />
					Filters
					{#if modpacksStore.mcVersion || modpacksStore.loader || modpacksStore.category}
						<span class="bg-primary text-primary-foreground ml-1 rounded-full px-1.5 text-xs">
							!
						</span>
					{/if}
				</Button>
			</div>
		{/if}
	</div>

	<!-- Platform Tabs -->
	<div class="flex flex-wrap gap-2">
		{#each platforms as { value, label } (value)}
			<Button
				variant={modpacksStore.platform === (value === 'all' ? null : value)
					? 'default'
					: 'secondary'}
				size="sm"
				onclick={() => handlePlatformChange(value)}
			>
				{label}
			</Button>
		{/each}
	</div>

	<!-- Search and Sort -->
	<div class="flex items-center gap-4" data-tutorial="modpack-search">
		<div class="relative max-w-md flex-1">
			<Search class="text-muted-foreground absolute top-1/2 left-3 z-10 h-4 w-4 -translate-y-1/2" />
			<Input
				type="text"
				placeholder="Search modpacks..."
				value={searchInput}
				oninput={(e) => handleSearchInput(e.currentTarget.value)}
				class="pl-9"
			/>
		</div>
		<Select.Root
			type="single"
			value={modpacksStore.sortBy}
			onValueChange={(v) => handleSortChange(v as ModpackSortBy)}
		>
			<Select.Trigger class="border-border bg-card w-48 border-2">
				<ArrowUpDown class="mr-2 h-4 w-4" />
				{sortOptions.find((o) => o.value === modpacksStore.sortBy)?.label || 'Sort by'}
			</Select.Trigger>
			<Select.Content class="border-border bg-card border-2">
				{#each sortOptions as { value, label } (value)}
					<Select.Item {value} {label}>{label}</Select.Item>
				{/each}
			</Select.Content>
		</Select.Root>
	</div>

	<!-- Filters Panel -->
	{#if showFilters && hasAnyFilters}
		<div class="bg-card border-border space-y-4 border-2 p-4">
			<div class="flex items-center justify-between">
				<h3 class="font-semibold">Filters</h3>
				<Button variant="ghost" size="sm" onclick={clearFilters}>
					<X class="mr-1 h-4 w-4" />
					Clear
				</Button>
			</div>
			<div class="grid grid-cols-1 gap-4 md:grid-cols-3">
				{#if showVersionFilter}
					<div>
						<p class="text-muted-foreground mb-1 block text-sm">Minecraft Version</p>
						<Select.Root
							type="single"
							value={modpacksStore.mcVersion || ''}
							onValueChange={handleVersionChange}
						>
							<Select.Trigger class="border-border bg-background w-full border-2">
								{modpacksStore.mcVersion || 'Any version'}
							</Select.Trigger>
							<Select.Content class="border-border bg-card max-h-[300px] border-2">
								<Select.Item value="" label="Any version">Any version</Select.Item>
								{#each versionsStore.versions.filter((v) => v.type === 'release') as version (version.id)}
									<Select.Item value={version.id} label={version.id}>{version.id}</Select.Item>
								{/each}
							</Select.Content>
						</Select.Root>
					</div>
				{/if}
				{#if showLoaderFilter}
					<div>
						<p class="text-muted-foreground mb-1 block text-sm">Mod Loader</p>
						<Select.Root
							type="single"
							value={modpacksStore.loader || 'any'}
							onValueChange={handleLoaderChange}
						>
							<Select.Trigger class="border-border bg-background w-full border-2">
								{modpacksStore.loader
									? modpacksStore.loader.charAt(0).toUpperCase() + modpacksStore.loader.slice(1)
									: 'Any loader'}
							</Select.Trigger>
							<Select.Content class="border-border bg-card border-2">
								<Select.Item value="any" label="Any loader">Any loader</Select.Item>
								<Select.Item value="fabric" label="Fabric">Fabric</Select.Item>
								<Select.Item value="forge" label="Forge">Forge</Select.Item>
								<Select.Item value="neoforge" label="NeoForge">NeoForge</Select.Item>
								<Select.Item value="quilt" label="Quilt">Quilt</Select.Item>
							</Select.Content>
						</Select.Root>
					</div>
				{/if}
				{#if showCategoryFilter && availableCategories.length > 0}
					<div class="relative">
						<p class="text-muted-foreground mb-1 block text-sm">Categories</p>
						<button
							type="button"
							class="border-border bg-background flex h-9 w-full items-center justify-between border-2 px-3 text-left text-sm"
							onclick={() => (categoriesOpen = !categoriesOpen)}
						>
							{#if selectedCategories.length === 0}
								<span class="text-muted-foreground">Any category</span>
							{:else if selectedCategories.length === 1}
								<span class="capitalize">{selectedCategories[0].replace(/-/g, ' ')}</span>
							{:else}
								<span>{selectedCategories.length} selected</span>
							{/if}
							<ChevronDown
								class="text-muted-foreground h-4 w-4 {categoriesOpen
									? 'rotate-180'
									: ''} transition-transform"
							/>
						</button>
						{#if categoriesOpen}
							<div
								class="border-border bg-card absolute top-full right-0 left-0 z-50 mt-1 max-h-[250px] overflow-y-auto border-2 shadow-lg"
							>
								<div
									class="border-border bg-card sticky top-0 flex items-center justify-between border-b p-2"
								>
									<span class="text-sm font-medium">Categories</span>
									{#if selectedCategories.length > 0}
										<button
											type="button"
											class="text-muted-foreground hover:text-foreground text-xs"
											onclick={clearCategories}
										>
											Clear all
										</button>
									{/if}
								</div>
								<div class="space-y-1 p-2">
									{#each availableCategories as category (category)}
										<button
											type="button"
											class="hover:bg-muted/50 flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-sm"
											onclick={() => toggleCategory(category)}
										>
											<Checkbox
												checked={selectedCategories.includes(category)}
												class="pointer-events-none"
											/>
											<span class="capitalize">{category.replace(/-/g, ' ')}</span>
										</button>
									{/each}
								</div>
							</div>
						{/if}
					</div>
				{/if}
			</div>
		</div>
	{/if}

	<!-- Error Display -->
	{#if modpacksStore.searchError}
		<div class="bg-destructive/10 border-destructive text-destructive border-2 p-4 text-sm">
			{modpacksStore.searchError}
			<button class="ml-2 underline" onclick={() => modpacksStore.clearSearchError()}>
				Dismiss
			</button>
		</div>
	{/if}

	<!-- Results -->
	{#if modpacksStore.isSearching && modpacksStore.modpacks.length === 0}
		<!-- Skeleton loading state -->
		<div class="text-muted-foreground text-sm">
			<Skeleton class="h-5 w-40" />
		</div>
		<div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
			{#each Array.from({ length: 6 }, (_, i) => i) as i (i)}
				<div class="border-border bg-card border-2 p-4">
					<div class="flex gap-3">
						<Skeleton class="h-16 w-16 rounded" />
						<div class="min-w-0 flex-1 space-y-2">
							<Skeleton class="h-5 w-3/4" />
							<Skeleton class="h-4 w-full" />
							<Skeleton class="h-4 w-2/3" />
							<div class="flex gap-2 pt-1">
								<Skeleton class="h-5 w-16 rounded-full" />
								<Skeleton class="h-5 w-20 rounded-full" />
							</div>
						</div>
					</div>
				</div>
			{/each}
		</div>
	{:else if modpacksStore.modpacks.length === 0}
		<div class="border-border bg-card/50 border-2 border-dashed p-12 text-center">
			<Package class="text-muted-foreground/50 mx-auto h-12 w-12" />
			<p class="text-muted-foreground mt-4 text-sm">
				{searchInput ? 'No modpacks match your search' : 'Search for modpacks to get started'}
			</p>
		</div>
	{:else}
		<!-- Results count -->
		<div class="text-muted-foreground text-sm">
			Showing {modpacksStore.modpacks.length} of {modpacksStore.totalCount} modpacks
		</div>

		<!-- Modpack Grid -->
		<div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
			{#each modpacksStore.modpacks as modpack, index (`${modpack.platform}-${modpack.id}`)}
				<button
					class="border-border bg-card hover:border-primary/50 cursor-pointer border-2 p-4 text-left transition-colors"
					onclick={() => handleModpackClick(modpack)}
					data-tutorial={index === 0 ? 'modpack-card' : undefined}
				>
					<div class="flex gap-3">
						{#if modpack.iconUrl}
							<img
								src={modpack.iconUrl}
								alt={modpack.name}
								class="h-16 w-16 rounded object-cover"
							/>
						{:else}
							<div class="bg-muted flex h-16 w-16 items-center justify-center rounded">
								<Package class="text-muted-foreground/50 h-8 w-8" />
							</div>
						{/if}
						<div class="min-w-0 flex-1">
							<h3 class="truncate font-bold">{modpack.name}</h3>
							<p class="text-muted-foreground truncate text-sm">{modpack.author}</p>
							<div class="mt-1 flex flex-wrap items-center gap-2">
								<span
									class="rounded border px-1.5 py-0.5 text-xs {getPlatformColor(modpack.platform)}"
								>
									{modpack.platform}
								</span>
								{#each (modpack.loaders || [])
									.filter((l) => l && l !== 'unknown' && l !== 'vanilla')
									.slice(0, 2) as loader (loader)}
									<span class="rounded px-1.5 py-0.5 text-xs {getLoaderColor(loader)}">
										{loader}
									</span>
								{/each}
							</div>
						</div>
					</div>
					<p class="text-muted-foreground mt-3 line-clamp-2 text-sm">
						{modpack.description}
					</p>
					<div class="text-muted-foreground mt-3 flex items-center gap-4 text-xs">
						<span class="flex items-center gap-1">
							<Download class="h-3 w-3" />
							{formatDownloads(modpack.downloads)}
						</span>
						{#if modpack.mcVersions.length > 0}
							<span>MC {modpack.mcVersions[0]}</span>
						{/if}
					</div>
				</button>
			{/each}
		</div>

		<!-- Infinite scroll sentinel -->
		<div bind:this={loadMoreSentinel} class="h-4">
			{#if modpacksStore.isSearching && modpacksStore.modpacks.length > 0}
				<div class="flex justify-center py-4">
					<Loader2 class="text-muted-foreground h-6 w-6 animate-spin" />
				</div>
			{/if}
		</div>
	{/if}
</div>

<!-- Modpack Detail Modal -->
{#if selectedModpackDetail}
	<div
		class="fixed inset-x-0 top-[var(--titlebar-height)] z-50 flex h-[calc(100vh-var(--titlebar-height))] items-center justify-center bg-black/50 p-4"
	>
		<div
			class="bg-card border-border flex max-h-[90vh] w-full max-w-6xl flex-col rounded-lg border-2 shadow-2xl"
		>
			<!-- Header -->
			<div class="border-border flex-shrink-0 border-b p-6">
				<div class="flex gap-4">
					{#if selectedModpackDetail.iconUrl}
						<img
							src={selectedModpackDetail.iconUrl}
							alt={selectedModpackDetail.name}
							class="h-24 w-24 rounded object-cover"
						/>
					{:else}
						<div class="bg-muted flex h-24 w-24 items-center justify-center rounded">
							<Package class="text-muted-foreground/50 h-12 w-12" />
						</div>
					{/if}
					<div class="flex-1">
						<div class="flex items-start justify-between gap-2">
							<h2 class="text-xl font-bold">{selectedModpackDetail.name}</h2>
							<button
								class="text-muted-foreground hover:text-foreground"
								onclick={closeModpackDetail}
							>
								<X class="h-5 w-5" />
							</button>
						</div>
						<p class="text-muted-foreground">{selectedModpackDetail.author}</p>
						<div class="mt-2 flex flex-wrap items-center gap-2">
							<span
								class="rounded border px-1.5 py-0.5 text-xs {getPlatformColor(
									selectedModpackDetail.platform
								)}"
							>
								{selectedModpackDetail.platform}
							</span>
							{#each (selectedModpackDetail.loaders || []).filter((l) => l && l !== 'unknown' && l !== 'vanilla') as loader (loader)}
								<span class="rounded px-1.5 py-0.5 text-xs {getLoaderColor(loader)}">
									{loader}
								</span>
							{/each}
							<span class="text-muted-foreground flex items-center gap-1 text-xs">
								<Download class="h-3 w-3" />
								{formatDownloads(selectedModpackDetail.downloads)}
							</span>
						</div>
					</div>
				</div>
			</div>

			<div
				class="grid min-h-0 flex-1 grid-cols-1 gap-4 overflow-hidden p-5 md:grid-cols-[2.5fr_1fr] xl:grid-cols-[3fr_1fr]"
			>
				<div class="min-h-0 space-y-4 overflow-y-auto pr-1">
					<div class="flex items-center gap-2">
						<Button
							size="sm"
							variant={modpackDetailTab === 'about' ? 'default' : 'secondary'}
							onclick={() => (modpackDetailTab = 'about')}
						>
							About
						</Button>
						<Button
							size="sm"
							variant={modpackDetailTab === 'gallery' ? 'default' : 'secondary'}
							disabled={(selectedModpackDetail.gallery?.length ?? 0) === 0}
							onclick={() => (modpackDetailTab = 'gallery')}
						>
							Gallery
						</Button>
						<Button
							size="sm"
							variant={modpackDetailTab === 'mods' ? 'default' : 'secondary'}
							disabled={!selectedVersionId || !isModListSupported(selectedModpackDetail.platform)}
							onclick={() => (modpackDetailTab = 'mods')}
						>
							Mods
						</Button>
					</div>

					{#if modpackDetailTab === 'mods'}
						{#if !isModListSupported(selectedModpackDetail.platform)}
							<p class="text-muted-foreground text-sm">
								Mod list is not available for this platform.
							</p>
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
						{:else if (modListCache[selectedVersionId]?.length ?? 0) === 0}
							<p class="text-muted-foreground text-sm">No mods found for this version.</p>
						{:else}
							<div class="space-y-2">
								<div class="flex items-center justify-between">
									<h3 class="text-sm font-semibold">Mods</h3>
									<span class="text-muted-foreground text-xs">
										{modListCache[selectedVersionId]?.length ?? 0} mods
									</span>
								</div>
								<div class="border-border overflow-hidden rounded-lg border-2">
									<div class="max-h-[60vh] overflow-y-auto">
										{#each modListCache[selectedVersionId] ?? [] as modItem (modItem.id)}
											<button
												type="button"
												class="hover:bg-muted/50 border-border flex w-full items-center gap-3 border-b p-3 text-left last:border-b-0"
												onclick={() => modItem.url && openUrl(modItem.url)}
												disabled={!modItem.url}
											>
												{#if modItem.iconUrl}
													<img
														src={modItem.iconUrl}
														alt={modItem.name}
														class="border-border bg-muted h-10 w-10 rounded border object-cover"
														loading="lazy"
													/>
												{:else}
													<div
														class="border-border bg-muted text-muted-foreground flex h-10 w-10 items-center justify-center rounded border text-xs"
													>
														MOD
													</div>
												{/if}
												<div class="min-w-0">
													<div class="truncate text-sm font-medium">{modItem.name}</div>
													{#if modItem.author}
														<div class="text-muted-foreground truncate text-xs">
															{modItem.author}
														</div>
													{/if}
												</div>
											</button>
										{/each}
									</div>
								</div>
							</div>
						{/if}
					{:else if modpackDetailTab === 'gallery'}
						{#if (selectedModpackDetail.gallery?.length ?? 0) > 0}
							<div class="space-y-2">
								<div class="flex items-center justify-between">
									<h3 class="text-sm font-semibold">Gallery</h3>
									<span class="text-muted-foreground text-xs">
										{selectedModpackDetail.gallery?.length ?? 0} images
									</span>
								</div>
								<div class="grid gap-3 sm:grid-cols-2">
									{#each selectedModpackDetail.gallery ?? [] as image, idx (image.rawUrl ?? image.url)}
										<button
											type="button"
											class="border-border bg-muted/50 relative aspect-video cursor-pointer overflow-hidden rounded-lg border-2 text-left"
											onclick={() => openModpackLightbox(idx)}
										>
											<img
												src={image.rawUrl ?? image.url}
												alt={image.title ?? selectedModpackDetail.name}
												class="h-full w-full object-cover"
												loading="lazy"
											/>
											{#if image.title || image.description}
												<div
													class="absolute inset-x-0 bottom-0 space-y-1 bg-gradient-to-t from-black/80 via-black/40 to-transparent p-2 text-xs text-white"
												>
													{#if image.title}
														<div class="truncate leading-tight font-semibold">{image.title}</div>
													{/if}
													{#if image.description}
														<p class="line-clamp-2 leading-snug opacity-90">{image.description}</p>
													{/if}
												</div>
											{/if}
										</button>
									{/each}
								</div>
							</div>
						{:else}
							<p class="text-muted-foreground text-sm">No gallery available.</p>
						{/if}
					{:else}
						<div class="border-border bg-background/70 space-y-2 rounded-lg border-2 p-4">
							<div class="flex items-center justify-between gap-2">
								<h3 class="text-sm font-semibold">About</h3>
								<div class="flex items-center gap-2">
									<Button
										size="sm"
										variant="secondary"
										onclick={() => (descriptionExpanded = true)}
										disabled={!selectedModpackDetail.body && !selectedModpackDetail.description}
									>
										<Maximize2 class="mr-1 h-4 w-4" />
										Expand
									</Button>
									{#if selectedModpackDetail.url}
										<a
											href={selectedModpackDetail.url}
											target="_blank"
											rel="noopener noreferrer"
											class="bg-muted hover:bg-muted/80 inline-flex items-center gap-1.5 rounded px-2.5 py-1.5 text-xs transition-colors"
										>
											<ExternalLink class="h-3.5 w-3.5" />
											View on {selectedModpackDetail.platform}
										</a>
									{/if}
								</div>
							</div>

							{#if modpacksStore.isLoadingDetail}
								<div class="text-muted-foreground flex items-center gap-2 text-sm">
									<Loader2 class="h-4 w-4 animate-spin" />
									Loading description...
								</div>
							{:else}
								{#if modpacksStore.detailError}
									<div
										class="bg-destructive/10 border-destructive text-destructive rounded border-2 p-3 text-sm"
									>
										{modpacksStore.detailError}
									</div>
								{/if}
								{#if selectedModpackDetail.body || selectedModpackDetail.description}
									<!-- svelte-ignore a11y_click_events_have_key_events -->
									<!-- svelte-ignore a11y_no_static_element_interactions -->
									<div
										class="[&_a]:text-primary text-sm leading-relaxed [&_a]:underline [&_h1]:text-lg [&_h1]:font-semibold [&_h2]:text-base [&_h2]:font-semibold [&_img]:my-2 [&_img]:max-w-full [&_img]:rounded-md [&_ol]:list-decimal [&_ol]:pl-5 [&_p]:mb-3 [&_ul]:list-disc [&_ul]:pl-5"
										onclick={handleDescriptionLinkClick}
									>
										<!-- eslint-disable-next-line svelte/no-at-html-tags -->
										{@html renderMarkdown(
											selectedModpackDetail.body,
											selectedModpackDetail.description
										)}
									</div>
								{:else}
									<p class="text-muted-foreground text-sm">No description available.</p>
								{/if}
							{/if}
						</div>
					{/if}
				</div>

				<div class="min-h-0 space-y-3 overflow-y-auto pr-1">
					<div class="border-border bg-background/70 rounded-lg border-2 p-3">
						<h3 class="mb-2 text-sm font-semibold">Select Version</h3>
						{#if modpacksStore.isLoadingVersions}
							<div class="text-muted-foreground flex items-center gap-2 text-sm">
								<Loader2 class="h-4 w-4 animate-spin" />
								Loading versions...
							</div>
						{:else if modpacksStore.selectedModpackVersions.length === 0}
							<p class="text-muted-foreground text-sm">No versions available</p>
						{:else}
							<div class="space-y-1">
								{#each modpacksStore.selectedModpackVersions.slice(0, 15) as version, versionIndex (version.id)}
									{@const isSelected = selectedVersionId === version.id}
									<button
										type="button"
										class="w-full rounded border-2 p-2 text-left transition-colors {isSelected
											? 'border-primary bg-primary/10'
											: 'border-border hover:border-primary/50'}"
										onclick={() => (selectedVersionId = version.id)}
										data-tutorial={versionIndex === 0 ? 'modpack-install' : undefined}
									>
										<div class="flex items-center gap-2">
											{#if isSelected}
												<Check class="text-primary h-4 w-4" />
											{/if}
											<span class="text-sm font-medium">{version.name}</span>
										</div>
										<div class="text-muted-foreground mt-0.5 text-xs">
											MC {version.mcVersion} &bull; {version.loaderType}
											{#if version.releasedAt}
												&bull; {new Date(version.releasedAt * 1000).toLocaleDateString()}
											{/if}
										</div>
									</button>
								{/each}
							</div>
						{/if}
					</div>
				</div>
			</div>

			<!-- Footer with Install/Cancel buttons -->
			{#if !modpackInstallStore.isInstalling}
				<div class="border-border bg-card flex flex-shrink-0 justify-end gap-3 border-t p-4">
					<Button variant="outline" onclick={closeModpackDetail}>Cancel</Button>
					<Button
						disabled={!selectedVersionId || modpackInstallStore.isInstalling}
						onclick={() => selectedVersionId && handleInstall(selectedVersionId)}
					>
						<Download class="mr-2 h-4 w-4" />
						Install
					</Button>
				</div>
			{/if}

			<!-- Sticky Install Progress -->
			{#if modpackInstallStore.isInstalling}
				<div
					class="border-border bg-card flex-shrink-0 border-t p-4 shadow-[0_-4px_6px_-1px_rgba(0,0,0,0.1)]"
				>
					<div class="flex items-center gap-3">
						<div class="flex-1">
							{#if modpackInstallStore.progress}
								<DownloadProgress
									stage={modpackInstallStore.progress.stage}
									progress={modpackInstallStore.progress.progress}
									currentItem={modpackInstallStore.progress.currentItem}
									totalItems={modpackInstallStore.progress.totalItems}
									completedItems={modpackInstallStore.progress.completedItems}
								/>
							{:else}
								<div class="flex items-center gap-2 text-sm">
									<Loader2 class="text-primary h-4 w-4 flex-shrink-0 animate-spin" />
									<span class="font-medium">Starting installation...</span>
								</div>
							{/if}
						</div>
						<Button
							variant="destructive"
							size="sm"
							onclick={handleCancelInstall}
							disabled={modpackInstallStore.isCancelling}
						>
							<StopCircle class="mr-1 h-4 w-4" />
							Cancel
						</Button>
					</div>
				</div>
			{/if}
		</div>
	</div>
{/if}

<ScreenshotLightbox
	open={modpackLightboxIndex !== null}
	src={modpackLightboxIndex !== null
		? (currentModpackGallery[modpackLightboxIndex]?.rawUrl ??
			currentModpackGallery[modpackLightboxIndex]?.url ??
			null)
		: null}
	filename={modpackLightboxIndex !== null
		? (currentModpackGallery[modpackLightboxIndex]?.title ?? `Image ${modpackLightboxIndex + 1}`)
		: undefined}
	canPrev={modpackLightboxIndex !== null && modpackLightboxIndex > 0}
	canNext={modpackLightboxIndex !== null && modpackLightboxIndex < currentModpackGallery.length - 1}
	onClose={closeModpackLightbox}
	onPrev={prevModpackLightbox}
	onNext={nextModpackLightbox}
/>

<DescriptionModal
	open={descriptionExpanded && !!selectedModpackDetail}
	title={selectedModpackDetail ? `${selectedModpackDetail.name} — Description` : 'Description'}
	html={selectedModpackDetail
		? renderMarkdown(selectedModpackDetail.body, selectedModpackDetail.description)
		: ''}
	onClose={() => (descriptionExpanded = false)}
/>
