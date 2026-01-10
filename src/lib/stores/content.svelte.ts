import type {
  Content,
  ContentDownloadProgress,
  ContentDownloadProgressWithId,
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
} from "$lib/types";
import * as contentService from "$lib/services/content";

/** Create the content store */
function createContentStore() {
  // Search state
  let items = $state<Content[]>([]);
  let isSearching = $state(false);
  let searchError = $state<string | null>(null);
  let totalCount = $state(0);
  let currentPage = $state(0);
  const pageSize = $state(20);

  // Filter state (can be set from instance context)
  let query = $state("");
  let platform = $state<ContentPlatform>("modrinth");
  let mcVersion = $state<string | null>(null);
  let loader = $state<LoaderType | null>(null);
  let contentType = $state<ContentType>("mod");
  let category = $state<string | null>(null);
  let sortBy = $state<ContentSortBy>("relevance");

  // Instance context (for auto-filtering)
  let instanceId = $state<string | null>(null);

  // Selected content state
  let selectedContent = $state<Content | null>(null);
  let selectedContentVersions = $state<ContentVersion[]>([]);
  let selectedVersion = $state<ContentVersion | null>(null);
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
          (item.status === "pending" || item.status === "downloading")
      );
    },

    /** Check if a content item is currently downloading */
    isContentDownloading(contentId: string): boolean {
      return downloadQueue.some(
        (item) => item.content.id === contentId && item.status === "downloading"
      );
    },

    /** Get download progress for a specific content */
    getContentProgress(contentId: string): ContentDownloadProgress | null {
      const item = downloadQueue.find(
        (item) => item.content.id === contentId && item.status === "downloading"
      );
      return item?.progress ?? null;
    },

    /** Add item to download queue */
    async queueInstall(content: Content, version: ContentVersion): Promise<string> {
      if (!instanceId || !mcVersion) {
        throw new Error("Instance context not set");
      }

      const queueId = crypto.randomUUID();

      // Add to local queue immediately (optimistic update)
      const queueItem: QueuedDownload = {
        queueId,
        content,
        version,
        instanceId,
        status: "pending",
        queuedAt: Date.now(),
      };
      downloadQueue = [...downloadQueue, queueItem];

      // Send to backend
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
        loader: loader || undefined,
      });

      return queueId;
    },

    /** Cancel a queued download */
    async cancelQueueItem(queueId: string): Promise<void> {
      await contentService.cancelContentQueueItem(queueId);
      downloadQueue = downloadQueue.filter((item) => item.queueId !== queueId);
    },

    /** Update queue item status (called from event listener) */
    updateQueueItemStatus(queueId: string, status: QueueItemStatus, error?: string) {
      downloadQueue = downloadQueue.map((item) =>
        item.queueId === queueId ? { ...item, status, error } : item
      );

      // Remove completed/failed items after delay
      if (status === "completed" || status === "failed") {
        setTimeout(() => {
          downloadQueue = downloadQueue.filter((item) => item.queueId !== queueId);
        }, status === "completed" ? 1000 : 5000);
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
      loader = loaderType === "vanilla" ? null : loaderType;
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
        console.error("Failed to scan installed content:", e);
      } finally {
        if (!silent) {
          isScanning = false;
        }
      }
    },

    /** Normalize slug for cross-platform matching */
    normalizeSlug(slug: string): string {
      return slug
        .toLowerCase()
        .replace(/-api$/g, "")
        .replace(/_/g, "-");
    },

    /** Check if content is installed in the current instance */
    isContentInstalled(content: Content): boolean {
      if (!scanResult) return false;

      // Check if any scanned item matches this content's Modrinth ID
      if (content.platform === "modrinth") {
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
      if (content.platform === "curseforge") {
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
      const hasRequiredDeps = version.dependencies.some(
        (d) => d.dependencyType === "required"
      );
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
        console.error("Failed to resolve dependencies:", e);
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
        const loaderFilter = contentType === "mod" ? loader || undefined : undefined;

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
        searchError = e instanceof Error ? e.message : (typeof e === "string" ? e : JSON.stringify(e));
        console.error("Failed to search content:", e);
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
        const loaderFilter = contentType === "mod" ? loader || undefined : undefined;

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
        searchError = e instanceof Error ? e.message : (typeof e === "string" ? e : JSON.stringify(e));
        console.error("Failed to load more content:", e);
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
      platform = newPlatform;
    },

    /** Set content type filter and trigger scan */
    async setContentType(type: ContentType) {
      contentType = type;
      // Clear scan result when switching content types
      scanResult = null;
      // Trigger a new scan for this content type (silent to avoid flickering)
      await this.scanInstalledContent(true);
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
      query = "";
      platform = "modrinth";
      category = null;
      sortBy = "relevance";
      contentType = "mod";
      // Keep mcVersion and loader from instance context
    },

    /** Select content and load its versions */
    async selectContent(content: Content) {
      selectedContent = content;
      selectedContentVersions = [];
      selectedVersion = null;
      isLoadingVersions = true;

      try {
        // Pull full project details so the UI can show long descriptions and gallery
        try {
          const detailed = await contentService.getContent(content.platform, content.id);
          selectedContent = { ...content, ...detailed };
        } catch (e) {
          console.error("Failed to load detailed content info:", e);
          selectedContent = content;
        }

        // Only apply loader filter for mods - shaders/resourcepacks use different loader
        // identifiers (like "iris", "optifine") or none at all
        const shouldFilterByLoader = selectedContent.contentType === "mod";

        selectedContentVersions = await contentService.getContentVersions(
          selectedContent.platform,
          selectedContent.id,
          mcVersion || undefined,
          shouldFilterByLoader ? (loader || undefined) : undefined
        );
        // Auto-select the first version (already filtered & sorted by backend)
        if (selectedContentVersions.length > 0) {
          await this.setSelectedVersion(
            selectedContentVersions[0],
            selectedContent.platform
          );
        }
      } catch (e: unknown) {
        console.error("Failed to load content versions:", e);
      } finally {
        isLoadingVersions = false;
      }

      return selectedContent;
    },

    /** Clear selected content */
    clearSelection() {
      selectedContent = null;
      selectedContentVersions = [];
      selectedVersion = null;
      resolvedDependencies = [];
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
      query = "";
      platform = "modrinth";
      mcVersion = null;
      loader = null;
      contentType = "mod";
      category = null;
      sortBy = "relevance";
      instanceId = null;
      selectedContent = null;
      selectedContentVersions = [];
      selectedVersion = null;
      isLoadingVersions = false;
      scanResult = null;
      isScanning = false;
      isInstalling = false;
      installError = null;
      downloadProgress = null;
      downloadQueue = [];
      resolvedDependencies = [];
      isResolvingDeps = false;
    },
  };
}

/** Global content store instance */
export const contentStore = createContentStore();
