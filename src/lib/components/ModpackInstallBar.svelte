<script lang="ts">
  import { Loader2, StopCircle } from "@lucide/svelte";
  import { ask } from "@tauri-apps/plugin-dialog";
  import { modpackInstallStore } from "$lib/stores/modpackInstall.svelte";
  import { Button } from "$lib/ui/button";

  async function handleCancel() {
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
</script>

{#if modpackInstallStore.isInstalling}
  <div
    class="fixed bottom-0 left-0 right-0 z-40 bg-card border-t-2 border-border p-4 shadow-lg"
  >
    <div class="flex items-center gap-4 max-w-4xl mx-auto">
      <!-- Progress info -->
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2 text-sm mb-1">
          <Loader2 class="h-4 w-4 animate-spin text-primary flex-shrink-0" />
          <span class="font-medium truncate">
            {#if modpackInstallStore.isCancelling}
              Cancelling...
            {:else}
              Installing {modpackInstallStore.modpackName}
            {/if}
          </span>
        </div>

        {#if modpackInstallStore.progress}
          <div class="text-xs text-muted-foreground mb-1 truncate">
            {modpackInstallStore.progress.stage}
            {#if modpackInstallStore.progress.currentItem}
              - {modpackInstallStore.progress.currentItem}
            {/if}
          </div>

          <div class="h-2 bg-muted rounded-full overflow-hidden">
            <div
              class="h-full bg-primary transition-all duration-150 ease-out"
              style="width: {Math.min(
                100,
                Math.max(0, modpackInstallStore.progress.progress)
              )}%"
            ></div>
          </div>

          {#if modpackInstallStore.progress.totalItems > 0}
            <div class="text-xs text-muted-foreground mt-1">
              {modpackInstallStore.progress.completedItems} / {modpackInstallStore
                .progress.totalItems} items
            </div>
          {/if}
        {:else}
          <div class="text-xs text-muted-foreground mb-1">Starting...</div>
          <div class="h-2 bg-muted rounded-full overflow-hidden">
            <div
              class="h-full bg-primary transition-all duration-150 ease-out animate-pulse"
              style="width: 10%"
            ></div>
          </div>
        {/if}
      </div>

      <!-- Cancel button -->
      <Button
        variant="destructive"
        size="sm"
        onclick={handleCancel}
        disabled={modpackInstallStore.isCancelling}
      >
        <StopCircle class="h-4 w-4 mr-1" />
        Cancel
      </Button>
    </div>
  </div>
{/if}
