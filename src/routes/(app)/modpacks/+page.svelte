<script lang="ts">
  import { onMount } from "svelte";
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
    Check,
  } from "@lucide/svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { Button } from "$lib/ui/button";
  import * as Select from "$lib/ui/select";
  import { modpacksStore } from "$lib/stores/modpacks.svelte";
  import { versionsStore } from "$lib/stores/versions.svelte";
  import DownloadProgress from "$lib/components/DownloadProgress.svelte";
  import type { Modpack, ModpackPlatform, ModpackSortBy, LoaderType, ModpackInstallProgress } from "$lib/types";

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
  let installProgress = $state<ModpackInstallProgress | null>(null);
  let loadMoreSentinel: HTMLDivElement | undefined = $state();
  let selectedCategories = $state<string[]>([]);
  let categoriesOpen = $state(false);

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

  // Set up infinite scroll observer when sentinel is available
  $effect(() => {
    if (!loadMoreSentinel) return;

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

  async function handleModpackClick(modpack: Modpack) {
    selectedModpackDetail = modpack;
    await modpacksStore.selectModpack(modpack);
  }

  function closeModpackDetail() {
    selectedModpackDetail = null;
    modpacksStore.clearSelection();
  }

  async function handleInstall(versionId: string) {
    if (!selectedModpackDetail) return;

    console.log("[modpacks] Installing modpack:", {
      name: selectedModpackDetail.name,
      platform: selectedModpackDetail.platform,
      modpackId: selectedModpackDetail.id,
      versionId,
    });

    installProgress = null;
    let unlisten: UnlistenFn | undefined;

    try {
      // Set up progress listener before starting install
      unlisten = await listen<ModpackInstallProgress>("modpack_install_progress", (event) => {
        installProgress = event.payload;
      });

      const instance = await modpacksStore.installModpack(
        selectedModpackDetail.platform,
        selectedModpackDetail.id,
        versionId,
        selectedModpackDetail.name
      );

      if (instance) {
        console.log("[modpacks] Install successful, closing modal");
        closeModpackDetail();
        // TODO: Navigate to instances page or show success toast
        alert(`Successfully installed ${selectedModpackDetail.name}!`);
      } else {
        console.error("[modpacks] Install failed:", modpacksStore.installError);
        alert(`Failed to install: ${modpacksStore.installError}`);
      }
    } finally {
      unlisten?.();
      installProgress = null;
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
    {#each platforms as { value, label }}
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
  <div class="flex items-center gap-4">
    <div class="relative flex-1 max-w-md">
      <Search class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
      <input
        type="text"
        placeholder="Search modpacks..."
        value={searchInput}
        oninput={(e) => handleSearchInput(e.currentTarget.value)}
        class="w-full h-9 pl-9 pr-3 bg-card border-2 border-border text-sm focus:border-primary outline-none"
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
        {#each sortOptions as { value, label }}
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
              <label class="text-sm text-muted-foreground block mb-1">Minecraft Version</label>
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
                  {#each versionsStore.versions.filter((v) => v.type === "release") as version}
                    <Select.Item value={version.id} label={version.id}>{version.id}</Select.Item>
                  {/each}
                </Select.Content>
              </Select.Root>
            </div>
          {/if}
          {#if showLoaderFilter}
            <div>
              <label class="text-sm text-muted-foreground block mb-1">Mod Loader</label>
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
              <label class="text-sm text-muted-foreground block mb-1">Categories</label>
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
                    {#each availableCategories as category}
                      <button
                        type="button"
                        class="w-full flex items-center gap-2 px-2 py-1.5 text-sm hover:bg-muted/50 rounded text-left"
                        onclick={() => toggleCategory(category)}
                      >
                        <div class="w-4 h-4 border border-border rounded flex items-center justify-center shrink-0 {selectedCategories.includes(category) ? 'bg-primary border-primary' : ''}">
                          {#if selectedCategories.includes(category)}
                            <Check class="h-3 w-3 text-primary-foreground" />
                          {/if}
                        </div>
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
      {#each modpacksStore.modpacks as modpack (`${modpack.platform}-${modpack.id}`)}
        <button
          class="border-2 border-border bg-card p-4 hover:border-primary/50 transition-colors text-left cursor-pointer"
          onclick={() => handleModpackClick(modpack)}
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
                {#each modpack.loaders.slice(0, 2) as loader}
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
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
    <div class="bg-card border-2 border-border max-w-2xl w-full max-h-[80vh] overflow-y-auto">
      <!-- Header -->
      <div class="p-6 border-b border-border">
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
              {#each selectedModpackDetail.loaders as loader}
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

      <!-- Description -->
      <div class="p-6 border-b border-border">
        <p class="text-sm">{selectedModpackDetail.description}</p>
        {#if selectedModpackDetail.url}
          <a
            href={selectedModpackDetail.url}
            target="_blank"
            rel="noopener noreferrer"
            class="inline-flex items-center gap-1.5 text-sm bg-muted hover:bg-muted/80 px-3 py-1.5 rounded mt-3 transition-colors"
          >
            <ExternalLink class="h-4 w-4" />
            View on {selectedModpackDetail.platform}
          </a>
        {/if}
      </div>

      <!-- Versions -->
      <div class="p-6">
        <h3 class="font-semibold mb-3">Versions</h3>
        {#if modpacksStore.isLoadingVersions}
          <div class="flex items-center gap-2 text-muted-foreground">
            <Loader2 class="h-4 w-4 animate-spin" />
            Loading versions...
          </div>
        {:else if modpacksStore.selectedModpackVersions.length === 0}
          <p class="text-sm text-muted-foreground">No versions available</p>
        {:else}
          <div class="space-y-2 max-h-[300px] overflow-y-auto">
            {#each modpacksStore.selectedModpackVersions.slice(0, 10) as version}
              <div class="flex items-center justify-between p-3 bg-muted/50 rounded">
                <div>
                  <div class="font-medium">{version.name}</div>
                  <div class="text-xs text-muted-foreground">
                    MC {version.mcVersion} &bull; {version.loaderType}
                    {#if version.releasedAt}
                      &bull; {new Date(version.releasedAt * 1000).toLocaleDateString()}
                    {/if}
                  </div>
                </div>
                <Button
                  size="sm"
                  onclick={() => handleInstall(version.id)}
                  disabled={modpacksStore.isInstalling}
                >
                  {#if modpacksStore.isInstalling}
                    <Loader2 class="h-4 w-4 mr-1 animate-spin" />
                    Installing...
                  {:else}
                    <Download class="h-4 w-4 mr-1" />
                    Install
                  {/if}
                </Button>
              </div>
            {/each}
          </div>

          <!-- Install Progress -->
          {#if modpacksStore.isInstalling && installProgress}
            <div class="border-t border-border pt-4 mt-4">
              <DownloadProgress
                stage={installProgress.stage}
                progress={installProgress.progress}
                currentItem={installProgress.currentItem}
                totalItems={installProgress.totalItems}
                completedItems={installProgress.completedItems}
              />
            </div>
          {/if}
        {/if}
      </div>
    </div>
  </div>
{/if}
