<script lang="ts">
  import { ArrowLeft, Loader2, Search, Trash2 } from "@lucide/svelte";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { ask } from "@tauri-apps/plugin-dialog";
  import { Button } from "$lib/ui/button";
  import { Input } from "$lib/ui/input";
  import * as Select from "$lib/ui/select";
  import ScreenshotLightbox from "$lib/components/ScreenshotLightbox.svelte";
  import * as instanceDetailService from "$lib/services/instance-detail";
  import type { DateFilter, Screenshot } from "$lib/types";

  interface Props {
    instanceId: string;
    onBack: () => void;
  }

  let { instanceId, onBack }: Props = $props();

  let screenshots = $state<Screenshot[]>([]);
  let isLoading = $state(false);
  let search = $state("");
  let dateFilter = $state<DateFilter>("all");
  let error = $state<string | null>(null);
  let previewSources = $state<Record<string, string>>({});

  let lightboxIndex = $state<number | null>(null);
  let lightboxData = $state<string | null>(null);
  let lightboxLoading = $state(false);

  let lastLoadedId = $state<string | null>(null);
  const screenshotDateFormatter = new Intl.DateTimeFormat(undefined);
  const formatDate = (timestamp: number) => screenshotDateFormatter.format(timestamp);

  $effect(() => {
    if (instanceId && instanceId !== lastLoadedId) {
      loadScreenshots();
    }
  });

  const filteredScreenshots = $derived(
    screenshots.filter((shot) => {
      const query = search.trim().toLowerCase();
      const matchesSearch =
        query.length === 0 ||
        shot.filename.toLowerCase().includes(query) ||
        formatDate(shot.takenAt).toLowerCase().includes(query);

      const matchesDate = (() => {
        if (dateFilter === "all") return true;
        const now = Date.now();
        const msInDay = 24 * 60 * 60 * 1000;
        if (dateFilter === "today") {
          const start = now - (now % msInDay);
          return shot.takenAt >= start;
        }
        if (dateFilter === "week") {
          const weekAgo = now - 7 * 24 * 60 * 60 * 1000;
          return shot.takenAt >= weekAgo;
        }
        if (dateFilter === "month") {
          const monthAgo = now - 30 * 24 * 60 * 60 * 1000;
          return shot.takenAt >= monthAgo;
        }
        return true;
      })();

      return matchesSearch && matchesDate;
    })
  );

  async function loadScreenshots() {
    if (!instanceId) return;
    isLoading = true;
    error = null;
    lightboxIndex = null;
    lightboxData = null;

    try {
      const response = await instanceDetailService.getInstanceScreenshots(instanceId);
      screenshots = response.screenshots;
      lastLoadedId = instanceId;
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load screenshots";
      console.error("Failed to load screenshots:", e);
    } finally {
      isLoading = false;
    }
  }

  async function openLightbox(index: number) {
    lightboxIndex = index;
    await loadLightboxImage(index);
  }

  async function loadLightboxImage(index: number) {
    const shot = filteredScreenshots[index];
    if (!shot) return;

    lightboxLoading = true;
    try {
      const data = await instanceDetailService.getScreenshotData(instanceId, shot.filename);
      const dataUrl = `data:image/png;base64,${data}`;
      lightboxData = dataUrl;
      previewSources = { ...previewSources, [shot.filename]: dataUrl };
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load screenshot";
      console.error("Failed to load screenshot data:", e);
    } finally {
      lightboxLoading = false;
    }
  }

  function closeLightbox() {
    lightboxIndex = null;
    lightboxData = null;
  }

  async function loadPreview(shot: Screenshot) {
    if (previewSources[shot.filename]) return;
    try {
      const data = await instanceDetailService.getScreenshotData(instanceId, shot.filename);
      previewSources = {
        ...previewSources,
        [shot.filename]: `data:image/png;base64,${data}`,
      };
    } catch (e) {
      console.error("Failed to load screenshot preview", e);
    }
  }

  function goPrev() {
    if (lightboxIndex === null || lightboxIndex === 0) return;
    const nextIndex = lightboxIndex - 1;
    lightboxIndex = nextIndex;
    loadLightboxImage(nextIndex);
  }

  function goNext() {
    if (lightboxIndex === null) return;
    const nextIndex = lightboxIndex + 1;
    if (nextIndex >= filteredScreenshots.length) return;
    lightboxIndex = nextIndex;
    loadLightboxImage(nextIndex);
  }

  const canPrev = $derived(lightboxIndex !== null && lightboxIndex > 0);
  const canNext = $derived(
    lightboxIndex !== null && lightboxIndex < filteredScreenshots.length - 1
  );

  async function handleDelete(shot: Screenshot, event: MouseEvent) {
    event.stopPropagation();

    const confirmed = await ask(`Delete screenshot "${shot.filename}"?`, {
      title: "Delete Screenshot",
      kind: "warning",
    });

    if (!confirmed) {
      return;
    }

    try {
      await instanceDetailService.deleteScreenshot(instanceId, shot.filename);
      // Remove from local state
      screenshots = screenshots.filter(s => s.filename !== shot.filename);
      // Close lightbox if the deleted screenshot was open
      if (lightboxIndex !== null && filteredScreenshots[lightboxIndex]?.filename === shot.filename) {
        closeLightbox();
      }
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to delete screenshot";
      console.error("Failed to delete screenshot:", e);
    }
  }
</script>

<div class="flex flex-col h-full">
  <div class="flex items-center justify-between px-6 py-4 border-b border-border">
    <div class="flex items-center gap-3">
      <Button variant="ghost" size="icon" onclick={onBack} aria-label="Back">
        <ArrowLeft class="h-5 w-5" />
      </Button>
      <div>
        <p class="text-xs uppercase tracking-wide text-muted-foreground">Screenshots</p>
        <h2 class="text-xl font-semibold">Gallery</h2>
      </div>
    </div>
  </div>

  <div class="p-6 flex flex-col gap-4 overflow-y-auto flex-1">
    <div class="flex flex-col md:flex-row md:items-center gap-3">
      <div class="flex-1 relative">
        <Input
          placeholder="Search screenshots..."
          value={search}
          oninput={(e) => (search = e.currentTarget.value)}
          class="pl-10"
        />
        <Search class="h-4 w-4 text-muted-foreground absolute left-3 top-1/2 -translate-y-1/2" />
      </div>

      <Select.Root type="single" value={dateFilter} onValueChange={(v) => (dateFilter = v as DateFilter)}>
        <Select.Trigger class="h-10 border border-border bg-background min-w-[160px]">
          {{
            all: "All dates",
            today: "Today",
            week: "This week",
            month: "This month",
            custom: "Custom",
          }[dateFilter]}
        </Select.Trigger>
        <Select.Content class="border border-border bg-card">
          <Select.Item value="all" label="All dates">All dates</Select.Item>
          <Select.Item value="today" label="Today">Today</Select.Item>
          <Select.Item value="week" label="This week">This week</Select.Item>
          <Select.Item value="month" label="This month">This month</Select.Item>
        </Select.Content>
      </Select.Root>
    </div>

    {#if error}
      <div class="border border-destructive/60 bg-destructive/10 text-destructive text-sm px-4 py-3 rounded">
        {error}
      </div>
    {/if}

    {#if isLoading}
      <div class="flex items-center gap-3 text-muted-foreground">
        <Loader2 class="h-5 w-5 animate-spin" />
        <span>Loading screenshots...</span>
      </div>
    {:else if filteredScreenshots.length === 0}
      <p class="text-muted-foreground text-sm">No screenshots found.</p>
    {:else}
      <div class="grid gap-3 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4">
        {#each filteredScreenshots as shot, index (shot.filename)}
          <button
            class="border border-border rounded overflow-hidden bg-muted/30 aspect-[4/3] group relative"
            onclick={() => openLightbox(index)}
            title="Open screenshot"
          >
            <img
              src={previewSources[shot.filename] ?? convertFileSrc(shot.path)}
              alt={shot.filename}
              class="h-full w-full object-cover transition-transform group-hover:scale-[1.02]"
              loading="lazy"
              onerror={() => loadPreview(shot)}
            />
            <div class="absolute top-2 right-2">
              <Button
                variant="destructive"
                size="icon"
                class="h-7 w-7 opacity-0 group-hover:opacity-100 transition-opacity"
                onclick={(e) => handleDelete(shot, e)}
                aria-label="Delete screenshot"
              >
                <Trash2 class="h-4 w-4" />
              </Button>
            </div>
            <div class="absolute bottom-0 left-0 right-0 bg-black/50 text-white text-xs px-2 py-1 flex items-center justify-between">
              <span class="truncate">{shot.filename}</span>
              <span>{formatDate(shot.takenAt)}</span>
            </div>
          </button>
        {/each}
      </div>
    {/if}
  </div>
</div>

<ScreenshotLightbox
  open={lightboxIndex !== null}
  src={lightboxData}
  filename={lightboxIndex !== null ? filteredScreenshots[lightboxIndex]?.filename : undefined}
  isLoading={lightboxLoading}
  canPrev={canPrev}
  canNext={canNext}
  onClose={closeLightbox}
  onPrev={goPrev}
  onNext={goNext}
/>
