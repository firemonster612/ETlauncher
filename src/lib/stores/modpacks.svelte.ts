import { SvelteMap } from 'svelte/reactivity';
import type {
	Instance,
	Modpack,
	ModpackMod,
	ModpackSearchParams,
	ModpackVersion,
	ModpackPlatform,
	ModpackSortBy,
	LoaderType,
	SideFilter,
} from '$lib/types';
import * as modpackService from '$lib/services/modpack';
import * as minecraftService from '$lib/services/minecraft';

/** Cached results for a platform */
interface PlatformCache {
	modpacks: Modpack[];
	totalCount: number;
	cachedAt: number;
}

/** Create the modpacks store */
function createModpacksStore() {
	// Search state
	let modpacks = $state<Modpack[]>([]);
	let isSearching = $state(false);
	let searchError = $state<string | null>(null);
	let totalCount = $state(0);
	let currentPage = $state(0);
	const pageSize = $state(20);

	// Home page data
	let popularModpacks = $state<Modpack[]>([]);
	let recentModpacks = $state<Modpack[]>([]);
	let latestVersionModpacks = $state<Modpack[]>([]);
	let risingStarsModpacks = $state<Modpack[]>([]);
	let isLoadingPopular = $state(false);
	let isLoadingRecent = $state(false);
	let isLoadingLatestVersion = $state(false);
	let isLoadingRisingStars = $state(false);
	let homeDataLoaded = $state(false);
	let latestMcVersion = $state<string | null>(null);

	// Explore section state
	let exploreModpacks = $state<Modpack[]>([]);
	let isLoadingExplore = $state(false);
	let exploreTotalCount = $state(0);
	let exploreCurrentPage = $state(0);
	let explorePlatform = $state<ModpackPlatform | null>(null);
	let exploreSortBy = $state<ModpackSortBy>('downloads');
	let exploreMcVersion = $state<string | null>(null);
	let exploreLoader = $state<LoaderType | null>(null);
	let exploreCategory = $state<string | null>(null);
	let exploreSide = $state<SideFilter | null>(null);
	const explorePageSize = 20;

	// Cache per platform (persists across platform switches)
	const platformCache = new SvelteMap<string, PlatformCache>();
	const CACHE_TTL = 5 * 60 * 1000; // 5 minutes

	/** Get cache key for current platform */
	function getCacheKey(p: ModpackPlatform | null): string {
		return p ?? 'all';
	}

	// Filter state
	let query = $state('');
	let platform = $state<ModpackPlatform | null>(null);
	let mcVersion = $state<string | null>(null);
	let loader = $state<LoaderType | null>(null);
	let category = $state<string | null>(null);
	let sortBy = $state<ModpackSortBy>('relevance');

	// Selected modpack state
	let selectedModpack = $state<Modpack | null>(null);
	let selectedModpackVersions = $state<ModpackVersion[]>([]);
	let isLoadingDetail = $state(false);
	let detailError = $state<string | null>(null);
	let isLoadingVersions = $state(false);

	// Installation state
	let isInstalling = $state(false);
	let installError = $state<string | null>(null);

	// Modpack contents state (mods, shaders, resource packs)
	let selectedModpackMods = $state<ModpackMod[]>([]);
	let isLoadingMods = $state(false);
	let modsError = $state<string | null>(null);

	// Selection tracking to prevent race conditions
	let currentSelectionId: string | null = null;

	return {
		// Search state getters
		get modpacks() {
			return modpacks;
		},
		get isSearching() {
			return isSearching;
		},
		get searchError() {
			return searchError;
		},
		get totalCount() {
			return totalCount;
		},
		get currentPage() {
			return currentPage;
		},
		get pageSize() {
			return pageSize;
		},
		get hasMore() {
			return (currentPage + 1) * pageSize < totalCount;
		},

		// Home page data getters
		get popularModpacks() {
			return popularModpacks;
		},
		get recentModpacks() {
			return recentModpacks;
		},
		get latestVersionModpacks() {
			return latestVersionModpacks;
		},
		get risingStarsModpacks() {
			return risingStarsModpacks;
		},
		get isLoadingPopular() {
			return isLoadingPopular;
		},
		get isLoadingRecent() {
			return isLoadingRecent;
		},
		get isLoadingLatestVersion() {
			return isLoadingLatestVersion;
		},
		get isLoadingRisingStars() {
			return isLoadingRisingStars;
		},
		get homeDataLoaded() {
			return homeDataLoaded;
		},
		get latestMcVersion() {
			return latestMcVersion;
		},

		// Explore section getters
		get exploreModpacks() {
			return exploreModpacks;
		},
		get isLoadingExplore() {
			return isLoadingExplore;
		},
		get exploreTotalCount() {
			return exploreTotalCount;
		},
		get exploreCurrentPage() {
			return exploreCurrentPage;
		},
		get explorePlatform() {
			return explorePlatform;
		},
		get exploreSortBy() {
			return exploreSortBy;
		},
		get hasMoreExplore() {
			return (exploreCurrentPage + 1) * explorePageSize < exploreTotalCount;
		},
		get exploreMcVersion() {
			return exploreMcVersion;
		},
		get exploreLoader() {
			return exploreLoader;
		},
		get exploreCategory() {
			return exploreCategory;
		},
		get exploreSide() {
			return exploreSide;
		},

		// Filter state getters
		get query() {
			return query;
		},
		get platform() {
			return platform;
		},
		get mcVersion() {
			return mcVersion;
		},
		get loader() {
			return loader;
		},
		get category() {
			return category;
		},
		get sortBy() {
			return sortBy;
		},

		// Selected modpack getters
		get selectedModpack() {
			return selectedModpack;
		},
		get selectedModpackVersions() {
			return selectedModpackVersions;
		},
		get isLoadingDetail() {
			return isLoadingDetail;
		},
		get detailError() {
			return detailError;
		},
		get isLoadingVersions() {
			return isLoadingVersions;
		},

		// Installation state getters
		get isInstalling() {
			return isInstalling;
		},
		get installError() {
			return installError;
		},

		// Modpack contents getters
		get selectedModpackMods() {
			return selectedModpackMods;
		},
		get isLoadingMods() {
			return isLoadingMods;
		},
		get modsError() {
			return modsError;
		},

		/** Search for modpacks with current filters */
		async search(resetPage = true) {
			if (resetPage) {
				currentPage = 0;
			}

			isSearching = true;
			searchError = null;

			try {
				const params: ModpackSearchParams = {
					query: query || undefined,
					platform: platform || undefined,
					mcVersion: mcVersion || undefined,
					loader: loader || undefined,
					category: category || undefined,
					sortBy,
					page: currentPage,
					pageSize,
				};

				console.log('[modpacksStore] Searching modpacks with params:', params);
				const result = await modpackService.searchModpacks(params);
				console.log('[modpacksStore] Search result:', {
					count: result.modpacks.length,
					total: result.totalCount,
				});
				modpacks = result.modpacks;
				totalCount = result.totalCount;
			} catch (e: unknown) {
				console.error('[modpacksStore] Search failed:', e);
				searchError =
					e instanceof Error ? e.message : typeof e === 'string' ? e : JSON.stringify(e);
			} finally {
				isSearching = false;
			}
		},

		/** Load home page data (popular, recent, latest version, and rising stars modpacks) */
		async loadHomeData() {
			if (homeDataLoaded) return;

			// First, fetch the latest MC version from manifest
			try {
				const manifest = await minecraftService.fetchVersionManifest(false);
				if (manifest?.latest?.release) {
					latestMcVersion = manifest.latest.release;
					console.log('[modpacksStore] Latest MC version:', latestMcVersion);
				}
			} catch (e: unknown) {
				console.error('[modpacksStore] Failed to fetch MC versions:', e);
				latestMcVersion = '1.21.4'; // Fallback
			}

			// Load all sections in parallel
			const loadPopular = async () => {
				isLoadingPopular = true;
				try {
					const result = await modpackService.searchModpacks({
						sortBy: 'downloads',
						page: 0,
						pageSize: 8,
					});
					popularModpacks = result.modpacks;
				} catch (e: unknown) {
					console.error('[modpacksStore] Failed to load popular modpacks:', e);
				} finally {
					isLoadingPopular = false;
				}
			};

			const loadRecent = async () => {
				isLoadingRecent = true;
				try {
					// Use modrinth as default platform for recent updates since it supports this sort well
					const result = await modpackService.searchModpacks({
						platform: 'modrinth',
						sortBy: 'recentlyUpdated',
						page: 0,
						pageSize: 10,
					});
					recentModpacks = result.modpacks;
				} catch (e: unknown) {
					console.error('[modpacksStore] Failed to load recent modpacks:', e);
				} finally {
					isLoadingRecent = false;
				}
			};

			const loadLatestVersion = async () => {
				if (!latestMcVersion) return;
				isLoadingLatestVersion = true;
				try {
					// Use recentlyUpdated to show different modpacks than Popular section
					const result = await modpackService.searchModpacks({
						mcVersion: latestMcVersion,
						sortBy: 'recentlyUpdated',
						page: 0,
						pageSize: 12,
					});
					latestVersionModpacks = result.modpacks;
				} catch (e: unknown) {
					console.error('[modpacksStore] Failed to load latest version modpacks:', e);
				} finally {
					isLoadingLatestVersion = false;
				}
			};

			const loadRisingStars = async () => {
				isLoadingRisingStars = true;
				try {
					// Use "newest" sort to get recently created modpacks (rising stars)
					const result = await modpackService.searchModpacks({
						platform: 'modrinth',
						sortBy: 'newest',
						page: 0,
						pageSize: 5,
					});
					risingStarsModpacks = result.modpacks;
				} catch (e: unknown) {
					console.error('[modpacksStore] Failed to load rising stars modpacks:', e);
				} finally {
					isLoadingRisingStars = false;
				}
			};

			await Promise.all([loadPopular(), loadRecent(), loadLatestVersion(), loadRisingStars()]);
			homeDataLoaded = true;
		},

		/** Load explore section data */
		async loadExploreData(resetPage = true) {
			if (resetPage) {
				exploreCurrentPage = 0;
				// Don't clear modpacks immediately to prevent UI jump
			}

			isLoadingExplore = true;

			try {
				const result = await modpackService.searchModpacks({
					platform: explorePlatform || undefined,
					mcVersion: exploreMcVersion || undefined,
					loader: exploreLoader || undefined,
					category: exploreCategory || undefined,
					side: exploreSide || undefined,
					sortBy: exploreSortBy,
					page: exploreCurrentPage,
					pageSize: explorePageSize,
				});
				console.log('[modpacksStore] Explore result:', {
					count: result.modpacks.length,
					total: result.totalCount,
				});
				exploreModpacks = result.modpacks;
				exploreTotalCount = result.totalCount;
			} catch (e: unknown) {
				console.error('[modpacksStore] Failed to load explore modpacks:', e);
			} finally {
				isLoadingExplore = false;
			}
		},

		/** Load more explore results */
		async loadMoreExplore() {
			if (!this.hasMoreExplore || isLoadingExplore) return;

			exploreCurrentPage++;
			isLoadingExplore = true;

			try {
				const result = await modpackService.searchModpacks({
					platform: explorePlatform || undefined,
					mcVersion: exploreMcVersion || undefined,
					loader: exploreLoader || undefined,
					category: exploreCategory || undefined,
					side: exploreSide || undefined,
					sortBy: exploreSortBy,
					page: exploreCurrentPage,
					pageSize: explorePageSize,
				});
				console.log('[modpacksStore] Loaded more explore:', result.modpacks.length);
				exploreModpacks = [...exploreModpacks, ...result.modpacks];
				exploreTotalCount = result.totalCount;
			} catch (e: unknown) {
				console.error('[modpacksStore] Load more explore failed:', e);
			} finally {
				isLoadingExplore = false;
			}
		},

		/** Set explore platform filter */
		setExplorePlatform(newPlatform: ModpackPlatform | null) {
			explorePlatform = newPlatform;
		},

		/** Set explore sort order */
		setExploreSortBy(newSortBy: ModpackSortBy) {
			exploreSortBy = newSortBy;
		},

		/** Set explore MC version filter */
		setExploreMcVersion(version: string | null) {
			exploreMcVersion = version;
		},

		/** Set explore loader filter */
		setExploreLoader(newLoader: LoaderType | null) {
			exploreLoader = newLoader;
		},

		/** Set explore category filter */
		setExploreCategory(newCategory: string | null) {
			exploreCategory = newCategory;
		},

		/** Set explore side filter (client/server) */
		setExploreSide(newSide: SideFilter | null) {
			exploreSide = newSide;
		},

		/** Clear all explore filters */
		clearExploreFilters() {
			explorePlatform = null;
			exploreMcVersion = null;
			exploreLoader = null;
			exploreCategory = null;
			exploreSide = null;
			exploreSortBy = 'downloads';
		},

		/** Load more results (next page) */
		async loadMore() {
			if (!this.hasMore || isSearching) return;

			currentPage++;
			isSearching = true;

			try {
				const params: ModpackSearchParams = {
					query: query || undefined,
					platform: platform || undefined,
					mcVersion: mcVersion || undefined,
					loader: loader || undefined,
					category: category || undefined,
					sortBy,
					page: currentPage,
					pageSize,
				};

				console.log('[modpacksStore] Loading more modpacks, page:', currentPage);
				const result = await modpackService.searchModpacks(params);
				console.log('[modpacksStore] Loaded more:', result.modpacks.length);
				modpacks = [...modpacks, ...result.modpacks];
				totalCount = result.totalCount;
			} catch (e: unknown) {
				console.error('[modpacksStore] Load more failed:', e);
				searchError =
					e instanceof Error ? e.message : typeof e === 'string' ? e : JSON.stringify(e);
			} finally {
				isSearching = false;
			}
		},

		/** Set search query */
		setQuery(newQuery: string) {
			query = newQuery;
		},

		/** Set platform filter */
		setPlatform(newPlatform: ModpackPlatform | null) {
			if (platform !== newPlatform) {
				// Save current results to cache before switching
				if (modpacks.length > 0 && !query) {
					platformCache.set(getCacheKey(platform), {
						modpacks: [...modpacks],
						totalCount,
						cachedAt: Date.now(),
					});
				}

				platform = newPlatform;

				// Check if we have cached data for the new platform (only for default view)
				if (!query) {
					const cached = platformCache.get(getCacheKey(newPlatform));
					if (cached && Date.now() - cached.cachedAt < CACHE_TTL) {
						modpacks = cached.modpacks;
						totalCount = cached.totalCount;
						isSearching = false;
						return;
					}
				}

				// No cache - will need to search
				modpacks = [];
				totalCount = 0;
				isSearching = true;
			}
		},

		/** Set Minecraft version filter */
		setMcVersion(version: string | null) {
			mcVersion = version;
		},

		/** Set loader filter */
		setLoader(newLoader: LoaderType | null) {
			loader = newLoader;
		},

		/** Set category filter */
		setCategory(newCategory: string | null) {
			category = newCategory;
		},

		/** Set sort order */
		setSortBy(newSortBy: ModpackSortBy) {
			sortBy = newSortBy;
		},

		/** Clear all filters */
		clearFilters() {
			query = '';
			platform = null;
			mcVersion = null;
			loader = null;
			category = null;
			sortBy = 'relevance';
		},

		/** Select a modpack and load its details/versions in parallel */
		async selectModpack(modpack: Modpack): Promise<Modpack> {
			// Track this selection to prevent race conditions
			const selectionId = modpack.id;
			currentSelectionId = selectionId;

			const shouldLoadDetail =
				modpack.platform === 'modrinth' ||
				modpack.platform === 'curseforge' ||
				modpack.platform === 'ftb' ||
				modpack.platform === 'technic';

			// Show basic modpack immediately
			selectedModpack = modpack;
			selectedModpackVersions = [];
			detailError = null;
			isLoadingDetail = shouldLoadDetail;
			isLoadingVersions = true;

			// Load details and versions in parallel
			const detailsPromise = shouldLoadDetail
				? modpackService
						.getModpack(modpack.platform, modpack.id)
						.then((detailed) => {
							// Only update if this is still the current selection
							if (currentSelectionId === selectionId) {
								const mergedGallery =
									(detailed.gallery?.length ?? 0) > 0 ? detailed.gallery : modpack.gallery;
								selectedModpack = { ...modpack, ...detailed, gallery: mergedGallery };
								isLoadingDetail = false;
							}
						})
						.catch((e) => {
							console.error('[modpacksStore] Failed to load detailed modpack info:', e);
							if (currentSelectionId === selectionId) {
								detailError =
									e instanceof Error ? e.message : typeof e === 'string' ? e : JSON.stringify(e);
								isLoadingDetail = false;
							}
						})
				: Promise.resolve();

			const versionsPromise = modpackService
				.getModpackVersions(modpack.platform, modpack.id)
				.then((versions) => {
					// Only update if this is still the current selection
					if (currentSelectionId === selectionId) {
						selectedModpackVersions = versions;
						isLoadingVersions = false;
						console.log('[modpacksStore] Loaded versions:', versions.length);
					}
				})
				.catch((e) => {
					console.error('[modpacksStore] Failed to load modpack versions:', e);
					if (currentSelectionId === selectionId) {
						isLoadingVersions = false;
					}
				});

			// Wait for both to complete (but they update UI as they finish)
			await Promise.all([detailsPromise, versionsPromise]);

			return selectedModpack ?? modpack;
		},

		/** Clear selected modpack */
		clearSelection() {
			currentSelectionId = null;
			selectedModpack = null;
			selectedModpackVersions = [];
			selectedModpackMods = [];
			detailError = null;
			modsError = null;
			isLoadingDetail = false;
			isLoadingVersions = false;
			isLoadingMods = false;
		},

		/** Clear search error */
		clearSearchError() {
			searchError = null;
		},

		/** Clear install error */
		clearInstallError() {
			installError = null;
		},

		/** Clear mods error */
		clearModsError() {
			modsError = null;
		},

		/** Load mods/contents for a modpack version */
		async loadMods(
			modpackPlatform: ModpackPlatform,
			modpackId: string,
			versionId: string
		): Promise<void> {
			isLoadingMods = true;
			modsError = null;
			selectedModpackMods = [];

			try {
				console.log('[modpacksStore] Loading mods for:', {
					platform: modpackPlatform,
					modpackId,
					versionId,
				});
				const mods = await modpackService.getModpackMods(modpackPlatform, modpackId, versionId);
				selectedModpackMods = mods;
				console.log('[modpacksStore] Loaded mods:', mods.length);
			} catch (e: unknown) {
				console.error('[modpacksStore] Load mods failed:', e);
				modsError = e instanceof Error ? e.message : typeof e === 'string' ? e : JSON.stringify(e);
			} finally {
				isLoadingMods = false;
			}
		},

		/** Install a modpack and create a new instance */
		async installModpack(
			modpackPlatform: ModpackPlatform,
			modpackId: string,
			versionId: string,
			instanceName?: string
		): Promise<Instance | null> {
			isInstalling = true;
			installError = null;

			try {
				console.log('[modpacksStore] Installing modpack:', {
					platform: modpackPlatform,
					modpackId,
					versionId,
					instanceName,
				});

				const instance = await modpackService.installModpack(
					modpackPlatform,
					modpackId,
					versionId,
					instanceName
				);

				console.log(
					'[modpacksStore] Modpack installed successfully, created instance:',
					instance.id
				);
				return instance;
			} catch (e: unknown) {
				console.error('[modpacksStore] Install failed:', e);
				installError =
					e instanceof Error ? e.message : typeof e === 'string' ? e : JSON.stringify(e);
				return null;
			} finally {
				isInstalling = false;
			}
		},
	};
}

/** Global modpacks store instance */
export const modpacksStore = createModpacksStore();
