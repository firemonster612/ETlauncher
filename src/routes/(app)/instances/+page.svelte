<script lang="ts">
  import { onMount } from "svelte";
  import { Layers, Plus, Search, Loader2, CheckCircle, AlertTriangle, FileDown, Trash2 } from "@lucide/svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import * as modpackService from "$lib/services/modpack";
  import { Button } from "$lib/ui/button";
  import { Input } from "$lib/ui/input";
  import * as Select from "$lib/ui/select";
  import { LoaderSelect } from "$lib/ui/loader-select";
  import { instancesStore } from "$lib/stores/instances.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { versionsStore } from "$lib/stores/versions.svelte";
  import { launchStore } from "$lib/stores/launch.svelte";
  import { accountsStore } from "$lib/stores/accounts.svelte";
  import ContentBrowser from "$lib/components/ContentBrowser.svelte";
  import InstanceSettings from "$lib/components/InstanceSettings.svelte";
  import InstanceCard from "$lib/components/InstanceCard.svelte";
  import InstanceDetailModal from "$lib/components/InstanceDetailModal.svelte";
  import DownloadProgress from "$lib/components/DownloadProgress.svelte";
  import type { LoaderType, Instance, ModpackInstallProgress } from "$lib/types";

  let search = $state("");
  let showCreateModal = $state(false);
  let showDeleteModal = $state(false);
  let instanceToDelete = $state<string | null>(null);

  // Create form state
  let createName = $state("");
  let createVersion = $state("");
  let createLoader = $state<LoaderType>("vanilla");
  let createLoaderVersion = $state("");
  let isCreating = $state(false);

  // Content browser state
  let showContentBrowser = $state(false);
  let contentBrowserInstance = $state<Instance | null>(null);

  // Settings modal state
  let showSettings = $state(false);
  let settingsInstance = $state<Instance | null>(null);

  // Detail modal state
  let showDetailModal = $state(false);
  let detailInstance = $state<Instance | null>(null);

  // Export state
  let showExportModal = $state(false);
  let exportInstance = $state<Instance | null>(null);
  let isExporting = $state(false);
  let exportResult = $state<{ success: boolean; path?: string; error?: string } | null>(null);

  // Import state
  let showImportModal = $state(false);
  let importFilePath = $state("");
  let importInstanceName = $state("");
  let isImporting = $state(false);
  let importResult = $state<{ success: boolean; instance?: Instance; error?: string } | null>(null);
  let importProgress = $state<ModpackInstallProgress | null>(null);

  function openSettings(instance: Instance) {
    settingsInstance = instance;
    showSettings = true;
  }

  function closeSettings() {
    showSettings = false;
    settingsInstance = null;
  }

  function openContentBrowser(instance: Instance) {
    contentBrowserInstance = instance;
    showContentBrowser = true;
  }

  function closeContentBrowser() {
    showContentBrowser = false;
    contentBrowserInstance = null;
  }

  function openDetailModal(instance: Instance) {
    detailInstance = instance;
    showDetailModal = true;
  }

  function closeDetailModal() {
    showDetailModal = false;
    detailInstance = null;
  }

  onMount(() => {
    instancesStore.load();
    settingsStore.load();
    versionsStore.load();
    accountsStore.load();
    // launchStore is initialized at app layout level
  });

  // Set default version when versions load
  $effect(() => {
    if (versionsStore.latestRelease && !createVersion) {
      createVersion = versionsStore.latestRelease;
    }
  });

  const filteredInstances = $derived(
    instancesStore.instances.filter((instance) =>
      instance.name.toLowerCase().includes(search.toLowerCase())
    )
  );

  async function handleCreate() {
    if (!createName.trim()) return;

    isCreating = true;
    const instance = await instancesStore.create({
      name: createName.trim(),
      minecraftVersion: createVersion,
      loaderType: createLoader === "vanilla" ? undefined : createLoader,
      loaderVersion: createLoader !== "vanilla" ? createLoaderVersion : undefined,
    });

    if (instance) {
      showCreateModal = false;
      createName = "";
      createVersion = versionsStore.latestRelease ?? "";
      createLoader = "vanilla";
      createLoaderVersion = "";
    }
    isCreating = false;
  }

  function confirmDelete(instanceId: string) {
    instanceToDelete = instanceId;
    showDeleteModal = true;
  }

  async function handleDelete(deleteFiles: boolean) {
    if (instanceToDelete) {
      await instancesStore.delete(instanceToDelete, deleteFiles);
      showDeleteModal = false;
      instanceToDelete = null;
    }
  }

  async function handleLaunch(instanceId: string) {
    // Check if user is logged in
    if (!accountsStore.activeAccount) {
      alert("Please log in with a Microsoft account first. Go to Accounts and set one as active.");
      return;
    }

    console.log("Launching instance:", instanceId, "with account:", accountsStore.activeAccount.id);
    const result = await launchStore.launch(instanceId, accountsStore.activeAccount.id);
    console.log("Launch result:", result);
  }

  async function handleKill(instanceId: string) {
    console.log("Killing instance:", instanceId);
    await launchStore.kill(instanceId);
  }

  function getInstanceStatus(instanceId: string): string | null {
    const state = launchStore.launchStates.get(instanceId);
    if (!state) return null;
    return state.status.status;
  }

  function closeExportModal() {
    showExportModal = false;
    exportInstance = null;
    exportResult = null;
  }

  async function openImportModal() {
    try {
      // Open file picker dialog
      const filePath = await open({
        title: "Import .mrpack File",
        filters: [{ name: "Modrinth Pack", extensions: ["mrpack"] }],
        multiple: false,
      });

      // User cancelled
      if (!filePath) return;

      showImportModal = true;
      importFilePath = filePath as string;
      importInstanceName = "";
      importResult = null;
    } catch (e: unknown) {
      console.error("Failed to open file dialog:", e);
    }
  }

  function closeImportModal() {
    showImportModal = false;
    importFilePath = "";
    importInstanceName = "";
    importResult = null;
  }

  async function handleImport() {
    if (!importFilePath.trim()) return;

    isImporting = true;
    importResult = null;
    importProgress = null;

    let unlisten: UnlistenFn | undefined;

    try {
      // Set up progress listener before starting import
      unlisten = await listen<ModpackInstallProgress>("modpack_install_progress", (event) => {
        importProgress = event.payload;
      });

      const instance = await modpackService.importModpackFile(
        importFilePath.trim(),
        importInstanceName.trim() || undefined
      );
      importResult = { success: true, instance };
      // Refresh instance list
      await instancesStore.load();
    } catch (e: unknown) {
      importResult = { success: false, error: e instanceof Error ? e.message : "Import failed" };
    } finally {
      unlisten?.();
      isImporting = false;
      importProgress = null;
    }
  }
</script>

<div class="space-y-6">
  <div class="flex items-center justify-between gap-4">
    <h1 class="text-2xl">Instances</h1>
    <div class="flex items-center gap-4 flex-1 max-w-md">
      <div class="relative flex-1">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground z-10" />
        <Input
          type="text"
          placeholder="Search..."
          bind:value={search}
          class="pl-9"
        />
      </div>
      <Button variant="outline" onclick={openImportModal} title="Import .mrpack file">
        <FileDown class="mr-2 h-4 w-4" />
        Import
      </Button>
      <Button onclick={() => (showCreateModal = true)} data-tutorial="new-instance-btn">
        <Plus class="mr-2 h-4 w-4" />
        New
      </Button>
    </div>
  </div>

  <!-- Error Display -->
  {#if instancesStore.error}
    <div class="bg-destructive/10 border-2 border-destructive p-4 text-destructive text-sm">
      {instancesStore.error}
      <button class="underline ml-2" onclick={() => instancesStore.clearError()}>Dismiss</button>
    </div>
  {/if}

  {#if launchStore.error}
    <div class="bg-destructive/10 border-2 border-destructive p-4 text-destructive text-sm">
      Launch error: {launchStore.error}
      <button class="underline ml-2" onclick={() => launchStore.clearError()}>Dismiss</button>
    </div>
  {/if}

  {#if instancesStore.loaderInstallError}
    <div class="bg-destructive/10 border-2 border-destructive p-4 text-destructive text-sm">
      Loader installation error: {instancesStore.loaderInstallError}
      <button class="underline ml-2" onclick={() => instancesStore.clearLoaderError()}>Dismiss</button>
    </div>
  {/if}

  {#if instancesStore.isLoading}
    <div class="text-muted-foreground">Loading instances...</div>
  {:else if filteredInstances.length === 0}
    <div class="border-2 border-dashed border-border bg-card/50 p-12 text-center">
      <Layers class="mx-auto h-12 w-12 text-muted-foreground/50" />
      <p class="mt-4 text-sm text-muted-foreground">
        {search ? "No instances match your search" : "No instances yet"}
      </p>
      {#if !search}
        <Button class="mt-4" onclick={() => (showCreateModal = true)}>
          <Plus class="mr-2 h-4 w-4" />
          Create Instance
        </Button>
      {/if}
    </div>
  {:else}
    <!-- Instance Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {#each filteredInstances as instance (instance.id)}
        {@const status = getInstanceStatus(instance.id)}
        {@const launchStatus = launchStore.launchStates.get(instance.id)?.status}
        <InstanceCard
          {instance}
          {status}
          {launchStatus}
          onLaunch={handleLaunch}
          onKill={handleKill}
          onOpenSettings={openSettings}
          onOpenContentBrowser={openContentBrowser}
          onDelete={confirmDelete}
          onCardClick={openDetailModal}
        />
      {/each}
    </div>
  {/if}
</div>

<!-- Create Instance Modal -->
{#if showCreateModal}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div class="bg-card border-2 border-border p-6 max-w-md w-full mx-4 space-y-4">
      <h2 class="text-lg font-bold">Create New Instance</h2>

      <div class="space-y-4">
        <div>
          <label for="name" class="text-sm text-muted-foreground block mb-1">Instance Name</label>
          <Input
            id="name"
            type="text"
            bind:value={createName}
            placeholder="My Instance"
          />
        </div>

        <div data-tutorial="instance-version-loader" class="space-y-4">
          <div data-tutorial="instance-version">
            <span class="text-sm text-muted-foreground block mb-1">Minecraft Version</span>
            <Select.Root type="single" bind:value={createVersion} disabled={versionsStore.isLoading}>
              <Select.Trigger class="w-full border-2 border-border bg-background">
                {#if versionsStore.isLoading}
                  Loading versions...
                {:else if createVersion}
                  {createVersion}
                {:else}
                  Select version...
                {/if}
              </Select.Trigger>
              <Select.Content class="border-2 border-border bg-card max-h-[300px]">
                {#each versionsStore.versions as version (version.id)}
                  <Select.Item value={version.id} label={version.id}>
                    {version.id}
                    {#if version.type === "snapshot"}
                      <span class="text-muted-foreground ml-1">(snapshot)</span>
                    {:else if version.type === "old_beta"}
                      <span class="text-muted-foreground ml-1">(beta)</span>
                    {:else if version.type === "old_alpha"}
                      <span class="text-muted-foreground ml-1">(alpha)</span>
                    {/if}
                  </Select.Item>
                {/each}
              </Select.Content>
            </Select.Root>
          </div>

          <div data-tutorial="instance-loader">
            <LoaderSelect
              loaderType={createLoader}
              loaderVersion={createLoaderVersion}
              minecraftVersion={createVersion}
              onLoaderTypeChange={(loader) => createLoader = loader}
              onLoaderVersionChange={(version) => createLoaderVersion = version}
            />
          </div>
        </div>
      </div>

      <div class="flex gap-2 pt-2">
        <Button
          variant="outline"
          class="flex-1"
          onclick={() => (showCreateModal = false)}
          disabled={isCreating || instancesStore.isInstallingLoader}
          data-tutorial="create-cancel"
        >
          Cancel
        </Button>
        <Button
          class="flex-1"
          onclick={handleCreate}
          disabled={!createName.trim() || isCreating || instancesStore.isInstallingLoader}
          data-tutorial="instance-create"
        >
          {#if instancesStore.isInstallingLoader}
            Installing {createLoader}...
          {:else if isCreating}
            Creating...
          {:else}
            Create
          {/if}
        </Button>
      </div>

      <!-- Loader Installation Progress -->
      {#if instancesStore.isInstallingLoader && instancesStore.loaderInstallProgress}
        <div class="border-t border-border pt-4 mt-4">
          <div class="flex items-center gap-2 text-sm">
            <Loader2 class="h-4 w-4 animate-spin text-primary" />
            <span>{instancesStore.loaderInstallProgress.stage}</span>
          </div>
          <div class="mt-2 h-2 bg-muted rounded-full overflow-hidden">
            <div
              class="h-full bg-primary transition-all duration-300"
              style="width: {instancesStore.loaderInstallProgress.progress}%"
            ></div>
          </div>
          <div class="text-xs text-muted-foreground mt-1">
            {instancesStore.loaderInstallProgress.progress}% complete
          </div>
        </div>
      {/if}
    </div>
  </div>
{/if}

<!-- Delete Confirmation Modal -->
{#if showDeleteModal}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div class="bg-card border-2 border-border p-6 max-w-md w-full mx-4 space-y-4">
      <h2 class="text-lg font-bold">Delete Instance</h2>
      <p class="text-sm text-muted-foreground">
        How would you like to delete this instance?
      </p>

      <div class="flex flex-col gap-2 pt-2">
        <Button
          variant="outline"
          onclick={() => handleDelete(false)}
          class="justify-start"
        >
          Remove from launcher only
          <span class="text-xs text-muted-foreground ml-2">(keeps files)</span>
        </Button>
        <Button
          variant="destructive"
          onclick={() => handleDelete(true)}
          class="justify-start"
        >
          <Trash2 class="h-4 w-4 mr-2" />
          Delete everything
          <span class="text-xs ml-2">(permanent)</span>
        </Button>
        <Button variant="ghost" onclick={() => (showDeleteModal = false)}>
          Cancel
        </Button>
      </div>
    </div>
  </div>
{/if}

<!-- Content Browser -->
{#if showContentBrowser && contentBrowserInstance}
  <ContentBrowser
    instanceId={contentBrowserInstance.id}
    instanceName={contentBrowserInstance.name}
    mcVersion={contentBrowserInstance.minecraftVersion}
    loaderType={contentBrowserInstance.loaderType}
    onClose={closeContentBrowser}
  />
{/if}

<!-- Instance Settings -->
{#if settingsInstance}
  <InstanceSettings
    instance={settingsInstance}
    open={showSettings}
    onClose={closeSettings}
  />
{/if}

<!-- Instance Detail Modal -->
<InstanceDetailModal
  instance={detailInstance}
  open={showDetailModal}
  onClose={closeDetailModal}
/>

<!-- Export Modal -->
{#if showExportModal}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div class="bg-card border-2 border-border p-6 max-w-md w-full mx-4 space-y-4">
      <h2 class="text-lg font-bold">Export Instance</h2>

      {#if isExporting}
        <div class="flex items-center gap-3 py-4">
          <Loader2 class="h-6 w-6 animate-spin text-primary" />
          <div>
            <p class="font-medium">Exporting {exportInstance?.name}...</p>
            <p class="text-sm text-muted-foreground">Creating .mrpack file</p>
          </div>
        </div>
      {:else if exportResult?.success}
        <div class="py-4">
          <div class="flex items-center gap-3 text-green-500 mb-3">
            <CheckCircle class="h-6 w-6" />
            <p class="font-medium">Export Complete!</p>
          </div>
          <p class="text-sm text-muted-foreground">
            Saved to:
          </p>
          <p class="text-sm font-mono bg-muted p-2 mt-1 break-all">
            {exportResult.path}
          </p>
        </div>
      {:else if exportResult?.error}
        <div class="py-4">
          <div class="flex items-center gap-3 text-destructive mb-3">
            <AlertTriangle class="h-6 w-6" />
            <p class="font-medium">Export Failed</p>
          </div>
          <p class="text-sm text-muted-foreground">
            {exportResult.error}
          </p>
        </div>
      {/if}

      <div class="flex justify-end pt-2">
        <Button
          variant={exportResult?.success ? "default" : "outline"}
          onclick={closeExportModal}
          disabled={isExporting}
        >
          {exportResult?.success ? "Done" : "Close"}
        </Button>
      </div>
    </div>
  </div>
{/if}

<!-- Import Modal -->
{#if showImportModal}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div class="bg-card border-2 border-border p-6 max-w-md w-full mx-4 space-y-4">
      <h2 class="text-lg font-bold">Import .mrpack File</h2>

      {#if !importResult}
        <div class="space-y-4">
          <div>
            <p class="text-sm text-muted-foreground block mb-1">
              Selected File
            </p>
            <p class="text-sm font-mono bg-muted p-2 break-all">
              {importFilePath}
            </p>
          </div>

          <div>
            <label for="import-name" class="text-sm text-muted-foreground block mb-1">
              Instance Name (optional)
            </label>
            <Input
              id="import-name"
              type="text"
              bind:value={importInstanceName}
              placeholder="Leave empty to use modpack name"
              class="w-full"
              disabled={isImporting}
            />
          </div>
        </div>

        <div class="flex gap-2 pt-2">
          <Button
            variant="outline"
            class="flex-1"
            onclick={closeImportModal}
            disabled={isImporting}
          >
            Cancel
          </Button>
          <Button
            class="flex-1"
            onclick={handleImport}
            disabled={!importFilePath.trim() || isImporting}
          >
            {#if isImporting}
              <Loader2 class="h-4 w-4 animate-spin mr-2" />
              Importing...
            {:else}
              <FileDown class="h-4 w-4 mr-2" />
              Import
            {/if}
          </Button>
        </div>

        <!-- Import Progress -->
        {#if isImporting && importProgress}
          <div class="border-t border-border pt-4 mt-4">
            <DownloadProgress
              stage={importProgress.stage}
              progress={importProgress.progress}
              currentItem={importProgress.currentItem}
              totalItems={importProgress.totalItems}
              completedItems={importProgress.completedItems}
            />
          </div>
        {/if}
      {:else if importResult.success}
        <div class="py-4">
          <div class="flex items-center gap-3 text-green-500 mb-3">
            <CheckCircle class="h-6 w-6" />
            <p class="font-medium">Import Complete!</p>
          </div>
          <p class="text-sm text-muted-foreground">
            Instance "{importResult.instance?.name}" has been created.
          </p>
        </div>
        <div class="flex justify-end">
          <Button onclick={closeImportModal}>Done</Button>
        </div>
      {:else}
        <div class="py-4">
          <div class="flex items-center gap-3 text-destructive mb-3">
            <AlertTriangle class="h-6 w-6" />
            <p class="font-medium">Import Failed</p>
          </div>
          <p class="text-sm text-muted-foreground">
            {importResult.error}
          </p>
        </div>
        <div class="flex gap-2">
          <Button variant="outline" class="flex-1" onclick={closeImportModal}>
            Cancel
          </Button>
          <Button class="flex-1" onclick={() => importResult = null}>
            Try Again
          </Button>
        </div>
      {/if}
    </div>
  </div>
{/if}
