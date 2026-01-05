<script lang="ts">
  import { Loader2, Check, AlertTriangle, HelpCircle, ArrowRight, Package, Calendar, ChevronDown } from "@lucide/svelte";
  import { Button } from "$lib/ui/button";
  import * as Sheet from "$lib/ui/sheet";
  import { Select, SelectContent, SelectItem, SelectTrigger } from "$lib/ui/select";
  import * as updateService from "$lib/services/update";
  import type {
    ModpackInstanceUpdateCheck,
    ModpackUpdatePlan,
    InstanceUpdateCheck,
    InstanceUpdatePlan,
    UpdateProgress,
    UserContentDecision,
  } from "$lib/types/update";
  import type { Instance } from "$lib/types/instance";

  interface Props {
    instance: Instance;
    open: boolean;
    onClose: () => void;
    onUpdated?: (instance: Instance) => void;
  }

  let { instance, open: isOpen, onClose, onUpdated }: Props = $props();

  // Determine mode based on instance type
  let isModpackInstance = $derived(!!instance.modpackPlatform);

  // State
  let isChecking = $state(false);
  let isUpdating = $state(false);
  let error = $state<string | null>(null);
  let updateProgress = $state<UpdateProgress | null>(null);

  // Modpack update state
  let modpackCheck = $state<ModpackInstanceUpdateCheck | null>(null);
  let selectedVersionId = $state<string | null>(null);
  let userContentDecisions = $state<Record<string, UserContentDecision>>({});
  let showAdvancedVersionSelect = $state(false);

  // Non-modpack update state
  let instanceCheck = $state<InstanceUpdateCheck | null>(null);
  let incompatibleDecisions = $state<Record<string, UserContentDecision>>({});

  // Reset state when dialog opens
  $effect(() => {
    if (isOpen) {
      error = null;
      updateProgress = null;
      modpackCheck = null;
      instanceCheck = null;
      selectedVersionId = null;
      userContentDecisions = {};
      incompatibleDecisions = {};
      showAdvancedVersionSelect = false;
      // Auto-check for updates
      handleCheckUpdates();
    }
  });

  async function handleCheckUpdates() {
    isChecking = true;
    error = null;

    try {
      if (isModpackInstance) {
        modpackCheck = await updateService.checkModpackInstanceUpdates(instance.id);
        // Initialize decisions as pending for all user-added content
        const decisions: Record<string, UserContentDecision> = {};
        for (const item of modpackCheck.userAddedContent) {
          decisions[item.filename] = "pending";
        }
        userContentDecisions = decisions;
        // Auto-select the LATEST version (first non-current version in the list)
        const latestVersion = modpackCheck.availableVersions.find(v => !v.isCurrent);
        selectedVersionId = latestVersion?.versionId || modpackCheck.currentVersion.versionId;
      } else {
        instanceCheck = await updateService.checkInstanceUpdates(instance.id);
        // Initialize decisions as pending for all incompatible content
        const decisions: Record<string, UserContentDecision> = {};
        for (const item of instanceCheck.incompatibleContent) {
          decisions[item.filename] = "pending";
        }
        incompatibleDecisions = decisions;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to check for updates";
    } finally {
      isChecking = false;
    }
  }

  async function handleExecuteUpdate() {
    isUpdating = true;
    error = null;

    try {
      if (isModpackInstance && modpackCheck && selectedVersionId) {
        const plan: ModpackUpdatePlan = {
          instanceId: instance.id,
          targetVersionId: selectedVersionId,
          userContentDecisions,
        };

        const updatedInstance = await updateService.executeModpackUpdate(
          instance.id,
          plan,
          (progress) => {
            updateProgress = progress;
          }
        );

        onUpdated?.(updatedInstance);
        onClose();
      } else if (!isModpackInstance && instanceCheck) {
        const plan: InstanceUpdatePlan = {
          instanceId: instance.id,
          targetMcVersion: instanceCheck.latestMcVersion,
          targetLoaderType: instanceCheck.currentLoaderType,
          targetLoaderVersion: instanceCheck.targetLoaderVersion,
          incompatibleDecisions,
        };

        const updatedInstance = await updateService.executeInstanceUpdate(
          instance.id,
          plan,
          (progress) => {
            updateProgress = progress;
          }
        );

        onUpdated?.(updatedInstance);
        onClose();
      }
    } catch (e) {
      error = e instanceof Error ? e.message : "Update failed";
    } finally {
      isUpdating = false;
      updateProgress = null;
    }
  }

  // Get selected version details
  let selectedVersion = $derived(
    modpackCheck?.availableVersions.find((v) => v.versionId === selectedVersionId)
  );

  // Get the latest (non-current) version
  let latestVersion = $derived(
    modpackCheck?.availableVersions.find((v) => !v.isCurrent)
  );

  // Check if all decisions are made (not pending)
  let allUserContentDecisionsMade = $derived(
    modpackCheck?.userAddedContent.length === 0 ||
    Object.values(userContentDecisions).every((d) => d !== "pending")
  );

  let allIncompatibleDecisionsMade = $derived(
    instanceCheck?.incompatibleContent.length === 0 ||
    Object.values(incompatibleDecisions).every((d) => d !== "pending")
  );

  // Check if can update
  let canUpdateModpack = $derived(
    modpackCheck &&
    selectedVersionId &&
    selectedVersionId !== modpackCheck.currentVersion.versionId &&
    allUserContentDecisionsMade
  );

  let canUpdateInstance = $derived(
    instanceCheck &&
    instanceCheck.hasMcUpdate &&
    allIncompatibleDecisionsMade
  );

  // Format date
  function formatDate(timestamp: number | undefined): string {
    if (!timestamp) return "Unknown";
    return new Date(timestamp * 1000).toLocaleDateString();
  }
</script>

<Sheet.Root bind:open={isOpen} onOpenChange={(open) => !open && onClose()}>
  <Sheet.Content side="right" class="w-full sm:max-w-xl overflow-y-auto">
    <Sheet.Header class="border-b border-border pb-4 px-6">
      <Sheet.Title>Check for Updates</Sheet.Title>
      <Sheet.Description>
        {#if isModpackInstance}
          Check for new modpack versions for <span class="font-medium">{instance.name}</span>
        {:else}
          Check for Minecraft updates for <span class="font-medium">{instance.name}</span>
        {/if}
      </Sheet.Description>
    </Sheet.Header>

    <div class="py-6 px-6 space-y-6">
      {#if isChecking}
        <div class="flex items-center justify-center py-12">
          <div class="text-center space-y-3">
            <Loader2 class="h-8 w-8 animate-spin mx-auto text-primary" />
            <p class="text-sm text-muted-foreground">Checking for updates...</p>
          </div>
        </div>
      {:else if isUpdating && updateProgress}
        <div class="space-y-4">
          <div class="text-center space-y-2">
            <Loader2 class="h-8 w-8 animate-spin mx-auto text-primary" />
            <p class="font-medium">{updateProgress.stage}</p>
            {#if updateProgress.currentItem}
              <p class="text-sm text-muted-foreground">{updateProgress.currentItem}</p>
            {/if}
          </div>
          <div class="w-full bg-muted rounded-full h-2">
            <div
              class="bg-primary h-2 rounded-full transition-all"
              style="width: {updateProgress.progress}%"
            ></div>
          </div>
          {#if updateProgress.totalItems > 0}
            <p class="text-xs text-muted-foreground text-center">
              {updateProgress.completedItems} / {updateProgress.totalItems} items
            </p>
          {/if}
        </div>

      <!-- MODPACK INSTANCE UPDATE UI -->
      {:else if isModpackInstance && modpackCheck}
        {#if modpackCheck.hasUpdate && latestVersion}
          <!-- Update Available Banner -->
          <div class="bg-primary/10 border border-primary/30 p-4 rounded space-y-3">
            <div class="flex items-center gap-2">
              <Package class="h-5 w-5 text-primary" />
              <span class="font-medium text-primary">Update Available!</span>
            </div>

            <!-- Version Change Info -->
            <div class="space-y-2 text-sm">
              <div class="flex items-center gap-2">
                <span class="text-muted-foreground w-20">Modpack:</span>
                <span class="font-medium">{modpackCheck.currentVersion.versionName}</span>
                <ArrowRight class="h-4 w-4 text-primary" />
                <span class="font-medium text-primary">{latestVersion.versionName}</span>
              </div>
              <div class="flex items-center gap-2">
                <span class="text-muted-foreground w-20">Minecraft:</span>
                <span class="font-medium">{modpackCheck.currentVersion.mcVersion}</span>
                {#if modpackCheck.currentVersion.mcVersion !== latestVersion.mcVersion}
                  <ArrowRight class="h-4 w-4 text-primary" />
                  <span class="font-medium text-primary">{latestVersion.mcVersion}</span>
                {/if}
              </div>
              <div class="flex items-center gap-2">
                <span class="text-muted-foreground w-20">Loader:</span>
                <span class="font-medium capitalize">{latestVersion.loaderType}</span>
                {#if latestVersion.loaderVersion}
                  <span class="text-muted-foreground">({latestVersion.loaderVersion})</span>
                {/if}
              </div>
              {#if latestVersion.releasedAt}
                <div class="flex items-center gap-2">
                  <span class="text-muted-foreground w-20">Released:</span>
                  <Calendar class="h-3 w-3" />
                  <span>{formatDate(latestVersion.releasedAt)}</span>
                </div>
              {/if}
            </div>

            {#if latestVersion.changelog}
              <details class="mt-2">
                <summary class="text-sm font-medium cursor-pointer text-primary">View Changelog</summary>
                <div class="mt-2 text-sm text-muted-foreground whitespace-pre-wrap max-h-48 overflow-y-auto bg-background/50 p-3 rounded">
                  {latestVersion.changelog}
                </div>
              </details>
            {/if}
          </div>

          <!-- Advanced: Select Different Version -->
          <details
            class="group"
            bind:open={showAdvancedVersionSelect}
          >
            <summary class="text-sm text-muted-foreground cursor-pointer flex items-center gap-1 hover:text-foreground">
              <ChevronDown class="h-4 w-4 transition-transform group-open:rotate-180" />
              Advanced: Select a different version
            </summary>
            <div class="mt-3 space-y-2">
              <Select
                type="single"
                value={selectedVersionId || ""}
                onValueChange={(v) => { selectedVersionId = v; }}
              >
                <SelectTrigger class="w-full">
                  {#if selectedVersion}
                    {selectedVersion.versionName} (MC {selectedVersion.mcVersion})
                  {:else}
                    Select version...
                  {/if}
                </SelectTrigger>
                <SelectContent>
                  {#each modpackCheck.availableVersions as version (version.versionId)}
                    <SelectItem value={version.versionId} disabled={version.isCurrent}>
                      <div class="flex items-center gap-2">
                        <span>{version.versionName}</span>
                        <span class="text-xs text-muted-foreground">MC {version.mcVersion}</span>
                        {#if version.isCurrent}
                          <span class="text-xs text-green-500">(Current)</span>
                        {/if}
                      </div>
                    </SelectItem>
                  {/each}
                </SelectContent>
              </Select>

              {#if selectedVersion && selectedVersion !== latestVersion && !selectedVersion.isCurrent}
                <p class="text-xs text-muted-foreground">
                  Selected: {selectedVersion.versionName} (MC {selectedVersion.mcVersion})
                </p>
              {/if}
            </div>
          </details>

          <!-- User-Added Content Decisions -->
          {#if modpackCheck.userAddedContent.length > 0}
            <div class="space-y-3">
              <div class="flex items-center gap-2">
                <AlertTriangle class="h-4 w-4 text-yellow-500" />
                <h4 class="text-sm font-medium">User-Added Content</h4>
              </div>
              <p class="text-xs text-muted-foreground">
                These mods were added by you and are not part of the modpack. Choose what to do with each:
              </p>
              <div class="space-y-3 max-h-64 overflow-y-auto">
                {#each modpackCheck.userAddedContent as item (item.filename)}
                  <div class="p-3 bg-muted/30 rounded space-y-2">
                    <div class="font-medium text-sm">{item.name || item.filename}</div>
                    <div class="flex gap-4">
                      <label class="flex items-center gap-2 cursor-pointer text-sm">
                        <input
                          type="radio"
                          name={`user-content-${item.filename}`}
                          value="keep"
                          checked={userContentDecisions[item.filename] === "keep"}
                          onchange={() => { userContentDecisions[item.filename] = "keep"; }}
                          class="w-4 h-4 accent-primary"
                        />
                        Keep
                      </label>
                      <label class="flex items-center gap-2 cursor-pointer text-sm">
                        <input
                          type="radio"
                          name={`user-content-${item.filename}`}
                          value="remove"
                          checked={userContentDecisions[item.filename] === "remove"}
                          onchange={() => { userContentDecisions[item.filename] = "remove"; }}
                          class="w-4 h-4 accent-primary"
                        />
                        Remove
                      </label>
                    </div>
                    {#if userContentDecisions[item.filename] === "pending"}
                      <p class="text-xs text-yellow-500">Please select an option</p>
                    {/if}
                  </div>
                {/each}
              </div>
            </div>
          {/if}
        {:else}
          <!-- No Update Available -->
          <div class="text-center py-6">
            <Check class="h-12 w-12 text-green-500 mx-auto mb-3" />
            <p class="font-medium">You're up to date!</p>
            <p class="text-sm text-muted-foreground mt-1">
              {modpackCheck.modpackName} is on the latest version.
            </p>
            <div class="mt-4 text-sm text-muted-foreground bg-muted/50 p-3 rounded">
              <p>Current: <span class="font-medium text-foreground">{modpackCheck.currentVersion.versionName}</span></p>
              <p>Minecraft: <span class="font-medium text-foreground">{modpackCheck.currentVersion.mcVersion}</span></p>
            </div>
          </div>
        {/if}

      <!-- NON-MODPACK INSTANCE UPDATE UI -->
      {:else if !isModpackInstance && instanceCheck}
        {#if instanceCheck.hasMcUpdate}
          <!-- Update Available Banner -->
          <div class="bg-primary/10 border border-primary/30 p-4 rounded space-y-3">
            <div class="font-medium text-primary">Update Available!</div>

            <div class="space-y-2 text-sm">
              <div class="flex items-center gap-2">
                <span class="text-muted-foreground w-20">Minecraft:</span>
                <span class="font-medium">{instanceCheck.currentMcVersion}</span>
                <ArrowRight class="h-4 w-4 text-primary" />
                <span class="font-medium text-primary">{instanceCheck.latestMcVersion}</span>
              </div>
              <div class="flex items-center gap-2">
                <span class="text-muted-foreground w-20">Loader:</span>
                <span class="font-medium capitalize">{instanceCheck.currentLoaderType}</span>
                {#if instanceCheck.currentLoaderVersion}
                  <span class="text-muted-foreground">({instanceCheck.currentLoaderVersion})</span>
                {/if}
                {#if instanceCheck.targetLoaderVersion && instanceCheck.targetLoaderVersion !== instanceCheck.currentLoaderVersion}
                  <ArrowRight class="h-4 w-4 text-primary" />
                  <span class="font-medium text-primary">{instanceCheck.targetLoaderVersion}</span>
                {/if}
              </div>
            </div>
          </div>

          <!-- Compatible Content -->
          {#if instanceCheck.compatibleContent.length > 0}
            <details class="group" open>
              <summary class="text-sm font-medium flex items-center gap-2 cursor-pointer">
                <Check class="h-4 w-4 text-green-500" />
                Compatible Content ({instanceCheck.compatibleContent.length})
              </summary>
              <p class="text-xs text-muted-foreground mt-1">
                These will be updated to versions compatible with {instanceCheck.latestMcVersion}.
              </p>
              <div class="mt-2 space-y-1 max-h-32 overflow-y-auto">
                {#each instanceCheck.compatibleContent as item (item.filename)}
                  <div class="text-sm p-2 bg-muted/30 rounded">
                    {item.name}
                  </div>
                {/each}
              </div>
            </details>
          {/if}

          <!-- Incompatible Content Decisions -->
          {#if instanceCheck.incompatibleContent.length > 0}
            <div class="space-y-3">
              <div class="flex items-center gap-2">
                <AlertTriangle class="h-4 w-4 text-yellow-500" />
                <h4 class="text-sm font-medium">Incompatible Content ({instanceCheck.incompatibleContent.length})</h4>
              </div>
              <p class="text-xs text-muted-foreground">
                These mods don't have compatible versions for {instanceCheck.latestMcVersion}. Choose what to do with each:
              </p>
              <div class="space-y-3 max-h-64 overflow-y-auto">
                {#each instanceCheck.incompatibleContent as item (item.filename)}
                  <div class="p-3 bg-muted/30 rounded space-y-2">
                    <div class="font-medium text-sm">{item.name}</div>
                    <div class="flex gap-4">
                      <label class="flex items-center gap-2 cursor-pointer text-sm">
                        <input
                          type="radio"
                          name={`incompatible-${item.filename}`}
                          value="keep"
                          checked={incompatibleDecisions[item.filename] === "keep"}
                          onchange={() => { incompatibleDecisions[item.filename] = "keep"; }}
                          class="w-4 h-4 accent-primary"
                        />
                        Keep (may cause issues)
                      </label>
                      <label class="flex items-center gap-2 cursor-pointer text-sm">
                        <input
                          type="radio"
                          name={`incompatible-${item.filename}`}
                          value="remove"
                          checked={incompatibleDecisions[item.filename] === "remove"}
                          onchange={() => { incompatibleDecisions[item.filename] = "remove"; }}
                          class="w-4 h-4 accent-primary"
                        />
                        Remove
                      </label>
                    </div>
                    {#if incompatibleDecisions[item.filename] === "pending"}
                      <p class="text-xs text-yellow-500">Please select an option</p>
                    {/if}
                  </div>
                {/each}
              </div>
            </div>
          {/if}

          <!-- Unidentified Content -->
          {#if instanceCheck.unidentifiedContent.length > 0}
            <details class="group">
              <summary class="text-sm font-medium flex items-center gap-2 cursor-pointer">
                <HelpCircle class="h-4 w-4 text-muted-foreground" />
                Unidentified Content ({instanceCheck.unidentifiedContent.length})
              </summary>
              <p class="text-xs text-muted-foreground mt-1">
                These mods couldn't be identified and will be kept as-is.
              </p>
              <div class="mt-2 space-y-1 max-h-32 overflow-y-auto">
                {#each instanceCheck.unidentifiedContent as item (item.filename)}
                  <div class="text-sm p-2 bg-muted/30 rounded text-muted-foreground">
                    {item.filename}
                  </div>
                {/each}
              </div>
            </details>
          {/if}
        {:else}
          <!-- No Update Available -->
          <div class="text-center py-6">
            <Check class="h-12 w-12 text-green-500 mx-auto mb-3" />
            <p class="font-medium">You're up to date!</p>
            <p class="text-sm text-muted-foreground mt-1">
              Already running the latest Minecraft version.
            </p>
            <div class="mt-4 text-sm text-muted-foreground bg-muted/50 p-3 rounded">
              <p>Minecraft: <span class="font-medium text-foreground">{instanceCheck.currentMcVersion}</span></p>
              <p class="capitalize">Loader: <span class="font-medium text-foreground">{instanceCheck.currentLoaderType}</span>
                {#if instanceCheck.currentLoaderVersion}
                  ({instanceCheck.currentLoaderVersion})
                {/if}
              </p>
            </div>
          </div>
        {/if}
      {/if}

      {#if error}
        <div class="bg-destructive/10 border border-destructive text-destructive text-sm p-4 rounded">
          {error}
        </div>
      {/if}
    </div>

    <Sheet.Footer class="border-t border-border pt-4 px-6">
      <Button variant="outline" onclick={onClose} disabled={isUpdating}>
        {#if (isModpackInstance && modpackCheck && !modpackCheck.hasUpdate) || (!isModpackInstance && instanceCheck && !instanceCheck.hasMcUpdate)}
          Close
        {:else}
          Cancel
        {/if}
      </Button>
      {#if isModpackInstance}
        {#if canUpdateModpack}
          <Button onclick={handleExecuteUpdate} disabled={isUpdating}>
            {#if isUpdating}
              <Loader2 class="h-4 w-4 mr-2 animate-spin" />
              Updating...
            {:else}
              Update to {selectedVersion?.versionName}
            {/if}
          </Button>
        {:else if modpackCheck?.hasUpdate && !allUserContentDecisionsMade}
          <Button disabled>
            Make all selections to continue
          </Button>
        {/if}
      {:else}
        {#if canUpdateInstance}
          <Button onclick={handleExecuteUpdate} disabled={isUpdating}>
            {#if isUpdating}
              <Loader2 class="h-4 w-4 mr-2 animate-spin" />
              Updating...
            {:else}
              Update to {instanceCheck?.latestMcVersion}
            {/if}
          </Button>
        {:else if instanceCheck?.hasMcUpdate && !allIncompatibleDecisionsMade}
          <Button disabled>
            Make all selections to continue
          </Button>
        {/if}
      {/if}
    </Sheet.Footer>
  </Sheet.Content>
</Sheet.Root>
