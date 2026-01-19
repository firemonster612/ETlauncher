<script lang="ts">
	import { onMount } from 'svelte';
	import { Search, Loader2, AlertTriangle, ArrowUpDown, X, Filter } from '@lucide/svelte';
	import { ask } from '@tauri-apps/plugin-dialog';
	import { Button } from '$lib/ui/button';
	import { Input } from '$lib/ui/input';
	import * as Select from '$lib/ui/select';
	import { modpacksStore } from '$lib/stores/modpacks.svelte';
	import { modpackInstallStore } from '$lib/stores/modpackInstall.svelte';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { versionsStore } from '$lib/stores/versions.svelte';
	import ModpackDetailModalV2 from '$lib/components/modpack/ModpackDetailModalV2.svelte';
	import PopularModpacksSection from '$lib/components/modpack/PopularModpacksSection.svelte';
	import PlatformToggle from '$lib/components/modpack/PlatformToggle.svelte';
	import ModpackCard from '$lib/components/modpack/ModpackCard.svelte';
	import type { Modpack, ModpackPlatform, ModpackSortBy, LoaderType } from '$lib/types';

	let searchInput = $state('');
	let selectedModpackDetail = $state<Modpack | null>(null);
	let scrollContainer = $state<HTMLElement | null>(null);

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

	// Check if CurseForge API key is missing when CurseForge is selected
	let showCurseForgeWarning = $derived(
		modpacksStore.explorePlatform === 'curseforge' && !settingsStore.settings?.curseforgeApiKey
	);

	// Current platform for toggle buttons
	let currentPlatform = $derived<ModpackPlatform | 'all'>(modpacksStore.explorePlatform ?? 'all');

	// Show filters only for platforms that support them
	let showFilters = $derived(
		modpacksStore.explorePlatform === 'modrinth' || modpacksStore.explorePlatform === 'curseforge'
	);

	// Get available categories based on platform
	let availableCategories = $derived(
		modpacksStore.explorePlatform === 'modrinth'
			? modrinthCategories
			: modpacksStore.explorePlatform === 'curseforge'
				? curseforgeCategories
				: []
	);

	// Check if any filters are active
	let hasActiveFilters = $derived(
		modpacksStore.exploreMcVersion !== null ||
			modpacksStore.exploreLoader !== null ||
			modpacksStore.exploreCategory !== null
	);

	const platforms: (ModpackPlatform | 'all')[] = [
		'all',
		'modrinth',
		'curseforge',
		'ftb',
		'technic',
		'atlauncher',
	];

	const sortOptions: { value: ModpackSortBy; label: string }[] = [
		{ value: 'downloads', label: 'Most Popular' },
		{ value: 'recentlyUpdated', label: 'Recently Updated' },
		{ value: 'name', label: 'Name' },
	];

	onMount(() => {
		// Load versions for the filter dropdown
		versionsStore.load();
		// Load popular modpacks for the carousel
		modpacksStore.loadHomeData();
		// Load explore data for the grid
		modpacksStore.loadExploreData();
	});

	// Debounced search
	let searchTimeout: ReturnType<typeof setTimeout>;
	function handleSearchInput(value: string) {
		searchInput = value;
		clearTimeout(searchTimeout);
		searchTimeout = setTimeout(() => {
			if (value) {
				modpacksStore.setQuery(value);
				modpacksStore.search();
			} else {
				modpacksStore.setQuery('');
				modpacksStore.loadExploreData();
			}
		}, 300);
	}

	function handlePlatformChange(platform: ModpackPlatform | 'all') {
		const newPlatform = platform === 'all' ? null : platform;
		modpacksStore.setExplorePlatform(newPlatform);
		// Clear filters when changing platforms since they may not apply
		modpacksStore.setExploreMcVersion(null);
		modpacksStore.setExploreLoader(null);
		modpacksStore.setExploreCategory(null);
		modpacksStore.loadExploreData();
	}

	function handleSortChange(sort: ModpackSortBy) {
		modpacksStore.setExploreSortBy(sort);
		modpacksStore.loadExploreData();
	}

	function handleVersionChange(version: string) {
		modpacksStore.setExploreMcVersion(version || null);
		modpacksStore.loadExploreData();
	}

	function handleLoaderChange(loader: string) {
		modpacksStore.setExploreLoader(loader === 'any' ? null : (loader as LoaderType));
		modpacksStore.loadExploreData();
	}

	function handleCategoryChange(category: string) {
		modpacksStore.setExploreCategory(category || null);
		modpacksStore.loadExploreData();
	}

	function clearFilters() {
		modpacksStore.setExploreMcVersion(null);
		modpacksStore.setExploreLoader(null);
		modpacksStore.setExploreCategory(null);
		modpacksStore.loadExploreData();
	}

	function handleModpackClick(modpack: Modpack) {
		// Show basic modpack immediately
		selectedModpackDetail = modpack;
		// Start loading details + versions in parallel (non-blocking)
		modpacksStore.selectModpack(modpack).then((detailed) => {
			selectedModpackDetail = detailed;
		});
	}

	function closeModpackDetail() {
		selectedModpackDetail = null;
		modpacksStore.clearSelection();
	}

	async function handleInstall(versionId: string) {
		if (!selectedModpackDetail) return;

		const instance = await modpacksStore.installModpack(
			selectedModpackDetail.platform,
			selectedModpackDetail.id,
			versionId,
			selectedModpackDetail.name
		);

		if (instance) {
			closeModpackDetail();
			alert(`Successfully installed ${selectedModpackDetail.name}!`);
		} else if (modpacksStore.installError && !modpacksStore.installError.includes('CANCELLED')) {
			alert(`Failed to install: ${modpacksStore.installError}`);
		}
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

	// Infinite scroll handler
	function handleScroll(e: Event) {
		const target = e.target as HTMLElement;
		const { scrollTop, scrollHeight, clientHeight } = target;
		const nearBottom = scrollHeight - scrollTop - clientHeight < 300;

		if (
			nearBottom &&
			modpacksStore.hasMoreExplore &&
			!modpacksStore.isLoadingExplore &&
			!searchInput
		) {
			modpacksStore.loadMoreExplore();
		}
	}

	// Get modpacks to display - either search results or explore
	let displayModpacks = $derived(
		searchInput && modpacksStore.modpacks.length > 0
			? modpacksStore.modpacks
			: modpacksStore.exploreModpacks
	);
	let isLoading = $derived(
		searchInput ? modpacksStore.isSearching : modpacksStore.isLoadingExplore
	);
	let totalCount = $derived(
		searchInput ? modpacksStore.totalCount : modpacksStore.exploreTotalCount
	);
</script>

<div class="flex h-full flex-col gap-6 overflow-hidden">
	<!-- Search Bar (centered at top) -->
	<div class="flex justify-center px-4">
		<div class="relative w-full max-w-xl">
			<Search class="text-muted-foreground absolute top-1/2 left-3 z-10 h-5 w-5 -translate-y-1/2" />
			<Input
				type="text"
				placeholder="Search modpacks..."
				value={searchInput}
				oninput={(e) => handleSearchInput(e.currentTarget.value)}
				class="h-11 pl-10 text-base"
			/>
		</div>
	</div>

	<!-- Scrollable Content -->
	<div class="flex-1 overflow-y-auto px-4 pb-4" bind:this={scrollContainer} onscroll={handleScroll}>
		<!-- Popular Modpacks Section (only show when not searching) -->
		{#if !searchInput}
			<PopularModpacksSection
				modpacks={modpacksStore.popularModpacks}
				loading={modpacksStore.isLoadingPopular}
				onModpackClick={handleModpackClick}
			/>
		{/if}

		<!-- Explore Section -->
		<section class="mt-8">
			<!-- Platform Toggles + Sort -->
			<div class="explore-filters-bar mb-4">
				<!-- Platform Toggle Group -->
				<div class="platform-toggle-group">
					{#each platforms as platform (platform)}
						<PlatformToggle
							{platform}
							active={currentPlatform === platform}
							onclick={() => handlePlatformChange(platform)}
						/>
					{/each}
				</div>

				<!-- Spacer -->
				<div class="flex-1"></div>

				<!-- Sort Dropdown -->
				<Select.Root
					type="single"
					value={modpacksStore.exploreSortBy}
					onValueChange={(v) => handleSortChange(v as ModpackSortBy)}
				>
					<Select.Trigger class="sort-select w-40">
						<ArrowUpDown class="mr-1 h-3 w-3" />
						<span class="truncate">
							{sortOptions.find((o) => o.value === modpacksStore.exploreSortBy)?.label}
						</span>
					</Select.Trigger>
					<Select.Content class="border-border bg-card border-2">
						{#each sortOptions as { value, label } (value)}
							<Select.Item {value} {label}>{label}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>

			<!-- Advanced Filters (only for Modrinth/CurseForge) -->
			{#if showFilters}
				<div class="explore-filters-bar mb-4">
					<Filter class="text-muted-foreground h-4 w-4" />

					<!-- MC Version -->
					<Select.Root
						type="single"
						value={modpacksStore.exploreMcVersion || ''}
						onValueChange={handleVersionChange}
					>
						<Select.Trigger class="sort-select w-32">
							<span class="truncate">{modpacksStore.exploreMcVersion || 'MC Version'}</span>
						</Select.Trigger>
						<Select.Content class="border-border bg-card max-h-[300px] border-2">
							<Select.Item value="" label="Any">Any Version</Select.Item>
							{#each versionsStore.versions.filter((v) => v.type === 'release') as version (version.id)}
								<Select.Item value={version.id} label={version.id}>{version.id}</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>

					<!-- Loader -->
					<Select.Root
						type="single"
						value={modpacksStore.exploreLoader || 'any'}
						onValueChange={handleLoaderChange}
					>
						<Select.Trigger class="sort-select w-28">
							<span class="truncate">
								{modpacksStore.exploreLoader
									? modpacksStore.exploreLoader.charAt(0).toUpperCase() +
										modpacksStore.exploreLoader.slice(1)
									: 'Loader'}
							</span>
						</Select.Trigger>
						<Select.Content class="border-border bg-card border-2">
							<Select.Item value="any" label="Any">Any Loader</Select.Item>
							<Select.Item value="fabric" label="Fabric">Fabric</Select.Item>
							<Select.Item value="forge" label="Forge">Forge</Select.Item>
							<Select.Item value="neoforge" label="NeoForge">NeoForge</Select.Item>
							<Select.Item value="quilt" label="Quilt">Quilt</Select.Item>
						</Select.Content>
					</Select.Root>

					<!-- Category -->
					<Select.Root
						type="single"
						value={modpacksStore.exploreCategory || ''}
						onValueChange={handleCategoryChange}
					>
						<Select.Trigger class="sort-select w-36">
							<span class="truncate">
								{modpacksStore.exploreCategory
									? modpacksStore.exploreCategory.replace(/-/g, ' ')
									: 'Category'}
							</span>
						</Select.Trigger>
						<Select.Content class="border-border bg-card max-h-[300px] border-2">
							<Select.Item value="" label="Any">Any Category</Select.Item>
							{#each availableCategories as category (category)}
								<Select.Item value={category} label={category}>
									<span class="capitalize">{category.replace(/-/g, ' ')}</span>
								</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>

					<!-- Clear Filters -->
					{#if hasActiveFilters}
						<Button
							variant="ghost"
							size="sm"
							onclick={clearFilters}
							class="text-muted-foreground h-8"
						>
							<X class="mr-1 h-3 w-3" />
							Clear
						</Button>
					{/if}
				</div>
			{/if}

			<!-- CurseForge API Warning -->
			{#if showCurseForgeWarning}
				<div class="mb-4 flex items-start gap-3 border-2 border-yellow-500/50 bg-yellow-500/10 p-3">
					<AlertTriangle class="h-4 w-4 flex-shrink-0 text-yellow-500" />
					<div class="text-sm">
						<span class="font-bold text-yellow-500">CurseForge API Key Required</span>
						<span class="text-muted-foreground ml-2">
							<a href="/settings" class="text-primary hover:underline">Add in Settings</a>
						</span>
					</div>
				</div>
			{/if}

			<!-- Results Count -->
			<div class="text-muted-foreground mb-4 text-xs">
				{#if isLoading && displayModpacks.length === 0}
					Loading modpacks...
				{:else if searchInput}
					Found {totalCount.toLocaleString()} modpacks matching "{searchInput}"
				{:else}
					Showing {displayModpacks.length} of {totalCount.toLocaleString()} modpacks
				{/if}
			</div>

			<!-- Modpack Grid -->
			{#if isLoading && displayModpacks.length === 0}
				<div class="flex items-center justify-center py-16">
					<Loader2 class="text-muted-foreground h-10 w-10 animate-spin" />
				</div>
			{:else if displayModpacks.length === 0}
				<div
					class="border-border bg-card/50 flex flex-col items-center justify-center border-2 border-dashed py-16"
				>
					<Search class="text-muted-foreground/50 h-12 w-12" />
					<p class="text-muted-foreground mt-4 text-sm">No modpacks found</p>
					{#if searchInput}
						<p class="text-muted-foreground mt-1 text-xs">Try a different search term</p>
					{:else if hasActiveFilters}
						<p class="text-muted-foreground mt-1 text-xs">Try adjusting your filters</p>
					{/if}
				</div>
			{:else}
				<div class="modpack-grid">
					{#each displayModpacks as modpack (`${modpack.platform}-${modpack.id}`)}
						<ModpackCard {modpack} onclick={() => handleModpackClick(modpack)} />
					{/each}
				</div>

				<!-- Loading indicator for infinite scroll -->
				{#if modpacksStore.isLoadingExplore && displayModpacks.length > 0}
					<div class="mt-6 flex items-center justify-center">
						<Loader2 class="text-muted-foreground h-6 w-6 animate-spin" />
						<span class="text-muted-foreground ml-2 text-sm">Loading more...</span>
					</div>
				{/if}

				<!-- End of results indicator -->
				{#if !modpacksStore.hasMoreExplore && !searchInput && displayModpacks.length > 0}
					<div class="text-muted-foreground mt-6 text-center text-xs">You've reached the end</div>
				{/if}
			{/if}
		</section>
	</div>
</div>

<!-- Modpack Detail Modal -->
{#if selectedModpackDetail}
	<ModpackDetailModalV2
		modpack={selectedModpackDetail}
		versions={modpacksStore.selectedModpackVersions}
		isLoadingVersions={modpacksStore.isLoadingVersions}
		isLoadingDetail={modpacksStore.isLoadingDetail}
		detailError={modpacksStore.detailError}
		installProgress={modpackInstallStore.progress}
		isInstalling={modpackInstallStore.isInstalling}
		isCancelling={modpackInstallStore.isCancelling}
		onClose={closeModpackDetail}
		onInstall={handleInstall}
		onCancelInstall={handleCancelInstall}
	/>
{/if}
