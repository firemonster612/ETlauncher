<script lang="ts">
  import { Download, Loader2, RefreshCw } from "@lucide/svelte";
  import { Button } from "$lib/ui/button";
  import * as Sheet from "$lib/ui/sheet";
  import { instancesStore } from "$lib/stores/instances.svelte";
  import UpdateDialog from "./UpdateDialog.svelte";
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
  let name = $state(instance.name);
  let javaPath = $state(instance.javaPath || "");
  let memoryMin = $state(instance.memoryMinMb || 512);
  let memoryMax = $state(instance.memoryMaxMb || 4096);
  let jvmArgs = $state(instance.jvmArgs || "");
  let gameArgs = $state(instance.gameArgs || "");
  let resolutionWidth = $state(instance.resolutionWidth || 0);
  let resolutionHeight = $state(instance.resolutionHeight || 0);

  let isSaving = $state(false);
  let saveError = $state<string | null>(null);

  // Reset form when instance changes
  $effect(() => {
    name = instance.name;
    javaPath = instance.javaPath || "";
    memoryMin = instance.memoryMinMb || 512;
    memoryMax = instance.memoryMaxMb || 4096;
    jvmArgs = instance.jvmArgs || "";
    gameArgs = instance.gameArgs || "";
    resolutionWidth = instance.resolutionWidth || 0;
    resolutionHeight = instance.resolutionHeight || 0;
  });

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
      javaPath: javaPath || undefined,
      memoryMinMb: memoryMin !== 512 ? memoryMin : undefined,
      memoryMaxMb: memoryMax !== 4096 ? memoryMax : undefined,
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
        <input
          id="instance-name"
          type="text"
          bind:value={name}
          placeholder="My Instance"
          class="w-full h-10 px-3 bg-background border-2 border-border text-sm focus:border-primary outline-none"
        />
      </div>

      <!-- Java Path -->
      <div class="space-y-3">
        <label for="java-path" class="text-sm font-medium">Java Path</label>
        <p class="text-xs text-muted-foreground">Leave empty to use the default Java installation</p>
        <input
          id="java-path"
          type="text"
          bind:value={javaPath}
          placeholder="Auto-detect"
          class="w-full h-10 px-3 bg-background border-2 border-border text-sm focus:border-primary outline-none"
        />
      </div>

      <!-- Memory Allocation -->
      <div class="space-y-4">
        <div>
          <label class="text-sm font-medium">Memory Allocation</label>
          <p class="text-xs text-muted-foreground mt-1">Adjust the RAM allocated to Minecraft</p>
        </div>

        <div class="space-y-4">
          <div class="space-y-2">
            <div class="flex justify-between text-sm">
              <span>Minimum</span>
              <span class="text-primary">{formatMemory(memoryMin)}</span>
            </div>
            <input
              type="range"
              min="512"
              max="16384"
              step="512"
              bind:value={memoryMin}
              class="w-full h-2 accent-primary"
            />
          </div>

          <div class="space-y-2">
            <div class="flex justify-between text-sm">
              <span>Maximum</span>
              <span class="text-primary">{formatMemory(memoryMax)}</span>
            </div>
            <input
              type="range"
              min="512"
              max="32768"
              step="512"
              bind:value={memoryMax}
              class="w-full h-2 accent-primary"
            />
          </div>
        </div>
      </div>

      <!-- JVM Arguments -->
      <div class="space-y-3">
        <label for="jvm-args" class="text-sm font-medium">JVM Arguments</label>
        <p class="text-xs text-muted-foreground">Additional arguments passed to the Java Virtual Machine</p>
        <textarea
          id="jvm-args"
          bind:value={jvmArgs}
          placeholder="-XX:+UseG1GC"
          rows={3}
          class="w-full px-3 py-2 bg-background border-2 border-border text-sm focus:border-primary outline-none resize-none"
        ></textarea>
      </div>

      <!-- Game Arguments -->
      <div class="space-y-3">
        <label for="game-args" class="text-sm font-medium">Game Arguments</label>
        <p class="text-xs text-muted-foreground">Additional arguments passed to Minecraft</p>
        <textarea
          id="game-args"
          bind:value={gameArgs}
          placeholder="--width 1920 --height 1080"
          rows={2}
          class="w-full px-3 py-2 bg-background border-2 border-border text-sm focus:border-primary outline-none resize-none"
        ></textarea>
      </div>

      <!-- Resolution -->
      <div class="space-y-3">
        <label class="text-sm font-medium">Window Resolution</label>
        <p class="text-xs text-muted-foreground">Set to 0 for default resolution</p>
        <div class="flex gap-3 items-center">
          <input
            type="number"
            bind:value={resolutionWidth}
            placeholder="Width"
            min={0}
            class="w-24 h-10 px-3 bg-background border-2 border-border text-sm focus:border-primary outline-none"
          />
          <span class="text-muted-foreground">×</span>
          <input
            type="number"
            bind:value={resolutionHeight}
            placeholder="Height"
            min={0}
            class="w-24 h-10 px-3 bg-background border-2 border-border text-sm focus:border-primary outline-none"
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
