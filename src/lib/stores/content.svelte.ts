import { SvelteMap } from 'svelte/reactivity';
import type {
	Content,
	ContentDownloadProgress,
	ContentSearchParams,
	ContentVersion,
	ContentPlatform,
	ContentSortBy,
	ContentType,
	LoaderType,
	QueuedDownload,
	QueueItemStatus,
	ResolvedDependency,
	ScanResult,
} from '$lib/types';
import * as contentService from '$lib/services/content';

/** Cached results for a content type */
interface ContentTypeCache {
	items: Content[];
	totalCount: number;
	scanResult: ScanResult | null;
	cachedAt: number;
}

/** Create the content store */
function createContentStore() {
	// Search state
	let items = $state<Content[]>([]);
	let isSearching = $state(false);
	let searchError = $state<string | null>(null);
	let totalCount = $state(0);
	let currentPage = $state(0);
	const pageSize = $state(20);

	// Cache per platform + content type (persists across tab/platform switches)
	// Key format: "platform:contentType" e.g. "modrinth:mod"
	const contentCache = new SvelteMap<string, ContentTypeCache>();
	const CACHE_TTL = 5 * 60 * 1000; // 5 minutes

	/** Get cache key for current platform and content type */
	function getCacheKey(p: ContentPlatform, ct: ContentType): string {
		return `${p}:${ct}`;
	}

	// Filter state (can be set from instance context)
	let query = $state('');
	let platform = $state<ContentPlatform>('modrinth');
	let mcVersion = $state<string | null>(null);
	let loader = $state<LoaderType | null>(null);
	let isVanillaInstance = $state(false); // Track vanilla instance for datapack filtering
	let contentType = $state<ContentType>('mod');
	let category = $state<string | null>(null);
	let sortBy = $state<ContentSortBy>('relevance');

	// Instance context (for auto-filtering)
	let instanceId = $state<string | null>(null);

	// Selected content state
	let selectedContent = $state<Content | null>(null);
	let selectedContentVersions = $state<ContentVersion[]>([]);
	let selectedVersion = $state<ContentVersion | null>(null);
	let isLoadingDetails = $state(false);
	let isLoadingVersions = $state(false);

	// Installed content tracking (scan-based)
	let scanResult = $state<ScanResult | null>(null);
	let isScanning = $state(false);

	// Installation state
	let isInstalling = $state(false);
	let installError = $state<string | null>(null);

	// Download progress state (legacy single download)
	let downloadProgress = $state<ContentDownloadProgress | null>(null);

	// Download queue for parallel downloads
	let downloadQueue = $state<QueuedDownload[]>([]);

	// Resolved dependencies state
	let resolvedDependencies = $state<ResolvedDependency[]>([]);
	let isResolvingDeps = $state(false);

	// Selection tracking to prevent race conditions
	let currentSelectionId: string | null = null;

	return {
		// Search state getters
		get items() {
			return items;
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
		get contentType() {
			return contentType;
		},
		get category() {
			return category;
		},
		get sortBy() {
			return sortBy;
		},

		// Instance context getter
		get instanceId() {
			return instanceId;
		},

		// Selected content getters
		get selectedContent() {
			return selectedContent;
		},
		get selectedContentVersions() {
			return selectedContentVersions;
		},
		get selectedVersion() {
			return selectedVersion;
		},
		get isLoadingDetails() {
			return isLoadingDetails;
		},
		get isLoadingVersions() {
			return isLoadingVersions;
		},

		// Installed content getters (scan-based)
		get scanResult() {
			return scanResult;
		},
		get isScanning() {
			return isScanning;
		},

		// Installation state getters
		get isInstalling() {
			return isInstalling;
		},
		get installError() {
			return installError;
		},

		// Download progress getter (legacy)
		get downloadProgress() {
			return downloadProgress;
		},

		// Download queue getter
		get downloadQueue() {
			return downloadQueue;
		},

		// Resolved dependencies getters
		get resolvedDependencies() {
			return resolvedDependencies;
		},
		get isResolvingDeps() {
			return isResolvingDeps;
		},

		/** Set download progress (called from event listener) - legacy */
		setDownloadProgress(progress: ContentDownloadProgress | null) {
			downloadProgress = progress;
		},

		/** Check if a content item is in the download queue (pending or downloading) */
		isContentQueued(contentId: string): boolean {
			return downloadQueue.some(
				(item) =>
					item.content.id === contentId &&
					(item.status === 'pending' || item.status === 'downloading')
			);
		},

		/** Check if a content item is currently downloading */
		isContentDownloading(contentId: string): boolean {
			return downloadQueue.some(
				(item) => item.content.id === contentId && item.status === 'downloading'
			);
		},

		/** Get download progress for a specific content */
		getContentProgress(contentId: string): ContentDownloadProgress | null {
			const item = downloadQueue.find(
				(item) => item.content.id === contentId && item.status === 'downloading'
			);
			return item?.progress ?? null;
		},

		/** Add item to download queue */
		async queueInstall(content: Content, version: ContentVersion): Promise<string> {
			if (!instanceId || !mcVersion) {
				throw new Error('Instance context not set');
			}

			const queueId = crypto.randomUUID();

			// Add to local queue immediately (optimistic update)
			const queueItem: QueuedDownload = {
				queueId,
				content,
				version,
				instanceId,
				status: 'pending',
				queuedAt: Date.now(),
			};
			downloadQueue = [...downloadQueue, queueItem];

			// Send to backend
			// Only pass loader filter for mods - shaders/resourcepacks don't use mod loaders
			await contentService.queueContentInstall({
				queueId,
				instanceId,
				platform: content.platform,
				contentId: content.id,
				contentName: content.name,
				contentSlug: content.slug,
				contentType: content.contentType,
				versionId: version.id,
				versionName: version.versionNumber,
				mcVersion,
				loader: content.contentType === 'mod' ? loader || undefined : undefined,
			});

			return queueId;
		},

		/** Cancel a queued download */
		async cancelQueueItem(queueId: string): Promise<void> {
			await contentService.cancelContentQueueItem(queueId);
			downloadQueue = downloadQueue.filter((item) => item.queueId !== queueId);
		},

		/** Update queue item status (called from event listener)
		 * Also creates the queue item if it doesn't exist (for dependencies queued by backend)
		 */
		updateQueueItemStatus(
			queueId: string,
			contentId: string,
			contentName: string,
			itemContentType: ContentType,
			status: QueueItemStatus,
			error?: string
		) {
			// Check if item exists in local queue
			const exists = downloadQueue.some((item) => item.queueId === queueId);

			if (exists) {
				// Update existing item
				downloadQueue = downloadQueue.map((item) =>
					item.queueId === queueId ? { ...item, status, error } : item
				);
			} else {
				// Create new item for dependencies queued by backend
				// We don't have full Content/Version objects, so create minimal placeholders
				const newItem: QueuedDownload = {
					queueId,
					content: {
						id: contentId,
						name: contentName,
						slug: contentId,
						description: '',
						author: '',
						downloads: 0,
						iconUrl: undefined,
						categories: [],
						mcVersions: [],
						loaders: [],
						platform: 'modrinth', // Default, doesn't affect display
						contentType: itemContentType, // Use actual content type from backend
					},
					version: {
						id: '',
						projectId: contentId,
						name: '',
						versionNumber: '',
						mcVersions: [],
						loaders: [],
						files: [],
						dependencies: [],
					},
					instanceId: instanceId || '',
					status,
					error,
					queuedAt: Date.now(),
				};
				downloadQueue = [...downloadQueue, newItem];
			}

			// Remove completed/failed items after delay
			if (status === 'completed' || status === 'failed') {
				setTimeout(
					() => {
						downloadQueue = downloadQueue.filter((item) => item.queueId !== queueId);
					},
					status === 'completed' ? 1000 : 5000
				);
			}
		},

		/** Update download progress for a queue item */
		updateQueueItemProgress(queueId: string, progress: ContentDownloadProgress) {
			downloadQueue = downloadQueue.map((item) =>
				item.queueId === queueId ? { ...item, progress } : item
			);
		},

		/** Set the instance context (auto-filters by MC version and loader) */
		async setInstanceContext(id: string, mcVer: string, loaderType: LoaderType) {
			instanceId = id;
			mcVersion = mcVer;
			isVanillaInstance = loaderType === 'vanilla';
			loader = isVanillaInstance ? null : loaderType;
			// Clear content type cache since filters changed
			contentCache.clear();
			// Scan installed content for this instance
			await this.scanInstalledContent();
		},

		/** Scan installed content for the current content type
		 * @param silent - If true, don't show scanning indicator (used for background refreshes)
		 */
		async scanInstalledContent(silent = false) {
			if (!instanceId) return;
			if (!silent) {
				isScanning = true;
			}
			try {
				scanResult = await contentService.scanInstalledContent(instanceId, contentType);
			} catch (e) {
				console.error('Failed to scan installed content:', e);
			} finally {
				if (!silent) {
					isScanning = false;
				}
			}
		},

		/** Normalize slug for cross-platform matching */
		normalizeSlug(slug: string): string {
			return slug.toLowerCase().replace(/-api$/g, '').replace(/_/g, '-');
		},

		/** Check if content is installed in the current instance */
		isContentInstalled(content: Content): boolean {
			if (!scanResult) return false;

			// Check if any scanned item matches this content's Modrinth ID
			if (content.platform === 'modrinth') {
				const directMatch = scanResult.items.some(
					(item) => item.modrinthProject?.projectId === content.id
				);
				if (directMatch) return true;

				// Fallback: check if any CurseForge project matches by slug
				const normalizedContentSlug = this.normalizeSlug(content.slug);
				return scanResult.items.some(
					(item) =>
						item.curseforgeProject?.slug &&
						this.normalizeSlug(item.curseforgeProject.slug) === normalizedContentSlug
				);
			}

			// Check if any scanned item matches this content's CurseForge ID
			if (content.platform === 'curseforge') {
				const contentIdNum = parseInt(content.id, 10);
				const directMatch = scanResult.items.some(
					(item) => item.curseforgeProject?.projectId === contentIdNum
				);
				if (directMatch) return true;

				// Fallback: check if any Modrinth project matches by slug
				const normalizedContentSlug = this.normalizeSlug(content.slug);
				return scanResult.items.some(
					(item) =>
						item.modrinthProject?.slug &&
						this.normalizeSlug(item.modrinthProject.slug) === normalizedContentSlug
				);
			}

			return false;
		},

		/** Set the selected version and resolve dependencies */
		async setSelectedVersion(version: ContentVersion | null, platform?: ContentPlatform) {
			selectedVersion = version;
			if (version && platform) {
				await this.resolveDependencies(version, platform);
			} else {
				resolvedDependencies = [];
			}
		},

		/** Resolve dependencies for a version */
		async resolveDependencies(version: ContentVersion, platform: ContentPlatform) {
			if (!instanceId || !mcVersion) {
				resolvedDependencies = [];
				return;
			}

			// Only resolve if there are required dependencies
			const hasRequiredDeps = version.dependencies.some((d) => d.dependencyType === 'required');
			if (!hasRequiredDeps) {
				resolvedDependencies = [];
				return;
			}

			isResolvingDeps = true;
			try {
				resolvedDependencies = await contentService.resolveContentDependencies(
					instanceId,
					platform,
					version,
					mcVersion,
					loader || undefined
				);
			} catch (e) {
				console.error('Failed to resolve dependencies:', e);
				resolvedDependencies = [];
			} finally {
				isResolvingDeps = false;
			}
		},

		/** Refresh installed content by re-scanning (silent, no loading indicator) */
		async refreshInstalledContent() {
			await this.scanInstalledContent(true);
		},

		/** Clear the instance context */
		clearInstanceContext() {
			instanceId = null;
		},

		/** Search for content with current filters */
		async search(resetPage = true) {
			if (resetPage) {
				currentPage = 0;
			}

			isSearching = true;
			searchError = null;

			try {
				// Only mods use the instance loader filter (Fabric/Forge/etc).
				// CurseForge resource packs don't have modLoaderType, and shaders use different ecosystems.
				const loaderFilter = contentType === 'mod' ? loader || undefined : undefined;

				const params: ContentSearchParams = {
					query: query || undefined,
					platform: platform || undefined,
					mcVersion: mcVersion || undefined,
					loader: loaderFilter,
					contentType,
					category: category || undefined,
					sortBy,
					page: currentPage,
					pageSize,
				};

				const result = await contentService.searchContent(params);
				items = result.items;
				totalCount = result.totalCount;
			} catch (e: unknown) {
				searchError =
					e instanceof Error ? e.message : typeof e === 'string' ? e : JSON.stringify(e);
				console.error('Failed to search content:', e);
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
				const loaderFilter = contentType === 'mod' ? loader || undefined : undefined;

				const params: ContentSearchParams = {
					query: query || undefined,
					platform: platform || undefined,
					mcVersion: mcVersion || undefined,
					loader: loaderFilter,
					contentType,
					category: category || undefined,
					sortBy,
					page: currentPage,
					pageSize,
				};

				const result = await contentService.searchContent(params);
				items = [...items, ...result.items];
				totalCount = result.totalCount;
			} catch (e: unknown) {
				searchError =
					e instanceof Error ? e.message : typeof e === 'string' ? e : JSON.stringify(e);
				console.error('Failed to load more content:', e);
			} finally {
				isSearching = false;
			}
		},

		/** Set search query */
		setQuery(newQuery: string) {
			query = newQuery;
		},

		/** Set platform filter */
		setPlatform(newPlatform: ContentPlatform) {
			if (platform !== newPlatform) {
				// Save current results to cache before switching
				if (items.length > 0) {
					contentCache.set(getCacheKey(platform, contentType), {
						items: [...items],
						totalCount,
						scanResult,
						cachedAt: Date.now(),
					});
				}

				platform = newPlatform;

				// Check if we have cached data for the new platform
				const cached = contentCache.get(getCacheKey(newPlatform, contentType));
				if (cached && Date.now() - cached.cachedAt < CACHE_TTL) {
					items = cached.items;
					totalCount = cached.totalCount;
					// Keep scanResult as-is (it's per-instance, not per-platform)
					isSearching = false;
				} else {
					// No cache - will need to search
					items = [];
					totalCount = 0;
					isSearching = true;
				}
			}
		},

		/** Set content type filter and trigger scan */
		async setContentType(type: ContentType) {
			// Save current results to cache before switching
			if (items.length > 0) {
				contentCache.set(getCacheKey(platform, contentType), {
					items: [...items],
					totalCount,
					scanResult,
					cachedAt: Date.now(),
				});
			}

			contentType = type;
			currentPage = 0;

			// Check if we have valid cached data for the new content type
			const cached = contentCache.get(getCacheKey(platform, type));
			if (cached && Date.now() - cached.cachedAt < CACHE_TTL) {
				// Restore from cache - instant switch!
				items = cached.items;
				totalCount = cached.totalCount;
				scanResult = cached.scanResult;
				isSearching = false;
			} else {
				// No cache - show loading state
				items = [];
				scanResult = null;
				totalCount = 0;
				isSearching = true;
				// Trigger a new scan for this content type (silent to avoid flickering)
				await this.scanInstalledContent(true);
			}
		},

		/** Set category filter */
		setCategory(newCategory: string | null) {
			category = newCategory;
		},

		/** Set sort order */
		setSortBy(newSortBy: ContentSortBy) {
			sortBy = newSortBy;
		},

		/** Clear all filters (except instance context) */
		clearFilters() {
			query = '';
			platform = 'modrinth';
			category = null;
			sortBy = 'relevance';
			contentType = 'mod';
			// Keep mcVersion and loader from instance context
		},

		/** Select content and load its versions/details in parallel */
		async selectContent(content: Content) {
			// Track this selection to prevent race conditions
			const selectionId = content.id;
			currentSelectionId = selectionId;

			// Show basic content immediately
			selectedContent = content;
			selectedContentVersions = [];
			selectedVersion = null;
			resolvedDependencies = [];
			isLoadingDetails = true;
			isLoadingVersions = true;

			const shouldFilterByLoader = content.contentType === 'mod';
			// For mods on vanilla instances, filter by 'datapack' to get datapack-compatible versions
			let versionLoader: LoaderType | undefined;
			if (shouldFilterByLoader) {
				if (isVanillaInstance) {
					versionLoader = 'datapack';
				} else {
					versionLoader = loader || undefined;
				}
			}

			// Load details and versions in parallel
			const detailsPromise = contentService
				.getContent(content.platform, content.id)
				.then((detailed) => {
					// Only update if this is still the current selection
					if (currentSelectionId === selectionId) {
						selectedContent = { ...content, ...detailed };
						isLoadingDetails = false;
					}
				})
				.catch((e) => {
					console.error('Failed to load detailed content info:', e);
					if (currentSelectionId === selectionId) {
						isLoadingDetails = false;
					}
				});

			const versionsPromise = contentService
				.getContentVersions(content.platform, content.id, mcVersion || undefined, versionLoader)
				.then((versions) => {
					// Only update if this is still the current selection
					if (currentSelectionId === selectionId) {
						selectedContentVersions = versions;
						isLoadingVersions = false;
						// Auto-select first version and resolve dependencies (non-blocking)
						if (versions.length > 0) {
							this.setSelectedVersion(versions[0], content.platform);
						}
					}
				})
				.catch((e) => {
					console.error('Failed to load content versions:', e);
					if (currentSelectionId === selectionId) {
						isLoadingVersions = false;
					}
				});

			// Wait for both to complete (but they update UI as they finish)
			await Promise.all([detailsPromise, versionsPromise]);

			return selectedContent;
		},

		/** Clear selected content */
		clearSelection() {
			currentSelectionId = null;
			selectedContent = null;
			selectedContentVersions = [];
			selectedVersion = null;
			resolvedDependencies = [];
			isLoadingDetails = false;
			isLoadingVersions = false;
		},

		/** Clear search error */
		clearSearchError() {
			searchError = null;
		},

		/** Clear install error */
		clearInstallError() {
			installError = null;
		},

		/** Reset the store to initial state */
		reset() {
			items = [];
			isSearching = false;
			searchError = null;
			totalCount = 0;
			currentPage = 0;
			query = '';
			platform = 'modrinth';
			mcVersion = null;
			loader = null;
			isVanillaInstance = false;
			contentType = 'mod';
			category = null;
			sortBy = 'relevance';
			instanceId = null;
			currentSelectionId = null;
			selectedContent = null;
			selectedContentVersions = [];
			selectedVersion = null;
			isLoadingDetails = false;
			isLoadingVersions = false;
			scanResult = null;
			isScanning = false;
			isInstalling = false;
			installError = null;
			downloadProgress = null;
			downloadQueue = [];
			resolvedDependencies = [];
			isResolvingDeps = false;
			contentCache.clear();
		},
	};
}

/** Global content store instance */
export const contentStore = createContentStore();
