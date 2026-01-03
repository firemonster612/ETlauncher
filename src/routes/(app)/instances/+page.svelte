<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Layers, Plus, Search, Play, Square, Settings, Copy, Trash2, Clock, Calendar, Loader2 } from "@lucide/svelte";
  import { Button } from "$lib/ui/button";
  import * as Select from "$lib/ui/select";
  import { instancesStore } from "$lib/stores/instances.svelte";
  import { versionsStore } from "$lib/stores/versions.svelte";
  import { launchStore } from "$lib/stores/launch.svelte";
  import { accountsStore } from "$lib/stores/accounts.svelte";
  import type { LoaderType } from "$lib/types";

  let search = $state("");
  let showCreateModal = $state(false);
  let showDeleteModal = $state(false);
  let instanceToDelete = $state<string | null>(null);

  // Create form state
  let createName = $state("");
  let createVersion = $state("");
  let createLoader = $state<LoaderType>("vanilla");
  let isCreating = $state(false);

  const loaderTypes: { value: LoaderType; label: string }[] = [
    { value: "vanilla", label: "Vanilla" },
    { value: "fabric", label: "Fabric" },
    { value: "forge", label: "Forge" },
    { value: "neoforge", label: "NeoForge" },
    { value: "quilt", label: "Quilt" },
  ];

  onMount(() => {
    instancesStore.load();
    versionsStore.load();
    accountsStore.load();
    launchStore.init();
  });

  onDestroy(() => {
    launchStore.cleanup();
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
    });

    if (instance) {
      showCreateModal = false;
      createName = "";
      createVersion = "1.21.4";
      createLoader = "vanilla";
    }
    isCreating = false;
  }

  async function handleDuplicate(instanceId: string, instanceName: string) {
    await instancesStore.duplicate(instanceId, `${instanceName} (Copy)`);
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

  function getInstanceStatus(instanceId: string): string | null {
    const state = launchStore.launchStates.get(instanceId);
    if (!state) return null;
    return state.status.status;
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
</script>

<div class="space-y-6">
  <div class="flex items-center justify-between gap-4">
    <h1 class="text-2xl">Instances</h1>
    <div class="flex items-center gap-4 flex-1 max-w-md">
      <div class="relative flex-1">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
        <input
          type="text"
          placeholder="Search..."
          bind:value={search}
          class="w-full h-9 pl-9 pr-3 bg-card border-2 border-border text-sm focus:border-primary outline-none"
        />
      </div>
      <Button onclick={() => (showCreateModal = true)}>
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
        <div class="border-2 border-border bg-card p-4 hover:border-primary/50 transition-colors group">
          <div class="flex items-start justify-between gap-2">
            <div class="flex-1 min-w-0">
              <h3 class="font-bold truncate">{instance.name}</h3>
              <div class="flex items-center gap-2 mt-1">
                <span class="text-sm text-muted-foreground">{instance.minecraftVersion}</span>
                <span
                  class="text-xs px-1.5 py-0.5 border rounded capitalize {getLoaderColor(
                    instance.loaderType
                  )}"
                >
                  {instance.loaderType}
                </span>
              </div>
            </div>
            <Button
              variant="default"
              size="sm"
              class="opacity-0 group-hover:opacity-100 transition-opacity {status ? 'opacity-100' : ''}"
              onclick={() => handleLaunch(instance.id)}
              disabled={status !== null && status !== "stopped" && status !== "crashed"}
            >
              {#if status === "preparing" || status === "launching"}
                <Loader2 class="h-4 w-4 animate-spin" />
              {:else if status === "running"}
                <Square class="h-4 w-4" />
              {:else}
                <Play class="h-4 w-4" />
              {/if}
            </Button>
          </div>

          <div class="flex items-center gap-4 mt-4 text-xs text-muted-foreground">
            <span class="flex items-center gap-1">
              <Clock class="h-3 w-3" />
              {formatPlayTime(instance.totalPlayTime)}
            </span>
            <span class="flex items-center gap-1">
              <Calendar class="h-3 w-3" />
              {formatDate(instance.createdAt)}
            </span>
          </div>

          <div class="flex items-center gap-1 mt-3 pt-3 border-t border-border">
            <Button variant="ghost" size="sm" class="flex-1">
              <Settings class="h-4 w-4 mr-1" />
              Edit
            </Button>
            <Button
              variant="ghost"
              size="sm"
              class="flex-1"
              onclick={() => handleDuplicate(instance.id, instance.name)}
            >
              <Copy class="h-4 w-4 mr-1" />
              Clone
            </Button>
            <Button
              variant="ghost"
              size="sm"
              class="text-destructive hover:text-destructive hover:bg-destructive/10"
              onclick={() => confirmDelete(instance.id)}
            >
              <Trash2 class="h-4 w-4" />
            </Button>
          </div>
        </div>
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
          <input
            id="name"
            type="text"
            bind:value={createName}
            placeholder="My Instance"
            class="w-full h-9 px-3 bg-background border-2 border-border text-sm focus:border-primary outline-none"
          />
        </div>

        <div>
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

        <div>
          <span class="text-sm text-muted-foreground block mb-1">Mod Loader</span>
          <Select.Root type="single" bind:value={createLoader}>
            <Select.Trigger class="w-full border-2 border-border bg-background">
              {loaderTypes.find((l) => l.value === createLoader)?.label ?? "Vanilla"}
            </Select.Trigger>
            <Select.Content class="border-2 border-border bg-card">
              {#each loaderTypes as loader}
                <Select.Item value={loader.value} label={loader.label}>{loader.label}</Select.Item>
              {/each}
            </Select.Content>
          </Select.Root>
        </div>
      </div>

      <div class="flex gap-2 pt-2">
        <Button
          variant="outline"
          class="flex-1"
          onclick={() => (showCreateModal = false)}
          disabled={isCreating}
        >
          Cancel
        </Button>
        <Button class="flex-1" onclick={handleCreate} disabled={!createName.trim() || isCreating}>
          {isCreating ? "Creating..." : "Create"}
        </Button>
      </div>
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
