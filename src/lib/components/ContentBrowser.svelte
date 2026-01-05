<script lang="ts">
  import { onMount } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import { listen } from "@tauri-apps/api/event";
  import {
    Package,
    Search,
    Download,
    ExternalLink,
    Loader2,
    X,
    ChevronDown,
    ChevronLeft,
    Check,
    AlertTriangle,
    CheckCircle,
    Trash2,
    Power,
    PowerOff,
    SquareCheck,
    Square,
    Minus,
  } from "@lucide/svelte";
  import { Button } from "$lib/ui/button";
  import * as Select from "$lib/ui/select";
  import { contentStore } from "$lib/stores/content.svelte";
  import * as contentService from "$lib/services/content";
  import type {
    Content,
    ContentDownloadProgress,
    ContentType,
    ContentSortBy,
    ContentPlatform,
    LoaderType,
    ContentVersion,
    DetectedMod,
  } from "$lib/types";

  interface Props {
    instanceId: string;
    instanceName: string;
    mcVersion: string;
    loaderType: LoaderType;
    onClose: () => void;
  }

  let { instanceId, instanceName, mcVersion, loaderType, onClose }: Props = $props();

  let searchInput = $state("");
  let selectedContentDetail = $state<Content | null>(null);

  // Navigation history for dependency drilling
  let contentHistory = $state<Content[]>([]);

  // View mode: "browse" for searching online, "installed" for local files
  let viewMode = $state<"browse" | "installed">("browse");

  // Multi-select state for installed tab
  let selectedItems = new SvelteSet<string>();

  // Derived: installed items list from scan result
  const installedItems = $derived(contentStore.scanResult?.items ?? []);

  // Derived: check if all items are selected
  const allSelected = $derived(
    installedItems.length > 0 && selectedItems.size === installedItems.length
  );

  // Derived: check if some items are selected
  const someSelected = $derived(
    selectedItems.size > 0 && selectedItems.size < installedItems.length
  );

  // Derived: selected enabled items (for disable action)
  const selectedEnabledItems = $derived(
    installedItems.filter((item) => selectedItems.has(item.filename) && !item.isDisabled)
  );

  // Derived: selected disabled items (for enable action)
  const selectedDisabledItems = $derived(
    installedItems.filter((item) => selectedItems.has(item.filename) && item.isDisabled)
  );

  // Check if current content type is blocked for vanilla instances
  // Mods and shaders require a mod loader; resource packs work on vanilla
  const isBlockedForVanilla = $derived(
    loaderType === "vanilla" &&
    (contentStore.contentType === "mod" || contentStore.contentType === "shader")
  );

  // Installation state
  let isInstalling = $state(false);
  let installSuccess = $state<string | null>(null);
  let installError = $state<string | null>(null);

  // Uninstall state
  let isUninstalling = $state(false);
  let uninstallError = $state<string | null>(null);

  // Bulk action state
  let isBulkActioning = $state(false);
  let bulkActionError = $state<string | null>(null);
  let showRemoveConfirm = $state(false);

  // Initialize store with instance context on mount (once only)
  onMount(() => {
    contentStore.setInstanceContext(instanceId, mcVersion, loaderType);
    contentStore.search();

    // Listen for download progress events
    const unlistenPromise = listen<ContentDownloadProgress>(
      "content_download_progress",
      (event) => {
        contentStore.setDownloadProgress(event.payload);
      }
    );

    return () => {
      // Clean up event listener
      unlistenPromise.then((unlisten) => unlisten());
      contentStore.reset();
    };
  });

  // Debounced search
  let searchTimeout: ReturnType<typeof setTimeout>;
  function handleSearchInput(value: string) {
    searchInput = value;
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => {
      contentStore.setQuery(value);
      contentStore.search();
    }, 300);
  }

  async function handleContentTypeChange(type: ContentType) {
    await contentStore.setContentType(type);
    // Clear selection when changing content type
    selectedItems = new SvelteSet();
    if (viewMode === "browse") {
      contentStore.search();
    }
  }

  function handleViewModeChange(mode: "browse" | "installed") {
    viewMode = mode;
    selectedItems = new SvelteSet();
    bulkActionError = null;
  }

  function toggleItemSelection(filename: string) {
    if (selectedItems.has(filename)) {
      selectedItems.delete(filename);
    } else {
      selectedItems.add(filename);
    }
  }

  function toggleSelectAll() {
    if (allSelected) {
      selectedItems = new SvelteSet();
    } else {
      selectedItems = new SvelteSet(installedItems.map((item) => item.filename));
    }
  }

  async function handleDisableSelected() {
    if (selectedEnabledItems.length === 0) return;

    isBulkActioning = true;
    bulkActionError = null;

    try {
      const filenames = selectedEnabledItems.map((item) => item.filename);
      await contentService.disableContent(instanceId, filenames, contentStore.contentType);
      // Refresh scan results
      await contentStore.refreshInstalledContent();
      selectedItems = new SvelteSet();
    } catch (e: unknown) {
      bulkActionError = e instanceof Error ? e.message : "Failed to disable content";
    } finally {
      isBulkActioning = false;
    }
  }

  async function handleEnableSelected() {
    if (selectedDisabledItems.length === 0) return;

    isBulkActioning = true;
    bulkActionError = null;

    try {
      const filenames = selectedDisabledItems.map((item) => item.filename);
      await contentService.enableContent(instanceId, filenames, contentStore.contentType);
      // Refresh scan results
      await contentStore.refreshInstalledContent();
      selectedItems = new SvelteSet();
    } catch (e: unknown) {
      bulkActionError = e instanceof Error ? e.message : "Failed to enable content";
    } finally {
      isBulkActioning = false;
    }
  }

  async function handleRemoveSelected() {
    if (selectedItems.size === 0) return;

    isBulkActioning = true;
    bulkActionError = null;

    try {
      // Remove each selected item
      for (const filename of selectedItems) {
        await contentService.uninstallContentByFilename(
          instanceId,
          filename,
          contentStore.contentType
        );
      }
      // Refresh scan results
      await contentStore.refreshInstalledContent();
      selectedItems = new SvelteSet();
      showRemoveConfirm = false;
    } catch (e: unknown) {
      bulkActionError = e instanceof Error ? e.message : "Failed to remove content";
    } finally {
      isBulkActioning = false;
    }
  }

  function getItemDisplayName(item: DetectedMod): string {
    // Try to get a nice name from Modrinth or CurseForge data
    if (item.modrinthProject?.name) {
      return item.modrinthProject.name;
    }
    if (item.curseforgeProject?.name) {
      return item.curseforgeProject.name;
    }
    // Fall back to filename without extension
    return item.filename.replace(/\.(jar|zip)$/i, "");
  }

  function getItemVersion(item: DetectedMod): string | null {
    if (item.modrinthProject?.versionNumber) {
      return item.modrinthProject.versionNumber;
    }
    return null;
  }

  function handlePlatformChange(platform: ContentPlatform | "all") {
    contentStore.setPlatform(platform === "all" ? null : platform);
    contentStore.search();
  }

  function handleSortChange(sort: ContentSortBy) {
    contentStore.setSortBy(sort);
    contentStore.search();
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

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function getPlatformColor(platform: ContentPlatform): string {
    switch (platform) {
      case "modrinth":
        return "bg-green-500/20 text-green-500 border-green-500/50";
      case "curseforge":
        return "bg-orange-500/20 text-orange-500 border-orange-500/50";
      default:
        return "bg-muted text-muted-foreground border-muted";
    }
  }

  function getLoaderColor(loader: LoaderType): string {
    switch (loader) {
      case "fabric":
        return "bg-amber-500/20 text-amber-500";
      case "forge":
        return "bg-orange-500/20 text-orange-500";
      case "neoforge":
        return "bg-red-500/20 text-red-500";
      case "quilt":
        return "bg-purple-500/20 text-purple-500";
      default:
        return "bg-muted/50 text-muted-foreground";
    }
  }

  async function handleContentClick(content: Content) {
    selectedContentDetail = content;
    await contentStore.selectContent(content);
    // Auto-resolve deps for first version
    if (contentStore.selectedVersion) {
      await contentStore.resolveDependencies(
        contentStore.selectedVersion,
        content.platform
      );
    }
  }

  function closeContentDetail() {
    selectedContentDetail = null;
    contentHistory = [];
    contentStore.clearSelection();
  }

  function handleVersionSelect(version: ContentVersion) {
    if (selectedContentDetail) {
      contentStore.setSelectedVersion(version, selectedContentDetail.platform);
    }
  }

  async function handleDependencyClick(content: Content) {
    // Push current content to history
    if (selectedContentDetail) {
      contentHistory = [...contentHistory, selectedContentDetail];
    }
    // Navigate to the dependency
    selectedContentDetail = content;
    await contentStore.selectContent(content);
    // Resolve deps for the new content's first version
    if (contentStore.selectedVersion) {
      await contentStore.resolveDependencies(
        contentStore.selectedVersion,
        content.platform
      );
    }
  }

  async function handleBackClick() {
    if (contentHistory.length === 0) return;

    const previous = contentHistory[contentHistory.length - 1];
    contentHistory = contentHistory.slice(0, -1);

    selectedContentDetail = previous;
    await contentStore.selectContent(previous);
    if (contentStore.selectedVersion) {
      await contentStore.resolveDependencies(
        contentStore.selectedVersion,
        previous.platform
      );
    }
  }

  async function handleInstall() {
    if (!selectedContentDetail || !contentStore.selectedVersion) return;

    isInstalling = true;
    installSuccess = null;
    installError = null;
    contentStore.setDownloadProgress(null);

    try {
      console.log("[ContentBrowser] Installing:", selectedContentDetail.name, contentStore.selectedVersion.versionNumber);

      // Install with dependencies
      const installed = await contentService.installContentWithDependencies(
        instanceId,
        selectedContentDetail.platform,
        selectedContentDetail,
        contentStore.selectedVersion,
        mcVersion,
        loaderType === "vanilla" ? undefined : loaderType
      );

      console.log("[ContentBrowser] Installation complete:", installed.length, "items installed");

      // Refresh installed content manifest so badges update
      await contentStore.refreshInstalledContent();

      installSuccess = installed.length > 1
        ? `Installed ${selectedContentDetail.name} and ${installed.length - 1} dependencies`
        : `Installed ${selectedContentDetail.name}`;

      // Close the detail modal after a short delay
      setTimeout(() => {
        closeContentDetail();
        installSuccess = null;
      }, 1500);
    } catch (e: unknown) {
      console.error("[ContentBrowser] Installation failed:", e);
      installError = e instanceof Error ? e.message : (typeof e === "string" ? e : JSON.stringify(e));
    } finally {
      isInstalling = false;
      contentStore.setDownloadProgress(null);
    }
  }

  async function handleUninstall() {
    if (!selectedContentDetail) return;

    const contentId = selectedContentDetail.id;
    const platform = selectedContentDetail.platform;

    // Find the matching installed item from scan result to get the filename
    let installedItem = contentStore.scanResult?.items.find((item) => {
      if (platform === "modrinth") {
        return item.modrinthProject?.projectId === contentId;
      } else if (platform === "curseforge") {
        const contentIdNum = parseInt(contentId, 10);
        return item.curseforgeProject?.projectId === contentIdNum;
      }
      return false;
    });

    if (!installedItem) {
      uninstallError = "Could not find the installed file for this content";
      return;
    }

    isUninstalling = true;
    uninstallError = null;

    try {
      console.log("[ContentBrowser] Uninstalling:", selectedContentDetail.name, "file:", installedItem.filename);

      await contentService.uninstallContentByFilename(
        instanceId,
        installedItem.filename,
        selectedContentDetail.contentType
      );

      console.log("[ContentBrowser] Uninstall complete");

      // Refresh installed content by re-scanning
      await contentStore.refreshInstalledContent();

      // Close the detail modal
      closeContentDetail();
    } catch (e: unknown) {
      console.error("[ContentBrowser] Uninstall failed:", e);
      uninstallError = e instanceof Error ? e.message : (typeof e === "string" ? e : JSON.stringify(e));
    } finally {
      isUninstalling = false;
    }
  }

  const contentTypes: { value: ContentType; label: string }[] = [
    { value: "mod", label: "Mods" },
    { value: "shader", label: "Shaders" },
    { value: "resourcepack", label: "Resource Packs" },
  ];

  const platforms: { value: ContentPlatform | "all"; label: string }[] = [
    { value: "all", label: "All" },
    { value: "modrinth", label: "Modrinth" },
    { value: "curseforge", label: "CurseForge" },
  ];

  const sortOptions: { value: ContentSortBy; label: string }[] = [
    { value: "downloads", label: "Downloads" },
    { value: "recentlyUpdated", label: "Recently Updated" },
    { value: "relevance", label: "Relevance" },
  ];
</script>

<!-- Backdrop -->
<div class="fixed inset-0 bg-black/50 z-50" onclick={onClose}></div>

<!-- Panel -->
<div class="fixed inset-y-0 right-0 w-full max-w-2xl bg-card border-l-2 border-border z-50 flex flex-col">
  <!-- Header -->
  <div class="p-4 border-b border-border flex items-center justify-between">
    <div>
      <h2 class="text-lg font-bold">{viewMode === "browse" ? "Add Content" : "Manage Content"}</h2>
      <p class="text-sm text-muted-foreground">
        {instanceName} &bull; MC {mcVersion}
        {#if loaderType !== "vanilla"}
          &bull; {loaderType}
        {/if}
      </p>
    </div>
    <button
      class="text-muted-foreground hover:text-foreground p-2"
      onclick={onClose}
    >
      <X class="h-5 w-5" />
    </button>
  </div>

  <!-- Content Type Tabs -->
  <div class="p-4 border-b border-border flex gap-2">
    {#each contentTypes as { value, label } (value)}
      <Button
        variant={contentStore.contentType === value ? "default" : "secondary"}
        size="sm"
        onclick={() => handleContentTypeChange(value)}
      >
        {label}
      </Button>
    {/each}
  </div>

  <!-- Browse / Installed Tabs -->
  <div class="px-4 pt-2 flex gap-1 border-b border-border">
    <button
      class="px-4 py-2 text-sm font-medium transition-colors relative {viewMode === 'browse'
        ? 'text-primary'
        : 'text-muted-foreground hover:text-foreground'}"
      onclick={() => handleViewModeChange("browse")}
    >
      Browse
      {#if viewMode === "browse"}
        <div class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary"></div>
      {/if}
    </button>
    <button
      class="px-4 py-2 text-sm font-medium transition-colors relative flex items-center gap-2 {viewMode === 'installed'
        ? 'text-primary'
        : 'text-muted-foreground hover:text-foreground'}"
      onclick={() => handleViewModeChange("installed")}
    >
      Installed
      {#if installedItems.length > 0}
        <span class="text-xs bg-muted px-1.5 py-0.5 rounded-full">
          {installedItems.length}
        </span>
      {/if}
      {#if viewMode === "installed"}
        <div class="absolute bottom-0 left-0 right-0 h-0.5 bg-primary"></div>
      {/if}
    </button>
  </div>

  <!-- Vanilla Instance Warning (only show in browse mode) -->
  {#if isBlockedForVanilla && viewMode === "browse"}
    <div class="mx-4 mt-4 bg-amber-500/10 border-2 border-amber-500/50 p-3 text-amber-500 text-sm flex items-center gap-2">
      <AlertTriangle class="h-4 w-4 flex-shrink-0" />
      <span>
        {contentStore.contentType === "mod" ? "Mods" : "Shaders"} require a mod loader (Fabric, Forge, etc.).
        This is a vanilla instance. Switch to Resource Packs or add a mod loader to this instance.
      </span>
    </div>
  {/if}

  {#if viewMode === "browse"}
  <!-- Search and Filters -->
  <div class="p-4 border-b border-border space-y-3">
    <div class="flex gap-2">
      <div class="relative flex-1">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
        <input
          type="text"
          placeholder="Search {contentStore.contentType}s..."
          value={searchInput}
          oninput={(e) => handleSearchInput(e.currentTarget.value)}
          class="w-full h-9 pl-9 pr-3 bg-background border-2 border-border text-sm focus:border-primary outline-none"
        />
      </div>
    </div>
    <div class="flex gap-2 flex-wrap">
      {#each platforms as { value, label } (value)}
        <Button
          variant={contentStore.platform === (value === "all" ? null : value) ? "default" : "outline"}
          size="sm"
          onclick={() => handlePlatformChange(value)}
        >
          {label}
        </Button>
      {/each}
      <div class="ml-auto">
        <Select.Root
          type="single"
          value={contentStore.sortBy}
          onValueChange={(v) => handleSortChange(v as ContentSortBy)}
        >
          <Select.Trigger class="h-8 text-xs border-2 border-border bg-background">
            Sort: {sortOptions.find((o) => o.value === contentStore.sortBy)?.label}
          </Select.Trigger>
          <Select.Content class="border-2 border-border bg-card">
            {#each sortOptions as { value, label } (value)}
              <Select.Item {value} {label}>{label}</Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </div>
    </div>
  </div>

  <!-- Error Display -->
  {#if contentStore.searchError}
    <div class="mx-4 mt-4 bg-destructive/10 border-2 border-destructive p-3 text-destructive text-sm">
      {contentStore.searchError}
      <button class="underline ml-2" onclick={() => contentStore.clearSearchError()}>
        Dismiss
      </button>
    </div>
  {/if}

  <!-- Scanning Indicator -->
  {#if contentStore.isScanning}
    <div class="mx-4 mt-4 bg-primary/10 border-2 border-primary/30 p-3 text-primary text-sm flex items-center gap-2">
      <Loader2 class="h-4 w-4 animate-spin" />
      Scanning installed mods...
    </div>
  {/if}

  <!-- Results -->
  <div class="flex-1 overflow-y-auto p-4">
    {#if contentStore.isSearching && contentStore.items.length === 0}
      <div class="flex items-center justify-center py-12">
        <Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    {:else if contentStore.items.length === 0}
      <div class="border-2 border-dashed border-border bg-card/50 p-12 text-center">
        <Package class="mx-auto h-12 w-12 text-muted-foreground/50" />
        <p class="mt-4 text-sm text-muted-foreground">
          {searchInput ? "No results found" : `Search for ${contentStore.contentType}s`}
        </p>
      </div>
    {:else}
      <div class="text-xs text-muted-foreground mb-3">
        {contentStore.totalCount} results
      </div>

      <div class="space-y-2">
        {#each contentStore.items as content (content.id)}
          <button
            class="w-full border-2 border-border bg-background p-3 hover:border-primary/50 transition-colors text-left flex gap-3"
            onclick={() => handleContentClick(content)}
          >
            {#if content.iconUrl}
              <img
                src={content.iconUrl}
                alt={content.name}
                class="w-12 h-12 object-cover rounded flex-shrink-0"
              />
            {:else}
              <div class="w-12 h-12 bg-muted flex items-center justify-center rounded flex-shrink-0">
                <Package class="h-6 w-6 text-muted-foreground/50" />
              </div>
            {/if}
            <div class="flex-1 min-w-0">
              <div class="flex items-start justify-between gap-2">
                <div class="flex items-center gap-2 min-w-0">
                  <h3 class="font-medium truncate">{content.name}</h3>
                  {#if contentStore.isContentInstalled(content)}
                    <span class="text-xs bg-green-500/20 text-green-500 px-1.5 py-0.5 rounded flex items-center gap-1 flex-shrink-0">
                      <CheckCircle class="h-3 w-3" />
                      Installed
                    </span>
                  {/if}
                </div>
                <span
                  class="text-xs px-1.5 py-0.5 border rounded flex-shrink-0 {getPlatformColor(content.platform)}"
                >
                  {content.platform}
                </span>
              </div>
              <p class="text-xs text-muted-foreground truncate">{content.author}</p>
              <p class="text-xs text-muted-foreground mt-1 line-clamp-1">
                {content.description}
              </p>
              <div class="flex items-center gap-3 mt-1 text-xs text-muted-foreground">
                <span class="flex items-center gap-1">
                  <Download class="h-3 w-3" />
                  {formatDownloads(content.downloads)}
                </span>
                {#each content.loaders.filter(l => l !== "unknown").slice(0, 2) as loader (loader)}
                  <span class="px-1 rounded {getLoaderColor(loader)}">{loader}</span>
                {/each}
              </div>
            </div>
          </button>
        {/each}
      </div>

      <!-- Load More -->
      {#if contentStore.hasMore}
        <div class="flex justify-center mt-4">
          <Button
            variant="outline"
            size="sm"
            onclick={() => contentStore.loadMore()}
            disabled={contentStore.isSearching}
          >
            {#if contentStore.isSearching}
              <Loader2 class="h-4 w-4 animate-spin mr-2" />
            {:else}
              <ChevronDown class="h-4 w-4 mr-2" />
            {/if}
            Load More
          </Button>
        </div>
      {/if}
    {/if}
  </div>
  {:else}
  <!-- Installed Tab Content -->

  <!-- Bulk Action Error -->
  {#if bulkActionError}
    <div class="mx-4 mt-4 bg-destructive/10 border-2 border-destructive p-3 text-destructive text-sm">
      {bulkActionError}
      <button class="underline ml-2" onclick={() => bulkActionError = null}>
        Dismiss
      </button>
    </div>
  {/if}

  <!-- Scanning Indicator -->
  {#if contentStore.isScanning}
    <div class="mx-4 mt-4 bg-primary/10 border-2 border-primary/30 p-3 text-primary text-sm flex items-center gap-2">
      <Loader2 class="h-4 w-4 animate-spin" />
      Scanning installed {contentStore.contentType}s...
    </div>
  {/if}

  <!-- Installed Items List -->
  <div class="flex-1 overflow-y-auto p-4">
    {#if installedItems.length === 0 && !contentStore.isScanning}
      <div class="border-2 border-dashed border-border bg-card/50 p-12 text-center">
        <Package class="mx-auto h-12 w-12 text-muted-foreground/50" />
        <p class="mt-4 text-sm text-muted-foreground">
          No {contentStore.contentType}s installed
        </p>
        <Button
          variant="outline"
          size="sm"
          class="mt-4"
          onclick={() => handleViewModeChange("browse")}
        >
          Browse {contentStore.contentType}s
        </Button>
      </div>
    {:else}
      <!-- Select All Header -->
      <div class="flex items-center justify-between mb-3 pb-2 border-b border-border">
        <button
          class="flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground"
          onclick={toggleSelectAll}
        >
          {#if allSelected}
            <SquareCheck class="h-4 w-4 text-primary" />
          {:else if someSelected}
            <div class="relative">
              <Square class="h-4 w-4" />
              <Minus class="h-2 w-2 absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2" />
            </div>
          {:else}
            <Square class="h-4 w-4" />
          {/if}
          Select All
        </button>
        <div class="text-xs text-muted-foreground">
          {installedItems.length} items
          {#if installedItems.filter(i => i.isDisabled).length > 0}
            &bull; {installedItems.filter(i => i.isDisabled).length} disabled
          {/if}
        </div>
      </div>

      <!-- Items List -->
      <div class="space-y-2">
        {#each installedItems as item (item.filename)}
          {@const isSelected = selectedItems.has(item.filename)}
          <button
            class="w-full border-2 p-3 text-left flex gap-3 transition-colors {isSelected
              ? 'border-primary bg-primary/5'
              : 'border-border bg-background hover:border-primary/50'} {item.isDisabled ? 'opacity-60' : ''}"
            onclick={() => toggleItemSelection(item.filename)}
          >
            <!-- Checkbox -->
            <div class="flex-shrink-0 pt-0.5">
              {#if isSelected}
                <SquareCheck class="h-5 w-5 text-primary" />
              {:else}
                <Square class="h-5 w-5 text-muted-foreground" />
              {/if}
            </div>

            <!-- Content -->
            <div class="flex-1 min-w-0">
              <div class="flex items-start justify-between gap-2">
                <div class="flex items-center gap-2 min-w-0">
                  <h3 class="font-medium truncate">{getItemDisplayName(item)}</h3>
                  {#if item.isDisabled}
                    <span class="text-xs bg-amber-500/20 text-amber-500 px-1.5 py-0.5 rounded flex items-center gap-1 flex-shrink-0">
                      <PowerOff class="h-3 w-3" />
                      Disabled
                    </span>
                  {/if}
                </div>
                <div class="flex items-center gap-1 flex-shrink-0">
                  {#if item.modrinthProject}
                    <span class="text-xs px-1.5 py-0.5 border rounded bg-green-500/20 text-green-500 border-green-500/50">
                      modrinth
                    </span>
                  {:else if item.curseforgeProject}
                    <span class="text-xs px-1.5 py-0.5 border rounded bg-orange-500/20 text-orange-500 border-orange-500/50">
                      curseforge
                    </span>
                  {:else}
                    <span class="text-xs px-1.5 py-0.5 border rounded bg-muted text-muted-foreground border-muted">
                      unknown
                    </span>
                  {/if}
                </div>
              </div>
              <p class="text-xs text-muted-foreground truncate mt-0.5">{item.filename}</p>
              <div class="flex items-center gap-3 mt-1 text-xs text-muted-foreground">
                {#if getItemVersion(item)}
                  <span>v{getItemVersion(item)}</span>
                {/if}
                <span>{formatBytes(item.size)}</span>
              </div>
            </div>
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Action Bar (shown when items are selected) -->
  {#if selectedItems.size > 0}
    <div class="p-4 border-t border-border bg-background">
      <div class="flex items-center justify-between mb-2">
        <span class="text-sm text-muted-foreground">
          {selectedItems.size} item{selectedItems.size === 1 ? '' : 's'} selected
        </span>
        <button
          class="text-xs text-muted-foreground hover:text-foreground"
          onclick={() => selectedItems = new SvelteSet()}
        >
          Clear selection
        </button>
      </div>
      <div class="flex gap-2">
        {#if selectedEnabledItems.length > 0}
          <Button
            variant="secondary"
            size="sm"
            class="flex-1"
            disabled={isBulkActioning}
            onclick={handleDisableSelected}
          >
            {#if isBulkActioning}
              <Loader2 class="h-4 w-4 animate-spin mr-2" />
            {:else}
              <PowerOff class="h-4 w-4 mr-2" />
            {/if}
            Disable ({selectedEnabledItems.length})
          </Button>
        {/if}
        {#if selectedDisabledItems.length > 0}
          <Button
            variant="secondary"
            size="sm"
            class="flex-1"
            disabled={isBulkActioning}
            onclick={handleEnableSelected}
          >
            {#if isBulkActioning}
              <Loader2 class="h-4 w-4 animate-spin mr-2" />
            {:else}
              <Power class="h-4 w-4 mr-2" />
            {/if}
            Enable ({selectedDisabledItems.length})
          </Button>
        {/if}
        <Button
          variant="destructive"
          size="sm"
          class="flex-1"
          disabled={isBulkActioning}
          onclick={() => showRemoveConfirm = true}
        >
          <Trash2 class="h-4 w-4 mr-2" />
          Remove ({selectedItems.size})
        </Button>
      </div>
    </div>
  {/if}
  {/if}
</div>

<!-- Remove Confirmation Modal -->
{#if showRemoveConfirm}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-[60] p-4">
    <div class="bg-card border-2 border-border max-w-sm w-full p-4">
      <h3 class="font-bold mb-2">Remove {selectedItems.size} item{selectedItems.size === 1 ? '' : 's'}?</h3>
      <p class="text-sm text-muted-foreground mb-4">
        This will permanently delete the selected files. This action cannot be undone.
      </p>
      <div class="flex gap-2">
        <Button
          variant="outline"
          class="flex-1"
          onclick={() => showRemoveConfirm = false}
          disabled={isBulkActioning}
        >
          Cancel
        </Button>
        <Button
          variant="destructive"
          class="flex-1"
          onclick={handleRemoveSelected}
          disabled={isBulkActioning}
        >
          {#if isBulkActioning}
            <Loader2 class="h-4 w-4 animate-spin mr-2" />
            Removing...
          {:else}
            Remove
          {/if}
        </Button>
      </div>
    </div>
  </div>
{/if}

<!-- Content Detail Modal -->
{#if selectedContentDetail}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-[60] p-4">
    <div class="bg-card border-2 border-border max-w-lg w-full max-h-[80vh] overflow-hidden flex flex-col">
      <!-- Header -->
      <div class="p-4 border-b border-border">
        <div class="flex gap-3">
          <!-- Back button when navigating dependencies -->
          {#if contentHistory.length > 0}
            <button
              class="text-muted-foreground hover:text-foreground flex-shrink-0 self-center -ml-1 mr-1"
              onclick={handleBackClick}
              title="Back to {contentHistory[contentHistory.length - 1]?.name}"
            >
              <ChevronLeft class="h-6 w-6" />
            </button>
          {/if}
          {#if selectedContentDetail.iconUrl}
            <img
              src={selectedContentDetail.iconUrl}
              alt={selectedContentDetail.name}
              class="w-16 h-16 object-cover rounded"
            />
          {:else}
            <div class="w-16 h-16 bg-muted flex items-center justify-center rounded">
              <Package class="h-8 w-8 text-muted-foreground/50" />
            </div>
          {/if}
          <div class="flex-1 min-w-0">
            <div class="flex items-start justify-between gap-2">
              <h2 class="font-bold truncate">{selectedContentDetail.name}</h2>
              <button
                class="text-muted-foreground hover:text-foreground flex-shrink-0"
                onclick={closeContentDetail}
              >
                <X class="h-5 w-5" />
              </button>
            </div>
            <p class="text-sm text-muted-foreground">{selectedContentDetail.author}</p>
            <div class="flex items-center gap-2 mt-1 flex-wrap">
              <span
                class="text-xs px-1.5 py-0.5 border rounded {getPlatformColor(selectedContentDetail.platform)}"
              >
                {selectedContentDetail.platform}
              </span>
              <span class="text-xs text-muted-foreground flex items-center gap-1">
                <Download class="h-3 w-3" />
                {formatDownloads(selectedContentDetail.downloads)}
              </span>
              {#if contentStore.isContentInstalled(selectedContentDetail)}
                <span class="text-xs bg-green-500/20 text-green-500 px-1.5 py-0.5 rounded flex items-center gap-1">
                  <CheckCircle class="h-3 w-3" />
                  Already Installed
                </span>
              {/if}
            </div>
          </div>
        </div>
        <p class="text-sm mt-3">{selectedContentDetail.description}</p>
        {#if selectedContentDetail.url}
          <a
            href={selectedContentDetail.url}
            target="_blank"
            rel="noopener noreferrer"
            class="inline-flex items-center gap-1.5 text-xs bg-muted hover:bg-muted/80 px-2.5 py-1.5 rounded mt-3 transition-colors"
          >
            <ExternalLink class="h-3.5 w-3.5" />
            View on {selectedContentDetail.platform}
          </a>
        {/if}
      </div>

      <!-- Version Selection -->
      <div class="flex-1 overflow-y-auto p-4">
        <h3 class="font-semibold mb-2 text-sm">Select Version</h3>
        {#if contentStore.isLoadingVersions}
          <div class="flex items-center gap-2 text-muted-foreground text-sm">
            <Loader2 class="h-4 w-4 animate-spin" />
            Loading versions...
          </div>
        {:else if contentStore.selectedContentVersions.length === 0}
          <p class="text-sm text-muted-foreground">No compatible versions found</p>
        {:else}
          <div class="space-y-1">
            {#each contentStore.selectedContentVersions.slice(0, 15) as version (version.id)}
              {@const isSelected = contentStore.selectedVersion?.id === version.id}
              {@const hasRequiredDeps = version.dependencies.some(d => d.dependencyType === "required")}
              <button
                class="w-full p-2 text-left rounded border-2 transition-colors {isSelected
                  ? 'border-primary bg-primary/10'
                  : 'border-border hover:border-primary/50'}"
                onclick={() => handleVersionSelect(version)}
              >
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-2">
                    {#if isSelected}
                      <Check class="h-4 w-4 text-primary" />
                    {/if}
                    <span class="font-medium text-sm">{version.versionNumber}</span>
                  </div>
                  {#if hasRequiredDeps}
                    <span class="text-xs text-amber-500 flex items-center gap-1">
                      <AlertTriangle class="h-3 w-3" />
                      Dependencies
                    </span>
                  {/if}
                </div>
                <div class="text-xs text-muted-foreground mt-0.5">
                  {version.mcVersions.slice(0, 3).join(", ")}
                  {#if version.mcVersions.length > 3}+{version.mcVersions.length - 3} more{/if}
                  {#if version.releasedAt}
                    &bull; {new Date(version.releasedAt * 1000).toLocaleDateString()}
                  {/if}
                </div>
              </button>
            {/each}
          </div>
        {/if}

        <!-- Dependencies Section -->
        {#if selectedContentDetail && contentStore.selectedVersion}
          {@const hasRequiredDeps = contentStore.selectedVersion.dependencies.some(
            d => d.dependencyType === "required"
          )}

          {#if hasRequiredDeps}
            {#if contentStore.isResolvingDeps}
              <!-- Loading state -->
              <div class="mt-4 p-3 rounded border-2 bg-muted/50 border-border">
                <div class="flex items-center gap-2 text-sm text-muted-foreground">
                  <Loader2 class="h-4 w-4 animate-spin" />
                  Loading dependencies...
                </div>
              </div>
            {:else if contentStore.resolvedDependencies.length > 0}
              {@const allDepsInstalled = contentStore.resolvedDependencies.every(d => d.alreadyInstalled)}
              <div class="mt-4 p-3 rounded border-2 {allDepsInstalled
                ? 'bg-green-500/10 border-green-500/50'
                : 'bg-amber-500/10 border-amber-500/50'}">
                <div class="flex items-center gap-2 text-sm font-medium {allDepsInstalled
                  ? 'text-green-500'
                  : 'text-amber-500'}">
                  {#if allDepsInstalled}
                    <CheckCircle class="h-4 w-4" />
                    Dependencies Installed
                  {:else}
                    <AlertTriangle class="h-4 w-4" />
                    Required Dependencies
                  {/if}
                </div>
                <p class="text-xs text-muted-foreground mt-1">
                  {#if allDepsInstalled}
                    All required dependencies are already installed.
                  {:else}
                    This {selectedContentDetail.contentType} requires the following (will be auto-installed):
                  {/if}
                </p>
                <ul class="text-xs mt-2 space-y-1">
                  {#each contentStore.resolvedDependencies as resolved (resolved.content.id)}
                    <li class="flex items-center gap-2">
                      {#if resolved.alreadyInstalled}
                        <CheckCircle class="h-3 w-3 text-green-500" />
                      {:else}
                        <span class="w-1.5 h-1.5 bg-amber-500 rounded-full"></span>
                      {/if}
                      <button
                        type="button"
                        class="underline text-primary hover:text-primary/80 transition-colors text-left flex items-center gap-1 {resolved.alreadyInstalled ? 'text-green-500/80 hover:text-green-500' : ''}"
                        onclick={() => handleDependencyClick(resolved.content)}
                      >
                        {resolved.content.name}
                        <ExternalLink class="h-2.5 w-2.5" />
                      </button>
                      {#if resolved.alreadyInstalled}
                        <span class="text-green-500 text-[10px]">installed</span>
                      {/if}
                    </li>
                  {/each}
                </ul>
              </div>
            {:else}
              <!-- Fallback if resolution failed - show raw IDs -->
              {@const requiredDeps = contentStore.selectedVersion.dependencies.filter(
                d => d.dependencyType === "required"
              )}
              <div class="mt-4 p-3 rounded border-2 bg-amber-500/10 border-amber-500/50">
                <div class="flex items-center gap-2 text-sm font-medium text-amber-500">
                  <AlertTriangle class="h-4 w-4" />
                  Required Dependencies
                </div>
                <ul class="text-xs mt-2 space-y-1">
                  {#each requiredDeps as dep (dep.id)}
                    <li class="flex items-center gap-2">
                      <span class="w-1.5 h-1.5 bg-amber-500 rounded-full"></span>
                      <span class="text-muted-foreground italic">{dep.id}</span>
                    </li>
                  {/each}
                </ul>
              </div>
            {/if}
          {/if}
        {/if}
      </div>

      <!-- Footer -->
      <div class="p-4 border-t border-border space-y-3">
        <!-- Success Message -->
        {#if installSuccess}
          <div class="flex items-center gap-2 p-3 bg-green-500/10 border-2 border-green-500/50 rounded text-green-500 text-sm">
            <CheckCircle class="h-4 w-4 flex-shrink-0" />
            {installSuccess}
          </div>
        {/if}

        <!-- Install Error Message -->
        {#if installError}
          <div class="p-3 bg-destructive/10 border-2 border-destructive rounded text-destructive text-sm">
            <div class="flex items-center gap-2 font-medium">
              <AlertTriangle class="h-4 w-4 flex-shrink-0" />
              Installation Failed
            </div>
            <p class="text-xs mt-1 opacity-80">{installError}</p>
          </div>
        {/if}

        <!-- Uninstall Error Message -->
        {#if uninstallError}
          <div class="p-3 bg-destructive/10 border-2 border-destructive rounded text-destructive text-sm">
            <div class="flex items-center gap-2 font-medium">
              <AlertTriangle class="h-4 w-4 flex-shrink-0" />
              Uninstall Failed
            </div>
            <p class="text-xs mt-1 opacity-80">{uninstallError}</p>
          </div>
        {/if}

        <!-- Download Progress Bar -->
        {#if isInstalling && contentStore.downloadProgress}
          <div class="p-3 bg-primary/5 border-2 border-primary/30 rounded">
            <div class="flex items-center justify-between text-sm mb-2">
              <span class="text-muted-foreground truncate flex-1 mr-2">
                Downloading {contentStore.downloadProgress.filename}
              </span>
              <span class="text-primary font-medium flex-shrink-0">
                {contentStore.downloadProgress.progressPercent}%
              </span>
            </div>
            <div class="h-2 bg-muted rounded-full overflow-hidden">
              <div
                class="h-full bg-primary transition-all duration-150 ease-out"
                style="width: {contentStore.downloadProgress.progressPercent}%"
              ></div>
            </div>
            <div class="text-xs text-muted-foreground mt-1">
              {formatBytes(contentStore.downloadProgress.downloadedBytes)} / {formatBytes(contentStore.downloadProgress.totalBytes)}
            </div>
          </div>
        {/if}

        <div class="flex gap-2">
          <Button variant="outline" class="flex-1" onclick={closeContentDetail}>
            Cancel
          </Button>
          {#if contentStore.isContentInstalled(selectedContentDetail)}
            <Button
              variant="destructive"
              class="flex-1"
              disabled={isUninstalling || contentStore.isScanning}
              onclick={handleUninstall}
            >
              {#if isUninstalling}
                <Loader2 class="h-4 w-4 animate-spin mr-2" />
                Uninstalling...
              {:else}
                <Trash2 class="h-4 w-4 mr-2" />
                Uninstall
              {/if}
            </Button>
          {:else}
            <Button
              class="flex-1"
              disabled={!contentStore.selectedVersion || isInstalling || !!installSuccess || contentStore.isScanning || isBlockedForVanilla}
              onclick={handleInstall}
            >
              {#if isInstalling}
                <Loader2 class="h-4 w-4 animate-spin mr-2" />
                Installing...
              {:else if contentStore.isScanning}
                <Loader2 class="h-4 w-4 animate-spin mr-2" />
                Scanning...
              {:else if isBlockedForVanilla}
                <AlertTriangle class="h-4 w-4 mr-2" />
                Requires Mod Loader
              {:else}
                <Download class="h-4 w-4 mr-2" />
                Install
              {/if}
            </Button>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}
