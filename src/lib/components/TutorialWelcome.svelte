<script lang="ts">
  import { Gamepad2, ArrowRight, X } from "@lucide/svelte";
  import { Button } from "$lib/ui/button";
  import { tutorialStore } from "$lib/stores/tutorial.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";

  interface Props {
    open: boolean;
  }

  let { open }: Props = $props();

  function handleStart() {
    tutorialStore.start();
  }

  async function handleSkip() {
    tutorialStore.hideWelcome();
    await settingsStore.update({ setupCompleted: true });
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) handleSkip();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!open) return;
    if (e.key === "Escape") handleSkip();
  }

  function stopClickPropagation(e: MouseEvent) {
    e.stopPropagation();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <div
    class="fixed left-0 right-0 bottom-0 top-[var(--titlebar-height)] bg-black/50 z-50 flex items-center justify-center p-6"
    onclick={handleBackdropClick}
    role="dialog"
    aria-label="Welcome to ETLauncher"
  >
    <div
      class="relative w-full max-w-lg max-h-[calc(100vh-var(--titlebar-height)-4rem)] bg-card border-2 border-border rounded-lg shadow-2xl overflow-hidden flex flex-col"
      onclick={stopClickPropagation}
      role="presentation"
      tabindex="-1"
    >
      <div class="flex items-center justify-between px-6 py-4 border-b border-border">
        <h3 class="text-2xl font-semibold">Welcome to ETLauncher</h3>
        <Button variant="secondary" size="icon" onclick={handleSkip} aria-label="Close">
          <X class="h-5 w-5" />
        </Button>
      </div>

      <div class="py-6 px-6 space-y-6 overflow-y-auto">
        <!-- App Icon/Logo -->
        <div class="w-20 h-20 bg-primary/10 rounded-2xl flex items-center justify-center mx-auto">
          <Gamepad2 class="h-10 w-10 text-primary" />
        </div>

        <p class="text-base text-center">
          Your all-in-one Minecraft launcher for managing instances, modpacks, and more.
        </p>

        <!-- What you'll learn section -->
        <div class="space-y-3">
          <h4 class="text-sm font-medium text-foreground">What you'll learn:</h4>
          <ul class="space-y-2 text-sm text-muted-foreground">
            <li class="flex items-start gap-2">
              <div class="w-5 h-5 rounded-full bg-primary/10 flex items-center justify-center flex-shrink-0 mt-0.5">
                <span class="text-xs text-primary font-medium">1</span>
              </div>
              <span>How to sign in with your Microsoft account</span>
            </li>
            <li class="flex items-start gap-2">
              <div class="w-5 h-5 rounded-full bg-primary/10 flex items-center justify-center flex-shrink-0 mt-0.5">
                <span class="text-xs text-primary font-medium">2</span>
              </div>
              <span>How to browse and install modpacks</span>
            </li>
            <li class="flex items-start gap-2">
              <div class="w-5 h-5 rounded-full bg-primary/10 flex items-center justify-center flex-shrink-0 mt-0.5">
                <span class="text-xs text-primary font-medium">3</span>
              </div>
              <span>How to create custom instances from scratch</span>
            </li>
            <li class="flex items-start gap-2">
              <div class="w-5 h-5 rounded-full bg-primary/10 flex items-center justify-center flex-shrink-0 mt-0.5">
                <span class="text-xs text-primary font-medium">4</span>
              </div>
              <span>How to add mods, shaders, and resource packs</span>
            </li>
            <li class="flex items-start gap-2">
              <div class="w-5 h-5 rounded-full bg-primary/10 flex items-center justify-center flex-shrink-0 mt-0.5">
                <span class="text-xs text-primary font-medium">5</span>
              </div>
              <span>How to update Minecraft versions and mod loaders</span>
            </li>
            <li class="flex items-start gap-2">
              <div class="w-5 h-5 rounded-full bg-primary/10 flex items-center justify-center flex-shrink-0 mt-0.5">
                <span class="text-xs text-primary font-medium">6</span>
              </div>
              <span>How to launch and manage your Minecraft installations</span>
            </li>
          </ul>
        </div>

        <p class="text-sm text-muted-foreground">
          This quick tutorial will walk you through both ways to play: installing ready-made modpacks
          and creating your own custom instances.
        </p>
      </div>

      <div class="border-t border-border pt-4 px-6 flex flex-col sm:flex-row gap-2">
        <Button variant="ghost" onclick={handleSkip} class="w-full sm:w-auto">
          Skip for now
        </Button>
        <Button onclick={handleStart} class="w-full sm:w-auto gap-2">
          Start Tutorial
          <ArrowRight class="h-4 w-4" />
        </Button>
      </div>
    </div>
  </div>
{/if}
