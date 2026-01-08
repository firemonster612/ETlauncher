<script lang="ts">
  import { Play, Square, X, Settings, Trash2, Clock, Calendar, Loader2, PackagePlus, ExternalLink } from "@lucide/svelte";
  import { Button } from "$lib/ui/button";
  import DownloadProgress from "$lib/components/DownloadProgress.svelte";
  import { parseIconPath, getIconUrl } from "$lib/utils/icons";
  import type { Instance, LoaderType, LaunchStatus } from "$lib/types";

  interface Props {
    instance: Instance;
    status: string | null;
    launchStatus: LaunchStatus | undefined;
    onLaunch: (instanceId: string) => void;
    onKill: (instanceId: string) => void;
    onOpenSettings: (instance: Instance) => void;
    onOpenContentBrowser: (instance: Instance) => void;
    onDelete: (instanceId: string) => void;
    onCardClick: (instance: Instance) => void;
  }

  let {
    instance,
    status,
    launchStatus,
    onLaunch,
    onKill,
    onOpenSettings,
    onOpenContentBrowser,
    onDelete,
    onCardClick,
  }: Props = $props();

  let isPlayButtonHovered = $state(false);

  function formatPlayTime(seconds: number): string {
    if (seconds < 60) return "< 1 min";
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    if (hours === 0) return `${minutes}m`;
    return `${hours}h ${minutes}m`;
  }

  function formatDate(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleDateString();
  }

  function getLoaderColor(loader: LoaderType): string {
    switch (loader) {
      case "fabric":
        return "bg-amber-500/20 text-amber-500 border-amber-500/50";
      case "forge":
        return "bg-orange-500/20 text-orange-500 border-orange-500/50";
      case "neoforge":
        return "bg-red-500/20 text-red-500 border-red-500/50";
      case "quilt":
        return "bg-purple-500/20 text-purple-500 border-purple-500/50";
      default:
        return "bg-green-500/20 text-green-500 border-green-500/50";
    }
  }

  function getIconSrc(iconPath: string | undefined): string {
    const icon = parseIconPath(iconPath);
    if (icon) {
      return getIconUrl(icon);
    }
    // Fallback to creeper
    return "/icons/entities/creeper/creeper.png";
  }

  function handleCardClick(e: MouseEvent) {
    // Don't trigger card click if clicking on a button
    if ((e.target as HTMLElement).closest("button")) return;
    onCardClick(instance);
  }

  function handlePlayButtonClick(e: MouseEvent) {
    e.stopPropagation();
    if (status === "running") {
      onKill(instance.id);
    } else {
      onLaunch(instance.id);
    }
  }
</script>

<div
  class="border-2 border-border bg-card p-4 hover:border-primary/50 transition-colors group cursor-pointer relative"
  onclick={handleCardClick}
  onkeydown={(e) => e.key === "Enter" && onCardClick(instance)}
  role="button"
  tabindex="0"
>
  <!-- Open detail indicator - top right -->
  <div class="absolute top-3 right-3 p-1.5 rounded bg-muted/50 group-hover:bg-primary/20 transition-colors" title="View details">
    <ExternalLink class="h-4 w-4 text-muted-foreground group-hover:text-primary transition-colors" />
  </div>

  <!-- Main content area -->
  <div class="flex gap-4">
    <!-- Icon -->
    <div class="flex-shrink-0">
      <img
        src={getIconSrc(instance.iconPath)}
        alt="{instance.name} icon"
        class="w-16 h-16 pixelated"
      />
    </div>

    <!-- Info -->
    <div class="flex-1 min-w-0">
      <h3 class="text-lg font-bold truncate pr-8">{instance.name}</h3>
      <div class="flex items-center gap-2 mt-1 flex-wrap">
        <span class="text-sm text-muted-foreground">{instance.minecraftVersion}</span>
        <span
          class="text-xs px-1.5 py-0.5 border rounded capitalize {getLoaderColor(instance.loaderType)}"
        >
          {instance.loaderType}
          {#if instance.loaderVersion}
            <span class="opacity-75 ml-1">{instance.loaderVersion}</span>
          {/if}
        </span>
      </div>

      <div class="flex items-center gap-4 mt-3 text-xs text-muted-foreground">
        <span class="flex items-center gap-1">
          <Clock class="h-3 w-3" />
          {formatPlayTime(instance.totalPlayTime)}
        </span>
        <span class="flex items-center gap-1">
          <Calendar class="h-3 w-3" />
          {formatDate(instance.createdAt)}
        </span>
      </div>
    </div>
  </div>

  <!-- Bottom bar with play button and actions -->
  <div class="flex items-center justify-between gap-2 mt-4 pt-3 border-t border-border">
    <!-- Play button - always visible -->
    <Button
      variant="default"
      size="sm"
      onclick={handlePlayButtonClick}
      onmouseenter={() => isPlayButtonHovered = true}
      onmouseleave={() => isPlayButtonHovered = false}
      disabled={status !== null && status !== "stopped" && status !== "crashed" && status !== "running"}
    >
      {#if status === "preparing" || status === "launching"}
        <Loader2 class="h-4 w-4 animate-spin mr-2" />
        Launching
      {:else if status === "running"}
        {#if isPlayButtonHovered}
          <X class="h-4 w-4 mr-2" />
        {:else}
          <Square class="h-4 w-4 mr-2" />
        {/if}
        Running
      {:else}
        <Play class="h-4 w-4 mr-2" />
        Play
      {/if}
    </Button>

    <!-- Action buttons -->
    <div class="flex items-center gap-1">
      <Button
        variant="ghost"
        size="sm"
        onclick={(e: MouseEvent) => { e.stopPropagation(); onOpenSettings(instance); }}
        title="Instance settings"
        data-tutorial="instance-settings-btn"
      >
        <Settings class="h-4 w-4" />
      </Button>
      <Button
        variant="ghost"
        size="sm"
        onclick={(e: MouseEvent) => { e.stopPropagation(); onOpenContentBrowser(instance); }}
        title="Add mods, shaders, resource packs"
        data-tutorial="content-browser-btn"
      >
        <PackagePlus class="h-4 w-4" />
      </Button>
      <Button
        variant="ghost"
        size="sm"
        class="text-destructive hover:text-destructive hover:bg-destructive/10"
        onclick={(e: MouseEvent) => { e.stopPropagation(); onDelete(instance.id); }}
        title="Delete instance"
      >
        <Trash2 class="h-4 w-4" />
      </Button>
    </div>
  </div>

  <!-- Download Progress -->
  {#if launchStatus?.status === "downloading"}
    {@const progress = launchStatus.progress}
    <div class="mt-3 pt-3 border-t border-border">
      <DownloadProgress
        stage="Downloading game files"
        progress={progress.totalBytes > 0 ? (progress.downloadedBytes / progress.totalBytes) * 100 : 0}
        currentItem={progress.currentFile}
        totalBytes={progress.totalBytes}
        downloadedBytes={progress.downloadedBytes}
        compact
      />
    </div>
  {/if}
</div>
