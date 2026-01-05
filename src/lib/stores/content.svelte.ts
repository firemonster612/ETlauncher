import type {
  Content,
  ContentDownloadProgress,
  ContentSearchParams,
  ContentVersion,
  ContentPlatform,
  ContentSortBy,
  ContentType,
  LoaderType,
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
  let pageSize = $state(20);

  // Filter state (can be set from instance context)
  let query = $state("");
  let platform = $state<ContentPlatform | null>(null);
  let mcVersion = $state<string | null>(null);
  let loader = $state<LoaderType | null>(null);
  let contentType = $state<ContentType>("mod");
  let category = $state<string | null>(null);
  let sortBy = $state<ContentSortBy>("downloads");

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

  // Download progress state
  let downloadProgress = $state<ContentDownloadProgress | null>(null);

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

    // Download progress getter
    get downloadProgress() {
      return downloadProgress;
    },

    // Resolved dependencies getters
    get resolvedDependencies() {
      return resolvedDependencies;
    },
    get isResolvingDeps() {
      return isResolvingDeps;
    },

    /** Set download progress (called from event listener) */
    setDownloadProgress(progress: ContentDownloadProgress | null) {
      downloadProgress = progress;
    },

    /** Set the instance context (auto-filters by MC version and loader) */
    async setInstanceContext(id: string, mcVer: string, loaderType: LoaderType) {
      instanceId = id;
      mcVersion = mcVer;
      loader = loaderType === "vanilla" ? null : loaderType;
      // Scan installed content for this instance
      await this.scanInstalledContent();
    },

    /** Scan installed content for the current content type */
    async scanInstalledContent() {
      if (!instanceId) return;
      isScanning = true;
      try {
        scanResult = await contentService.scanInstalledContent(instanceId, contentType);
      } catch (e) {
        console.error("Failed to scan installed content:", e);
      } finally {
        isScanning = false;
      }
    },

    /** Check if content is installed in the current instance */
    isContentInstalled(content: Content): boolean {
      if (!scanResult) return false;

      // Check if any scanned item matches this content's Modrinth ID
      if (content.platform === "modrinth") {
        return scanResult.items.some(
          (item) => item.modrinthProject?.projectId === content.id
        );
      }

      // Check if any scanned item matches this content's CurseForge ID
      if (content.platform === "curseforge") {
        const contentIdNum = parseInt(content.id, 10);
        return scanResult.items.some(
          (item) => item.curseforgeProject?.projectId === contentIdNum
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

    /** Refresh installed content by re-scanning */
    async refreshInstalledContent() {
      await this.scanInstalledContent();
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
        const params: ContentSearchParams = {
          query: query || undefined,
          platform: platform || undefined,
          mcVersion: mcVersion || undefined,
          loader: loader || undefined,
          contentType,
          category: category || undefined,
          sortBy,
          page: currentPage,
          pageSize,
        };

        const result = await contentService.searchContent(params);
        items = result.items;
        totalCount = result.totalCount;
      } catch (e: any) {
        searchError = e?.message || (typeof e === "string" ? e : JSON.stringify(e));
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
        const params: ContentSearchParams = {
          query: query || undefined,
          platform: platform || undefined,
          mcVersion: mcVersion || undefined,
          loader: loader || undefined,
          contentType,
          category: category || undefined,
          sortBy,
          page: currentPage,
          pageSize,
        };

        const result = await contentService.searchContent(params);
        items = [...items, ...result.items];
        totalCount = result.totalCount;
      } catch (e: any) {
        searchError = e?.message || (typeof e === "string" ? e : JSON.stringify(e));
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
    setPlatform(newPlatform: ContentPlatform | null) {
      platform = newPlatform;
    },

    /** Set content type filter and trigger scan */
    async setContentType(type: ContentType) {
      contentType = type;
      // Clear scan result when switching content types
      scanResult = null;
      // Trigger a new scan for this content type
      await this.scanInstalledContent();
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
      platform = null;
      category = null;
      sortBy = "downloads";
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
        // Only apply loader filter for mods - shaders/resourcepacks use different loader
        // identifiers (like "iris", "optifine") or none at all
        const shouldFilterByLoader = content.contentType === "mod";

        selectedContentVersions = await contentService.getContentVersions(
          content.platform,
          content.id,
          mcVersion || undefined,
          shouldFilterByLoader ? (loader || undefined) : undefined
        );
        // Auto-select the first version (already filtered & sorted by backend)
        if (selectedContentVersions.length > 0) {
          selectedVersion = selectedContentVersions[0];
        }
      } catch (e: any) {
        console.error("Failed to load content versions:", e);
      } finally {
        isLoadingVersions = false;
      }
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
      platform = null;
      mcVersion = null;
      loader = null;
      contentType = "mod";
      category = null;
      sortBy = "downloads";
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
      resolvedDependencies = [];
      isResolvingDeps = false;
    },
  };
}

/** Global content store instance */
export const contentStore = createContentStore();
