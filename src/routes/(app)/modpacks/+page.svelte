<script lang="ts">
  import { onMount } from "svelte";
  import { onDestroy } from "svelte";
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
  } from "@lucide/svelte";
  import { marked } from "marked";
    import { openUrl } from "@tauri-apps/plugin-opener";
  import { ask } from "@tauri-apps/plugin-dialog";
  import { Button } from "$lib/ui/button";
  import { Checkbox } from "$lib/ui/checkbox";
  import { Input } from "$lib/ui/input";
  import * as Select from "$lib/ui/select";
  import { modpacksStore } from "$lib/stores/modpacks.svelte";
  import { modpackInstallStore } from "$lib/stores/modpackInstall.svelte";
  import { versionsStore } from "$lib/stores/versions.svelte";
  import DownloadProgress from "$lib/components/DownloadProgress.svelte";
  import ScreenshotLightbox from "$lib/components/ScreenshotLightbox.svelte";
  import DescriptionModal from "$lib/components/DescriptionModal.svelte";
  import * as modpackService from "$lib/services/modpack";
  import type { Modpack, ModpackMod, ModpackPlatform, ModpackSortBy, LoaderType } from "$lib/types";

  // Category options per platform
  const modrinthCategories = [
    "adventure", "challenging", "combat", "kitchen-sink", "lightweight",
    "magic", "multiplayer", "optimization", "quests", "technology"
  ];

  const curseforgeCategories = [
    "Adventure and RPG", "Combat / PvP", "Expert", "Exploration", "Extra Large",
    "FTB Official Pack", "Hardcore", "Horror", "Magic", "Map Based", "Mini Game",
    "Multiplayer", "Quests", "Sci-Fi", "Skyblock", "Small / Light", "Tech", "Vanilla+"
  ];

  let searchInput = $state("");
  let showFilters = $state(false);
  let selectedModpackDetail = $state<Modpack | null>(null);
  let modpackDetailTab = $state<"about" | "gallery" | "mods">("about");
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
    modpacksStore.platform === "modrinth" ? modrinthCategories :
    modpacksStore.platform === "curseforge" ? curseforgeCategories :
    []
  );

  // Platforms that support category filtering
  const platformsWithCategoryFilter = ["modrinth", "curseforge"];
  let showCategoryFilter = $derived(modpacksStore.platform && platformsWithCategoryFilter.includes(modpacksStore.platform));

  onMount(() => {
    versionsStore.load();
    // Initial search
    modpacksStore.search();
  });

  // Track if the user has manually scrolled (mouse wheel or touch)
  const markScrollListener = () => markUserScrolled();
  onMount(() => {
    window.addEventListener("wheel", markScrollListener, { passive: true });
    window.addEventListener("touchmove", markScrollListener, { passive: true });
  });

  onDestroy(() => {
    window.removeEventListener("wheel", markScrollListener);
    window.removeEventListener("touchmove", markScrollListener);
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
  const platformsWithVersionFilter = ["modrinth", "curseforge", "atlauncher"];
  // Platforms that support loader filtering
  const platformsWithLoaderFilter = ["modrinth", "curseforge"];

  // Computed filter visibility - requires a specific platform to be selected
  let showVersionFilter = $derived(modpacksStore.platform && platformsWithVersionFilter.includes(modpacksStore.platform));
  let showLoaderFilter = $derived(modpacksStore.platform && platformsWithLoaderFilter.includes(modpacksStore.platform));

  // Show filter button only for platforms that have at least one filter type
  let hasAnyFilters = $derived(showVersionFilter || showLoaderFilter || (showCategoryFilter && availableCategories.length > 0));

  function handlePlatformChange(platform: ModpackPlatform | "all") {
    modpacksStore.setPlatform(platform === "all" ? null : platform);
    // Clear filters that don't work on the selected platform
    if (platform !== "all") {
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
    modpacksStore.search();
  }

  function toggleCategory(category: string) {
    if (selectedCategories.includes(category)) {
      selectedCategories = selectedCategories.filter(c => c !== category);
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
    modpacksStore.setLoader(loader === "any" ? null : (loader as LoaderType));
    modpacksStore.search();
  }

  function clearFilters() {
    modpacksStore.clearFilters();
    searchInput = "";
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
      case "modrinth":
        return "bg-green-500/20 text-green-500 border-green-500/50";
      case "curseforge":
        return "bg-orange-500/20 text-orange-500 border-orange-500/50";
      case "ftb":
        return "bg-blue-500/20 text-blue-500 border-blue-500/50";
      case "technic":
        return "bg-yellow-500/20 text-yellow-500 border-yellow-500/50";
      case "atlauncher":
        return "bg-purple-500/20 text-purple-500 border-purple-500/50";
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

  function renderDescription(body?: string | null, fallback?: string): string {
    const source = (body && body.trim()) || fallback || "";
    if (!source) return "";

    const trimmed = source.trim();
    const looksLikeHtml = /^</.test(trimmed) && /<\/?[a-z][\s\S]*>/i.test(trimmed);

    try {
      return looksLikeHtml ? trimmed : (marked.parse(trimmed) as string);
    } catch (e) {
      console.error("Failed to render description", e);
      return trimmed.replace(/\n/g, "<br/>");
    }
  }

  async function handleDescriptionLinkClick(e: MouseEvent) {
    const target = e.target as HTMLElement | null;
    const anchor = target?.closest("a") as HTMLAnchorElement | null;
    if (!anchor) return;

    const href = anchor.getAttribute("href");
    if (!href || href.startsWith("#")) return;

    e.preventDefault();
    e.stopPropagation();
    try {
      await openUrl(href);
    } catch (err) {
      console.error("Failed to open URL:", href, err);
    }
  }

  async function handleModpackClick(modpack: Modpack) {
    modpackDetailTab = "about";
    modpackLightboxIndex = null;
    descriptionExpanded = false;
    selectedVersionId = null;
    isLoadingMods = false;
    modsError = null;
    modListCache = {};
    selectedModpackDetail = modpack;
    selectedModpackDetail = await modpacksStore.selectModpack(modpack);
    selectedVersionId = modpacksStore.selectedModpackVersions[0]?.id ?? null;
  }

  function closeModpackDetail() {
    selectedModpackDetail = null;
    modpackDetailTab = "about";
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
    return platform === "modrinth" || platform === "curseforge" || platform === "ftb" || platform === "technic";
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
    } catch (e: any) {
      console.error("Failed to load mod list:", e);
      modsError = e?.message || (typeof e === "string" ? e : JSON.stringify(e));
    } finally {
      isLoadingMods = false;
    }
  }

  $effect(() => {
    if (modpackDetailTab !== "mods") return;
    // Track dependencies for re-run when selection changes
    selectedVersionId;
    selectedModpackDetail?.platform;
    void loadModList();
  });

  async function handleInstall(versionId: string) {
    if (!selectedModpackDetail) return;

    console.log("[modpacks] Installing modpack:", {
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
      console.log("[modpacks] Install successful, closing modal");
      closeModpackDetail();
      alert(`Successfully installed ${selectedModpackDetail.name}!`);
    } else if (modpacksStore.installError && !modpacksStore.installError.includes("CANCELLED")) {
      console.error("[modpacks] Install failed:", modpacksStore.installError);
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
        kind: "warning",
      }
    );

    if (confirmed) {
      modpackInstallStore.cancel();
    }
  }

  const platforms: { value: ModpackPlatform | "all"; label: string }[] = [
    { value: "all", label: "All" },
    { value: "modrinth", label: "Modrinth" },
    { value: "curseforge", label: "CurseForge" },
    { value: "ftb", label: "FTB" },
    { value: "technic", label: "Technic" },
    { value: "atlauncher", label: "ATLauncher" },
  ];

  const sortOptions: { value: ModpackSortBy; label: string }[] = [
    { value: "downloads", label: "Downloads" },
    { value: "recentlyUpdated", label: "Recently Updated" },
    { value: "relevance", label: "Relevance" },
    { value: "name", label: "Name" },
  ];
</script>

<div class="space-y-6">
  <!-- Header -->
  <div class="flex items-center justify-between gap-4">
    <h1 class="text-2xl">Modpacks</h1>
    {#if hasAnyFilters}
      <div class="flex items-center gap-2">
        <Button
          variant="outline"
          size="sm"
          onclick={() => (showFilters = !showFilters)}
        >
          <Filter class="h-4 w-4 mr-2" />
          Filters
          {#if modpacksStore.mcVersion || modpacksStore.loader || modpacksStore.category}
            <span class="ml-1 bg-primary text-primary-foreground text-xs px-1.5 rounded-full">
              !
            </span>
          {/if}
        </Button>
      </div>
    {/if}
  </div>

  <!-- Platform Tabs -->
  <div class="flex gap-2 flex-wrap">
    {#each platforms as { value, label } (value)}
      <Button
        variant={modpacksStore.platform === (value === "all" ? null : value) ? "default" : "secondary"}
        size="sm"
        onclick={() => handlePlatformChange(value)}
      >
        {label}
      </Button>
    {/each}
  </div>

  <!-- Search and Sort -->
  <div class="flex items-center gap-4" data-tutorial="modpack-search">
    <div class="relative flex-1 max-w-md">
      <Search class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground z-10" />
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
      <Select.Trigger class="w-48 border-2 border-border bg-card">
        <ArrowUpDown class="h-4 w-4 mr-2" />
        {sortOptions.find((o) => o.value === modpacksStore.sortBy)?.label || "Sort by"}
      </Select.Trigger>
      <Select.Content class="border-2 border-border bg-card">
        {#each sortOptions as { value, label } (value)}
          <Select.Item {value} {label}>{label}</Select.Item>
        {/each}
      </Select.Content>
    </Select.Root>
  </div>

  <!-- Filters Panel -->
  {#if showFilters && hasAnyFilters}
    <div class="bg-card border-2 border-border p-4 space-y-4">
      <div class="flex items-center justify-between">
        <h3 class="font-semibold">Filters</h3>
        <Button variant="ghost" size="sm" onclick={clearFilters}>
          <X class="h-4 w-4 mr-1" />
          Clear
        </Button>
      </div>
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
          {#if showVersionFilter}
            <div>
              <p class="text-sm text-muted-foreground block mb-1">Minecraft Version</p>
              <Select.Root
                type="single"
                value={modpacksStore.mcVersion || ""}
                onValueChange={handleVersionChange}
              >
                <Select.Trigger class="w-full border-2 border-border bg-background">
                  {modpacksStore.mcVersion || "Any version"}
                </Select.Trigger>
                <Select.Content class="border-2 border-border bg-card max-h-[300px]">
                  <Select.Item value="" label="Any version">Any version</Select.Item>
                  {#each versionsStore.versions.filter((v) => v.type === "release") as version (version.id)}
                    <Select.Item value={version.id} label={version.id}>{version.id}</Select.Item>
                  {/each}
                </Select.Content>
              </Select.Root>
            </div>
          {/if}
          {#if showLoaderFilter}
            <div>
              <p class="text-sm text-muted-foreground block mb-1">Mod Loader</p>
              <Select.Root
                type="single"
                value={modpacksStore.loader || "any"}
                onValueChange={handleLoaderChange}
              >
                <Select.Trigger class="w-full border-2 border-border bg-background">
                  {modpacksStore.loader ? modpacksStore.loader.charAt(0).toUpperCase() + modpacksStore.loader.slice(1) : "Any loader"}
                </Select.Trigger>
                <Select.Content class="border-2 border-border bg-card">
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
              <p class="text-sm text-muted-foreground block mb-1">Categories</p>
              <button
                type="button"
                class="w-full h-9 px-3 border-2 border-border bg-background text-sm text-left flex items-center justify-between"
                onclick={() => categoriesOpen = !categoriesOpen}
              >
                {#if selectedCategories.length === 0}
                  <span class="text-muted-foreground">Any category</span>
                {:else if selectedCategories.length === 1}
                  <span class="capitalize">{selectedCategories[0].replace(/-/g, " ")}</span>
                {:else}
                  <span>{selectedCategories.length} selected</span>
                {/if}
                <ChevronDown class="h-4 w-4 text-muted-foreground {categoriesOpen ? 'rotate-180' : ''} transition-transform" />
              </button>
              {#if categoriesOpen}
                <div class="absolute z-50 top-full left-0 right-0 mt-1 border-2 border-border bg-card max-h-[250px] overflow-y-auto shadow-lg">
                  <div class="p-2 border-b border-border flex items-center justify-between sticky top-0 bg-card">
                    <span class="text-sm font-medium">Categories</span>
                    {#if selectedCategories.length > 0}
                      <button
                        type="button"
                        class="text-xs text-muted-foreground hover:text-foreground"
                        onclick={clearCategories}
                      >
                        Clear all
                      </button>
                    {/if}
                  </div>
                  <div class="p-2 space-y-1">
                    {#each availableCategories as category (category)}
                      <button
                        type="button"
                        class="w-full flex items-center gap-2 px-2 py-1.5 text-sm hover:bg-muted/50 rounded text-left"
                        onclick={() => toggleCategory(category)}
                      >
                        <Checkbox
                          checked={selectedCategories.includes(category)}
                          class="pointer-events-none"
                        />
                        <span class="capitalize">{category.replace(/-/g, " ")}</span>
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
    <div class="bg-destructive/10 border-2 border-destructive p-4 text-destructive text-sm">
      {modpacksStore.searchError}
      <button class="underline ml-2" onclick={() => modpacksStore.clearSearchError()}>
        Dismiss
      </button>
    </div>
  {/if}

  <!-- Results -->
  {#if modpacksStore.isSearching && modpacksStore.modpacks.length === 0}
    <div class="flex items-center justify-center py-12">
      <Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
    </div>
  {:else if modpacksStore.modpacks.length === 0}
    <div class="border-2 border-dashed border-border bg-card/50 p-12 text-center">
      <Package class="mx-auto h-12 w-12 text-muted-foreground/50" />
      <p class="mt-4 text-sm text-muted-foreground">
        {searchInput ? "No modpacks match your search" : "Search for modpacks to get started"}
      </p>
    </div>
  {:else}
    <!-- Results count -->
    <div class="text-sm text-muted-foreground">
      Showing {modpacksStore.modpacks.length} of {modpacksStore.totalCount} modpacks
    </div>

    <!-- Modpack Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {#each modpacksStore.modpacks as modpack, index (`${modpack.platform}-${modpack.id}`)}
        <button
          class="border-2 border-border bg-card p-4 hover:border-primary/50 transition-colors text-left cursor-pointer"
          onclick={() => handleModpackClick(modpack)}
          data-tutorial={index === 0 ? "modpack-card" : undefined}
        >
          <div class="flex gap-3">
            {#if modpack.iconUrl}
              <img
                src={modpack.iconUrl}
                alt={modpack.name}
                class="w-16 h-16 object-cover rounded"
              />
            {:else}
              <div class="w-16 h-16 bg-muted flex items-center justify-center rounded">
                <Package class="h-8 w-8 text-muted-foreground/50" />
              </div>
            {/if}
            <div class="flex-1 min-w-0">
              <h3 class="font-bold truncate">{modpack.name}</h3>
              <p class="text-sm text-muted-foreground truncate">{modpack.author}</p>
              <div class="flex items-center gap-2 mt-1 flex-wrap">
                <span
                  class="text-xs px-1.5 py-0.5 border rounded {getPlatformColor(modpack.platform)}"
                >
                  {modpack.platform}
                </span>
                {#each (modpack.loaders || []).filter(l => l && l !== "unknown" && l !== "vanilla").slice(0, 2) as loader (loader)}
                  <span class="text-xs px-1.5 py-0.5 rounded {getLoaderColor(loader)}">
                    {loader}
                  </span>
                {/each}
              </div>
            </div>
          </div>
          <p class="text-sm text-muted-foreground mt-3 line-clamp-2">
            {modpack.description}
          </p>
          <div class="flex items-center gap-4 mt-3 text-xs text-muted-foreground">
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
          <Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
        </div>
      {/if}
    </div>
  {/if}
</div>

<!-- Modpack Detail Modal -->
 {#if selectedModpackDetail}
  <div class="fixed inset-x-0 top-[var(--titlebar-height)] h-[calc(100vh-var(--titlebar-height))] bg-black/50 flex items-center justify-center z-50 p-4">
    <div class="bg-card border-2 border-border max-w-6xl w-full max-h-[90vh] flex flex-col rounded-lg shadow-2xl">
      <!-- Header -->
      <div class="p-6 border-b border-border flex-shrink-0">
        <div class="flex gap-4">
          {#if selectedModpackDetail.iconUrl}
            <img
              src={selectedModpackDetail.iconUrl}
              alt={selectedModpackDetail.name}
              class="w-24 h-24 object-cover rounded"
            />
          {:else}
            <div class="w-24 h-24 bg-muted flex items-center justify-center rounded">
              <Package class="h-12 w-12 text-muted-foreground/50" />
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
            <div class="flex items-center gap-2 mt-2 flex-wrap">
              <span
                class="text-xs px-1.5 py-0.5 border rounded {getPlatformColor(
                  selectedModpackDetail.platform
                )}"
              >
                {selectedModpackDetail.platform}
              </span>
              {#each (selectedModpackDetail.loaders || []).filter(l => l && l !== "unknown" && l !== "vanilla") as loader (loader)}
                <span class="text-xs px-1.5 py-0.5 rounded {getLoaderColor(loader)}">
                  {loader}
                </span>
              {/each}
              <span class="flex items-center gap-1 text-xs text-muted-foreground">
                <Download class="h-3 w-3" />
                {formatDownloads(selectedModpackDetail.downloads)}
              </span>
            </div>
          </div>
        </div>
      </div>

      <div class="flex-1 overflow-hidden p-5 grid gap-4 grid-cols-1 md:grid-cols-[2.5fr_1fr] xl:grid-cols-[3fr_1fr] min-h-0">
          <div class="space-y-4 overflow-y-auto pr-1 min-h-0">
            <div class="flex items-center gap-2">
              <Button
                size="sm"
                variant={modpackDetailTab === "about" ? "default" : "secondary"}
                onclick={() => (modpackDetailTab = "about")}
              >
                About
              </Button>
              <Button
                size="sm"
                variant={modpackDetailTab === "gallery" ? "default" : "secondary"}
                disabled={(selectedModpackDetail.gallery?.length ?? 0) === 0}
                onclick={() => (modpackDetailTab = "gallery")}
              >
                Gallery
              </Button>
              <Button
                size="sm"
                variant={modpackDetailTab === "mods" ? "default" : "secondary"}
                disabled={!selectedVersionId || !isModListSupported(selectedModpackDetail.platform)}
                onclick={() => (modpackDetailTab = "mods")}
              >
                Mods
              </Button>
            </div>

            {#if modpackDetailTab === "mods"}
              {#if !isModListSupported(selectedModpackDetail.platform)}
                <p class="text-sm text-muted-foreground">Mod list is not available for this platform.</p>
              {:else if !selectedVersionId}
                <p class="text-sm text-muted-foreground">Select a version to view its mod list.</p>
              {:else if isLoadingMods}
                <div class="flex items-center gap-2 text-muted-foreground text-sm">
                  <Loader2 class="h-4 w-4 animate-spin" />
                  Loading mod list...
                </div>
              {:else if modsError}
                <div class="p-3 bg-destructive/10 border-2 border-destructive rounded text-destructive text-sm">
                  {modsError}
                </div>
              {:else if (modListCache[selectedVersionId]?.length ?? 0) === 0}
                <p class="text-sm text-muted-foreground">No mods found for this version.</p>
              {:else}
                <div class="space-y-2">
                  <div class="flex items-center justify-between">
                    <h3 class="text-sm font-semibold">Mods</h3>
                    <span class="text-xs text-muted-foreground">
                      {modListCache[selectedVersionId]?.length ?? 0} mods
                    </span>
                  </div>
                  <div class="border-2 border-border rounded-lg overflow-hidden">
                    <div class="max-h-[60vh] overflow-y-auto">
                      {#each modListCache[selectedVersionId] ?? [] as modItem (modItem.id)}
                        <button
                          type="button"
                          class="w-full flex items-center gap-3 p-3 text-left hover:bg-muted/50 border-b border-border last:border-b-0"
                          onclick={() => modItem.url && openUrl(modItem.url)}
                          disabled={!modItem.url}
                        >
                          {#if modItem.iconUrl}
                            <img
                              src={modItem.iconUrl}
                              alt={modItem.name}
                              class="w-10 h-10 rounded object-cover border border-border bg-muted"
                              loading="lazy"
                            />
                          {:else}
                            <div class="w-10 h-10 rounded border border-border bg-muted flex items-center justify-center text-muted-foreground text-xs">
                              MOD
                            </div>
                          {/if}
                          <div class="min-w-0">
                            <div class="font-medium text-sm truncate">{modItem.name}</div>
                            {#if modItem.author}
                              <div class="text-xs text-muted-foreground truncate">{modItem.author}</div>
                            {/if}
                          </div>
                        </button>
                      {/each}
                    </div>
                  </div>
                </div>
              {/if}
            {:else if modpackDetailTab === "gallery"}
              {#if (selectedModpackDetail.gallery?.length ?? 0) > 0}
                <div class="space-y-2">
                  <div class="flex items-center justify-between">
                    <h3 class="text-sm font-semibold">Gallery</h3>
                    <span class="text-xs text-muted-foreground">
                      {selectedModpackDetail.gallery?.length ?? 0} images
                    </span>
                  </div>
                  <div class="grid gap-3 sm:grid-cols-2">
                    {#each selectedModpackDetail.gallery ?? [] as image, idx (image.rawUrl ?? image.url)}
                      <button
                        type="button"
                        class="relative overflow-hidden rounded-lg border-2 border-border bg-muted/50 aspect-video text-left cursor-pointer"
                        onclick={() => openModpackLightbox(idx)}
                      >
                        <img
                          src={image.rawUrl ?? image.url}
                          alt={image.title ?? selectedModpackDetail.name}
                          class="w-full h-full object-cover"
                          loading="lazy"
                        />
                        {#if image.title || image.description}
                          <div class="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/80 via-black/40 to-transparent text-white text-xs p-2 space-y-1">
                            {#if image.title}
                              <div class="font-semibold leading-tight truncate">{image.title}</div>
                            {/if}
                            {#if image.description}
                              <p class="opacity-90 line-clamp-2 leading-snug">{image.description}</p>
                            {/if}
                          </div>
                        {/if}
                      </button>
                    {/each}
                  </div>
                </div>
              {:else}
                <p class="text-sm text-muted-foreground">No gallery available.</p>
              {/if}
            {:else}
              <div class="border-2 border-border rounded-lg bg-background/70 p-4 space-y-2">
                <div class="flex items-center justify-between gap-2">
                  <h3 class="text-sm font-semibold">About</h3>
                  <div class="flex items-center gap-2">
                    <Button
                      size="sm"
                      variant="secondary"
                      onclick={() => (descriptionExpanded = true)}
                      disabled={!selectedModpackDetail.body && !selectedModpackDetail.description}
                    >
                      <Maximize2 class="h-4 w-4 mr-1" />
                      Expand
                    </Button>
                    {#if selectedModpackDetail.url}
                      <a
                        href={selectedModpackDetail.url}
                        target="_blank"
                        rel="noopener noreferrer"
                        class="inline-flex items-center gap-1.5 text-xs bg-muted hover:bg-muted/80 px-2.5 py-1.5 rounded transition-colors"
                      >
                        <ExternalLink class="h-3.5 w-3.5" />
                        View on {selectedModpackDetail.platform}
                      </a>
                    {/if}
                  </div>
                </div>

                {#if modpacksStore.isLoadingDetail}
                  <div class="flex items-center gap-2 text-muted-foreground text-sm">
                    <Loader2 class="h-4 w-4 animate-spin" />
                    Loading description...
                  </div>
                {:else}
                  {#if modpacksStore.detailError}
                    <div class="p-3 bg-destructive/10 border-2 border-destructive rounded text-destructive text-sm">
                      {modpacksStore.detailError}
                    </div>
                  {/if}
                  {#if selectedModpackDetail.body || selectedModpackDetail.description}
                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <div
                      class="text-sm leading-relaxed [&_p]:mb-3 [&_ul]:list-disc [&_ul]:pl-5 [&_ol]:list-decimal [&_ol]:pl-5 [&_img]:max-w-full [&_img]:rounded-md [&_img]:my-2 [&_h1]:text-lg [&_h2]:text-base [&_h1]:font-semibold [&_h2]:font-semibold [&_a]:text-primary [&_a]:underline"
                      onclick={handleDescriptionLinkClick}
                    >
                      <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                      {@html renderDescription(selectedModpackDetail.body, selectedModpackDetail.description)}
                    </div>
                  {:else}
                    <p class="text-sm text-muted-foreground">No description available.</p>
                  {/if}
                {/if}
              </div>
            {/if}
          </div>

          <div class="space-y-3 overflow-y-auto pr-1 min-h-0">
            <div class="border-2 border-border rounded-lg bg-background/70 p-3">
              <h3 class="font-semibold mb-2 text-sm">Select Version</h3>
              {#if modpacksStore.isLoadingVersions}
                <div class="flex items-center gap-2 text-muted-foreground text-sm">
                  <Loader2 class="h-4 w-4 animate-spin" />
                  Loading versions...
                </div>
              {:else if modpacksStore.selectedModpackVersions.length === 0}
                <p class="text-sm text-muted-foreground">No versions available</p>
              {:else}
                <div class="space-y-1">
                  {#each modpacksStore.selectedModpackVersions.slice(0, 15) as version, versionIndex (version.id)}
                    {@const isSelected = selectedVersionId === version.id}
                    <button
                      type="button"
                      class="w-full p-2 rounded border-2 transition-colors text-left {isSelected
                        ? 'border-primary bg-primary/10'
                        : 'border-border hover:border-primary/50'}"
                      onclick={() => (selectedVersionId = version.id)}
                      data-tutorial={versionIndex === 0 ? "modpack-install" : undefined}
                    >
                      <div class="flex items-center gap-2">
                        {#if isSelected}
                          <Check class="h-4 w-4 text-primary" />
                        {/if}
                        <span class="font-medium text-sm">{version.name}</span>
                      </div>
                      <div class="text-xs text-muted-foreground mt-0.5">
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
        <div class="border-t border-border p-4 bg-card flex-shrink-0 flex justify-end gap-3">
          <Button variant="outline" onclick={closeModpackDetail}>
            Cancel
          </Button>
          <Button
            disabled={!selectedVersionId || modpackInstallStore.isInstalling}
            onclick={() => selectedVersionId && handleInstall(selectedVersionId)}
          >
            <Download class="h-4 w-4 mr-2" />
            Install
          </Button>
        </div>
      {/if}

      <!-- Sticky Install Progress -->
      {#if modpackInstallStore.isInstalling}
        <div class="border-t border-border p-4 bg-card flex-shrink-0 shadow-[0_-4px_6px_-1px_rgba(0,0,0,0.1)]">
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
                  <Loader2 class="h-4 w-4 animate-spin text-primary flex-shrink-0" />
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
              <StopCircle class="h-4 w-4 mr-1" />
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
  src={modpackLightboxIndex !== null ? (currentModpackGallery[modpackLightboxIndex]?.rawUrl ?? currentModpackGallery[modpackLightboxIndex]?.url ?? null) : null}
  filename={modpackLightboxIndex !== null ? (currentModpackGallery[modpackLightboxIndex]?.title ?? `Image ${modpackLightboxIndex + 1}`) : undefined}
  canPrev={modpackLightboxIndex !== null && modpackLightboxIndex > 0}
  canNext={modpackLightboxIndex !== null && modpackLightboxIndex < currentModpackGallery.length - 1}
  onClose={closeModpackLightbox}
  onPrev={prevModpackLightbox}
  onNext={nextModpackLightbox}
/>

<DescriptionModal
  open={descriptionExpanded && !!selectedModpackDetail}
  title={selectedModpackDetail ? `${selectedModpackDetail.name} — Description` : "Description"}
  html={selectedModpackDetail ? renderDescription(selectedModpackDetail.body, selectedModpackDetail.description) : ""}
  onClose={() => (descriptionExpanded = false)}
/>
