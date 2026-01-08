<script lang="ts">
  import { Download, Loader2, RefreshCw, FolderOpen, Copy, Upload } from "@lucide/svelte";
  import { save } from "@tauri-apps/plugin-dialog";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { Button } from "$lib/ui/button";
  import { Input } from "$lib/ui/input";
  import { Slider } from "$lib/ui/slider";
  import { Textarea } from "$lib/ui/textarea";
  import * as Sheet from "$lib/ui/sheet";
  import { instancesStore } from "$lib/stores/instances.svelte";
  import * as instanceService from "$lib/services/instance";
  import UpdateDialog from "./UpdateDialog.svelte";
  import IconPicker from "./IconPicker.svelte";
  import { makeIconPath, type EntityIcon } from "$lib/utils/icons";
  import type { Instance, UpdateInstanceRequest } from "$lib/types";

  interface Props {
    instance: Instance;
    open: boolean;
    onClose: () => void;
  }

  let { instance, open: isOpen, onClose }: Props = $props();

  // Update dialog state
  let showUpdateDialog = $state(false);

  function openUpdateDialog() {
    showUpdateDialog = true;
  }

  function handleInstanceUpdated() {
    // Reload the instances store to get updated data
    instancesStore.load();
  }

  // Form state - initialize from instance
  let name = $state("");
  let iconPath = $state<string | undefined>(undefined);
  let javaPath = $state("");
  let memoryMin = $state(512);
  let memoryMax = $state(4096);
  let jvmArgs = $state("");
  let gameArgs = $state("");
  let resolutionWidth = $state(0);
  let resolutionHeight = $state(0);

  let isSaving = $state(false);
  let saveError = $state<string | null>(null);

  const globalMemoryMinMb = $derived(settingsStore.settings?.memoryMinMb ?? 512);
  const globalMemoryMaxMb = $derived(settingsStore.settings?.memoryMaxMb ?? 4096);

  // Reset form when instance changes
  $effect(() => {
    name = instance.name;
    iconPath = instance.iconPath;
    javaPath = instance.javaPath || "";
    memoryMin = instance.memoryMinMb ?? globalMemoryMinMb;
    memoryMax = instance.memoryMaxMb ?? globalMemoryMaxMb;
    jvmArgs = instance.jvmArgs || "";
    gameArgs = instance.gameArgs || "";
    resolutionWidth = instance.resolutionWidth || 0;
    resolutionHeight = instance.resolutionHeight || 0;
  });

  function handleIconSelect(icon: EntityIcon) {
    iconPath = makeIconPath(icon);
  }

  // Ensure memoryMin <= memoryMax
  $effect(() => {
    if (memoryMin > memoryMax) {
      memoryMax = memoryMin;
    }
  });

  async function handleSave() {
    isSaving = true;
    saveError = null;

    const updates: UpdateInstanceRequest = {
      name: name !== instance.name ? name : undefined,
      iconPath: iconPath !== instance.iconPath ? iconPath : undefined,
      javaPath: javaPath || undefined,
      memoryMinMb: memoryMin !== globalMemoryMinMb ? memoryMin : undefined,
      memoryMaxMb: memoryMax !== globalMemoryMaxMb ? memoryMax : undefined,
      jvmArgs: jvmArgs || undefined,
      gameArgs: gameArgs || undefined,
      resolutionWidth: resolutionWidth || undefined,
      resolutionHeight: resolutionHeight || undefined,
    };

    // Remove undefined values
    Object.keys(updates).forEach((key) => {
      if (updates[key as keyof UpdateInstanceRequest] === undefined) {
        delete updates[key as keyof UpdateInstanceRequest];
      }
    });

    const result = await instancesStore.update(instance.id, updates);
    isSaving = false;

    if (result) {
      onClose();
    } else {
      saveError = instancesStore.error || "Failed to save settings";
    }
  }

  function formatMemory(mb: number): string {
    if (mb >= 1024) {
      return `${(mb / 1024).toFixed(1)} GB`;
    }
    return `${mb} MB`;
  }

  async function handleReinstallLoader() {
    if (instance.loaderType === "vanilla" || !instance.loaderVersion) return;
    await instancesStore.installLoader(instance.id, instance.loaderType, instance.loaderVersion);
  }

  async function handleOpenFolder() {
    await instanceService.openInstanceFolder(instance.id);
  }

  async function handleDuplicate() {
    await instancesStore.duplicate(instance.id, `${instance.name} (Copy)`);
    onClose();
  }

  let isExporting = $state(false);
  let exportError = $state<string | null>(null);

  async function handleExport() {
    try {
      const filePath = await save({
        title: "Export Instance",
        defaultPath: `${instance.name}.mrpack`,
        filters: [{ name: "Modrinth Pack", extensions: ["mrpack"] }],
      });

      if (!filePath) return;

      isExporting = true;
      exportError = null;
      await instanceService.exportInstance(instance.id, filePath);
      isExporting = false;
    } catch (e: unknown) {
      isExporting = false;
      exportError = e instanceof Error ? e.message : "Export failed";
    }
  }
</script>


<Sheet.Root bind:open={isOpen} onOpenChange={(open) => !open && onClose()}>
  <Sheet.Content side="right" class="w-full sm:max-w-lg overflow-y-auto">
    <Sheet.Header class="border-b border-border pb-4 px-6">
      <Sheet.Title>Instance Settings</Sheet.Title>
      <Sheet.Description>
        Configure settings for <span class="font-medium">{instance.name}</span>
      </Sheet.Description>
    </Sheet.Header>

    <div class="py-6 px-6 space-y-8">
      <!-- Instance Name -->
      <div class="space-y-3">
        <label for="instance-name" class="text-sm font-medium">Instance Name</label>
        <Input
          id="instance-name"
          type="text"
          bind:value={name}
          placeholder="My Instance"
        />
      </div>

      <!-- Instance Icon -->
      <div class="space-y-3">
        <p class="text-sm font-medium">Instance Icon</p>
        <p class="text-xs text-muted-foreground">Choose an icon for this instance</p>
        <IconPicker selected={iconPath} onSelect={handleIconSelect} />
      </div>

      <!-- Java Path -->
      <div class="space-y-3">
        <label for="java-path" class="text-sm font-medium">Java Path</label>
        <p class="text-xs text-muted-foreground">Leave empty to use the default Java installation</p>
        <Input
          id="java-path"
          type="text"
          bind:value={javaPath}
          placeholder="Auto-detect"
        />
      </div>

      <!-- Memory Allocation -->
      <div class="space-y-4">
        <div>
          <p class="text-sm font-medium">Memory Allocation</p>
          <p class="text-xs text-muted-foreground mt-1">Adjust the RAM allocated to Minecraft</p>
        </div>

        <div class="space-y-4">
          <div class="space-y-2">
            <div class="flex justify-between text-sm">
              <span>Minimum</span>
              <span class="text-primary">{formatMemory(memoryMin)}</span>
            </div>
            <Slider
              min={512}
              max={16384}
              step={512}
              bind:value={memoryMin}
            />
          </div>

          <div class="space-y-2">
            <div class="flex justify-between text-sm">
              <span>Maximum</span>
              <span class="text-primary">{formatMemory(memoryMax)}</span>
            </div>
            <Slider
              min={512}
              max={32768}
              step={512}
              bind:value={memoryMax}
            />
          </div>
        </div>
      </div>

      <!-- JVM Arguments -->
      <div class="space-y-3">
        <label for="jvm-args" class="text-sm font-medium">JVM Arguments</label>
        <p class="text-xs text-muted-foreground">Additional arguments passed to the Java Virtual Machine</p>
        <Textarea
          id="jvm-args"
          bind:value={jvmArgs}
          placeholder="-XX:+UseG1GC"
          rows={3}
        />
      </div>

      <!-- Game Arguments -->
      <div class="space-y-3">
        <label for="game-args" class="text-sm font-medium">Game Arguments</label>
        <p class="text-xs text-muted-foreground">Additional arguments passed to Minecraft</p>
        <Textarea
          id="game-args"
          bind:value={gameArgs}
          placeholder="--width 1920 --height 1080"
          rows={2}
        />
      </div>

      <!-- Resolution -->
      <div class="space-y-3">
        <p class="text-sm font-medium">Window Resolution</p>
        <p class="text-xs text-muted-foreground">Set to 0 for default resolution</p>
        <div class="flex gap-3 items-center">
          <Input
            type="number"
            bind:value={resolutionWidth}
            placeholder="Width"
            min={0}
            class="w-24"
          />
          <span class="text-muted-foreground">×</span>
          <Input
            type="number"
            bind:value={resolutionHeight}
            placeholder="Height"
            min={0}
            class="w-24"
          />
        </div>
      </div>

      <!-- Instance Info (Read-only) -->
      <div class="border-t border-border pt-6 space-y-4">
        <h4 class="text-sm font-medium text-muted-foreground">Instance Info</h4>
        <div class="text-sm space-y-2 bg-muted/50 p-4 rounded">
          <p><span class="text-muted-foreground">Minecraft:</span> <span class="font-medium">{instance.minecraftVersion}</span></p>
          <p>
            <span class="text-muted-foreground">Loader:</span>
            <span class="font-medium capitalize">{instance.loaderType}</span>
            {#if instance.loaderVersion}
              <span class="text-muted-foreground">({instance.loaderVersion})</span>
            {/if}
          </p>
          {#if instance.modpackPlatform}
            <p><span class="text-muted-foreground">Modpack:</span> <span class="font-medium">{instance.modpackPlatform}</span></p>
          {/if}
        </div>

        <!-- Update Button -->
        <Button
          variant="outline"
          class="w-full"
          onclick={openUpdateDialog}
          data-tutorial="instance-update-button"
        >
          <RefreshCw class="h-4 w-4 mr-2" />
          Check for Updates
        </Button>

        <!-- Reinstall Loader Button -->
        {#if instance.loaderType !== "vanilla" && instance.loaderVersion}
          <Button
            variant="outline"
            class="w-full"
            onclick={handleReinstallLoader}
            disabled={instancesStore.isInstallingLoader}
          >
            {#if instancesStore.isInstallingLoader}
              <Loader2 class="h-4 w-4 mr-2 animate-spin" />
              Reinstalling...
            {:else}
              <Download class="h-4 w-4 mr-2" />
              Reinstall {instance.loaderType} {instance.loaderVersion}
            {/if}
          </Button>
        {/if}
      </div>

      <!-- Quick Actions -->
      <div class="border-t border-border pt-6 space-y-4">
        <h4 class="text-sm font-medium text-muted-foreground">Quick Actions</h4>

        <Button variant="outline" class="w-full" onclick={handleOpenFolder}>
          <FolderOpen class="h-4 w-4 mr-2" />
          Open Game Folder
        </Button>

        <Button variant="outline" class="w-full" onclick={handleDuplicate}>
          <Copy class="h-4 w-4 mr-2" />
          Duplicate Instance
        </Button>

        <Button variant="outline" class="w-full" onclick={handleExport} disabled={isExporting}>
          {#if isExporting}
            <Loader2 class="h-4 w-4 mr-2 animate-spin" />
            Exporting...
          {:else}
            <Upload class="h-4 w-4 mr-2" />
            Export as .mrpack
          {/if}
        </Button>

        {#if exportError}
          <p class="text-sm text-destructive">{exportError}</p>
        {/if}
      </div>
    </div>

    {#if saveError}
      <div class="bg-destructive/10 border border-destructive text-destructive text-sm p-4 mx-6 mb-4 rounded">
        {saveError}
      </div>
    {/if}

    {#if instancesStore.loaderInstallError}
      <div class="bg-destructive/10 border border-destructive text-destructive text-sm p-4 mx-6 mb-4 rounded">
        {instancesStore.loaderInstallError}
      </div>
    {/if}

    <Sheet.Footer class="border-t border-border pt-4 px-6">
      <Button variant="outline" onclick={onClose} disabled={isSaving}>
        Cancel
      </Button>
      <Button onclick={handleSave} disabled={isSaving || !name.trim()}>
        {isSaving ? "Saving..." : "Save Changes"}
      </Button>
    </Sheet.Footer>
  </Sheet.Content>
</Sheet.Root>

<!-- Update Dialog -->
<UpdateDialog
  {instance}
  open={showUpdateDialog}
  onClose={() => { showUpdateDialog = false; }}
  onUpdated={handleInstanceUpdated}
/>
