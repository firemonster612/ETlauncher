import type {
	Instance,
	Modpack,
	ModpackSearchParams,
	ModpackVersion,
	ModpackPlatform,
	ModpackSortBy,
	LoaderType,
} from '$lib/types';
import * as modpackService from '$lib/services/modpack';

/** Create the modpacks store */
function createModpacksStore() {
	// Search state
	let modpacks = $state<Modpack[]>([]);
	let isSearching = $state(false);
	let searchError = $state<string | null>(null);
	let totalCount = $state(0);
	let currentPage = $state(0);
	const pageSize = $state(20);

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
			platform = newPlatform;
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

		/** Select a modpack and load its versions */
		async selectModpack(modpack: Modpack): Promise<Modpack> {
			const shouldLoadDetail =
				modpack.platform === 'modrinth' ||
				modpack.platform === 'curseforge' ||
				modpack.platform === 'ftb' ||
				modpack.platform === 'technic';
			selectedModpack = modpack;
			selectedModpackVersions = [];
			detailError = null;
			isLoadingDetail = shouldLoadDetail;
			isLoadingVersions = true;

			if (shouldLoadDetail) {
				try {
					const detailed = await modpackService.getModpack(modpack.platform, modpack.id);
					const mergedGallery =
						(detailed.gallery?.length ?? 0) > 0 ? detailed.gallery : modpack.gallery;
					selectedModpack = { ...modpack, ...detailed, gallery: mergedGallery };
				} catch (e: unknown) {
					console.error('[modpacksStore] Failed to load detailed modpack info:', e);
					detailError =
						e instanceof Error ? e.message : typeof e === 'string' ? e : JSON.stringify(e);
					selectedModpack = modpack;
				} finally {
					isLoadingDetail = false;
				}
			}

			try {
				console.log(
					'[modpacksStore] Loading versions for modpack:',
					modpack.name,
					modpack.platform
				);
				selectedModpackVersions = await modpackService.getModpackVersions(
					modpack.platform,
					modpack.id
				);
				console.log('[modpacksStore] Loaded versions:', selectedModpackVersions.length);
			} catch (e: unknown) {
				console.error('[modpacksStore] Failed to load modpack versions:', e);
			} finally {
				isLoadingVersions = false;
			}

			return selectedModpack ?? modpack;
		},

		/** Clear selected modpack */
		clearSelection() {
			selectedModpack = null;
			selectedModpackVersions = [];
			detailError = null;
			isLoadingDetail = false;
		},

		/** Clear search error */
		clearSearchError() {
			searchError = null;
		},

		/** Clear install error */
		clearInstallError() {
			installError = null;
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
