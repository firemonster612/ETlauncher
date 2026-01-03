<script lang="ts">
  import { onMount } from "svelte";
  import { Button } from "$lib/ui/button";
  import { Checkbox } from "$lib/ui/checkbox";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import * as settingsService from "$lib/services/settings";
  import { RefreshCw, RotateCcw } from "@lucide/svelte";

  let javaDetecting = $state(false);
  let saveStatus = $state<"idle" | "saving" | "saved" | "error">("idle");

  // Convenience getter for settings with defaults
  const settings = $derived(settingsStore.settings);

  onMount(() => {
    settingsStore.load();
  });

  async function detectJava() {
    javaDetecting = true;
    try {
      const detected = await settingsService.detectJava();
      if (detected) {
        await saveSettings({ javaPath: detected });
      }
    } finally {
      javaDetecting = false;
    }
  }

  async function saveSettings(updates: Record<string, unknown>) {
    saveStatus = "saving";
    try {
      await settingsStore.update(updates as any);
      saveStatus = "saved";
      setTimeout(() => {
        saveStatus = "idle";
      }, 2000);
    } catch {
      saveStatus = "error";
    }
  }

  async function resetToDefaults() {
    if (confirm("Reset all settings to defaults?")) {
      await settingsStore.reset();
    }
  }

  function formatMemory(mb: number | undefined): string {
    if (mb === undefined) return "...";
    if (mb >= 1024) {
      return `${(mb / 1024).toFixed(1)} GB`;
    }
    return `${mb} MB`;
  }
</script>

<div class="space-y-6 max-w-2xl">
  <div class="flex items-center justify-between">
    <h1 class="text-2xl">Settings</h1>
    {#if saveStatus === "saving"}
      <span class="text-sm text-muted-foreground">Saving...</span>
    {:else if saveStatus === "saved"}
      <span class="text-sm text-primary">Saved</span>
    {:else if saveStatus === "error"}
      <span class="text-sm text-destructive">Error saving</span>
    {/if}
  </div>

  {#if settingsStore.isLoading || !settings}
    <div class="text-muted-foreground">Loading settings...</div>
  {:else if settingsStore.error}
    <div class="text-destructive">Error: {settingsStore.error}</div>
  {:else}
    <!-- General Settings -->
    <section class="border-2 border-border bg-card p-4 space-y-4">
      <h3 class="text-sm uppercase tracking-wider text-muted-foreground">General</h3>

      <div class="space-y-2">
        <label for="instancesPath" class="text-sm">Instances Path</label>
        <div class="flex gap-2">
          <input
            id="instancesPath"
            type="text"
            value={settings.instancesPath}
            onchange={(e) => saveSettings({ instancesPath: e.currentTarget.value })}
            class="flex-1 h-9 px-3 bg-background border-2 border-border text-sm"
          />
        </div>
        <p class="text-xs text-muted-foreground">Where game instances are stored</p>
      </div>

      <div class="flex items-center justify-between">
        <div>
          <span class="text-sm">Close launcher when game starts</span>
          <p class="text-xs text-muted-foreground">Hide the launcher while playing</p>
        </div>
        <Checkbox
          checked={settings.closeLauncherOnGameStart}
          onCheckedChange={(checked) => saveSettings({ closeLauncherOnGameStart: !!checked })}
        />
      </div>

      <div class="flex items-center justify-between">
        <div>
          <span class="text-sm">Reopen launcher when game closes</span>
          <p class="text-xs text-muted-foreground">Show the launcher after exiting the game</p>
        </div>
        <Checkbox
          checked={settings.reopenLauncherOnGameClose}
          onCheckedChange={(checked) => saveSettings({ reopenLauncherOnGameClose: !!checked })}
        />
      </div>
    </section>

    <!-- Java Settings -->
    <section class="border-2 border-border bg-card p-4 space-y-4">
      <h3 class="text-sm uppercase tracking-wider text-muted-foreground">Java</h3>

      <div class="space-y-2">
        <label for="javaPath" class="text-sm">Java Path</label>
        <div class="flex gap-2">
          <input
            id="javaPath"
            type="text"
            value={settings.javaPath ?? ""}
            onchange={(e) => saveSettings({ javaPath: e.currentTarget.value || undefined })}
            placeholder="Auto-detect"
            class="flex-1 h-9 px-3 bg-background border-2 border-border text-sm"
          />
          <Button
            variant="outline"
            size="sm"
            onclick={detectJava}
            disabled={javaDetecting}
          >
            <RefreshCw class="h-4 w-4 {javaDetecting ? 'animate-spin' : ''}" />
          </Button>
        </div>
        <p class="text-xs text-muted-foreground">
          {settings.javaPath ? `Using: ${settings.javaPath}` : "Leave empty to auto-detect"}
        </p>
      </div>
    </section>

    <!-- Memory Settings -->
    <section class="border-2 border-border bg-card p-4 space-y-4">
      <h3 class="text-sm uppercase tracking-wider text-muted-foreground">Memory</h3>

      <div class="space-y-2">
        <div class="flex justify-between text-sm">
          <span>Minimum RAM</span>
          <span class="text-primary">{formatMemory(settings.memoryMinMb)}</span>
        </div>
        <input
          type="range"
          min="256"
          max="8192"
          step="256"
          value={settings.memoryMinMb}
          onchange={(e) => {
            const value = parseInt(e.currentTarget.value);
            if (value <= settings.memoryMaxMb) {
              saveSettings({ memoryMinMb: value });
            }
          }}
          class="w-full h-2 accent-primary"
        />
      </div>

      <div class="space-y-2">
        <div class="flex justify-between text-sm">
          <span>Maximum RAM</span>
          <span class="text-primary">{formatMemory(settings.memoryMaxMb)}</span>
        </div>
        <input
          type="range"
          min="512"
          max="16384"
          step="512"
          value={settings.memoryMaxMb}
          onchange={(e) => {
            const value = parseInt(e.currentTarget.value);
            if (value >= settings.memoryMinMb) {
              saveSettings({ memoryMaxMb: value });
            }
          }}
          class="w-full h-2 accent-primary"
        />
      </div>

      <p class="text-xs text-muted-foreground">
        Default memory allocation for new instances. Can be overridden per-instance.
      </p>
    </section>

    <!-- Downloads Settings -->
    <section class="border-2 border-border bg-card p-4 space-y-4">
      <h3 class="text-sm uppercase tracking-wider text-muted-foreground">Downloads</h3>

      <div class="space-y-2">
        <div class="flex justify-between text-sm">
          <span>Concurrent Downloads</span>
          <span class="text-primary">{settings.concurrentDownloads}</span>
        </div>
        <input
          type="range"
          min="1"
          max="16"
          step="1"
          value={settings.concurrentDownloads}
          onchange={(e) => saveSettings({ concurrentDownloads: parseInt(e.currentTarget.value) })}
          class="w-full h-2 accent-primary"
        />
        <p class="text-xs text-muted-foreground">
          Number of files to download simultaneously
        </p>
      </div>
    </section>

    <!-- Version Settings -->
    <section class="border-2 border-border bg-card p-4 space-y-4">
      <h3 class="text-sm uppercase tracking-wider text-muted-foreground">Versions</h3>

      <div class="flex items-center justify-between">
        <div>
          <span class="text-sm">Show snapshots</span>
          <p class="text-xs text-muted-foreground">Include snapshot versions in version list</p>
        </div>
        <Checkbox
          checked={settings.showSnapshots}
          onCheckedChange={(checked) => saveSettings({ showSnapshots: !!checked })}
        />
      </div>

      <div class="flex items-center justify-between">
        <div>
          <span class="text-sm">Show old versions</span>
          <p class="text-xs text-muted-foreground">Include alpha and beta versions</p>
        </div>
        <Checkbox
          checked={settings.showOldVersions}
          onCheckedChange={(checked) => saveSettings({ showOldVersions: !!checked })}
        />
      </div>
    </section>

    <!-- Reset -->
    <section class="border-2 border-border bg-card p-4">
      <div class="flex items-center justify-between">
        <div>
          <span class="text-sm">Reset to Defaults</span>
          <p class="text-xs text-muted-foreground">Restore all settings to their default values</p>
        </div>
        <Button variant="outline" size="sm" onclick={resetToDefaults}>
          <RotateCcw class="h-4 w-4 mr-2" />
          Reset
        </Button>
      </div>
    </section>
  {/if}
</div>
