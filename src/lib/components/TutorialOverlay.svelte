<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { ChevronLeft, ChevronRight, X, MousePointer2, Loader2, Copy, Check, ExternalLink } from "@lucide/svelte";
  import { Button } from "$lib/ui/button";
  import { tutorialStore } from "$lib/stores/tutorial.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { accountsStore } from "$lib/stores/accounts.svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";

  // Spotlight padding around the target element
  const SPOTLIGHT_PADDING = 8;

  // Tooltip offset from spotlight
  const TOOLTIP_OFFSET = 16;

  // Tooltip sizing for edge clamping
  const TOOLTIP_MAX_WIDTH = 320;
  const TOOLTIP_MAX_HEIGHT = 240;
  const TOOLTIP_VIEWPORT_PADDING = 12;

  let tooltipEl = $state<HTMLDivElement | null>(null);
  let tooltipRect = $state({ width: TOOLTIP_MAX_WIDTH, height: TOOLTIP_MAX_HEIGHT });

  $effect(() => {
    if (tutorialStore.isActive && tooltipEl) {
      const rect = tooltipEl.getBoundingClientRect();
      tooltipRect = { width: rect.width, height: rect.height };
    }
  });

  // Resize/scroll handler
  let resizeObserver: ResizeObserver | null = null;

  // Login state
  let copiedCode = $state(false);

  onMount(() => {
    // Update position on resize
    resizeObserver = new ResizeObserver(() => {
      tutorialStore.updateTargetPosition();
    });
    resizeObserver.observe(document.body);

    // Update position on scroll
    window.addEventListener("scroll", tutorialStore.updateTargetPosition, true);
  });

  onDestroy(() => {
    resizeObserver?.disconnect();
    window.removeEventListener("scroll", tutorialStore.updateTargetPosition, true);
  });

  // Watch for auth completion during login step
  $effect(() => {
    if (tutorialStore.isLoginStep && accountsStore.accounts.length > 0 && !accountsStore.isAuthenticating) {
      // User logged in, advance to next step
      tutorialStore.next();
    }
  });

  // Compute spotlight position and size
  let spotlightStyle = $derived.by(() => {
    const rect = tutorialStore.targetRect;
    if (!rect) {
      return {
        display: "none",
      };
    }

    return {
      display: "block",
      left: `${rect.left - SPOTLIGHT_PADDING}px`,
      top: `${rect.top - SPOTLIGHT_PADDING}px`,
      width: `${rect.width + SPOTLIGHT_PADDING * 2}px`,
      height: `${rect.height + SPOTLIGHT_PADDING * 2}px`,
    };
  });

  // Compute tooltip position
  let tooltipStyle = $derived.by(() => {
    const rect = tutorialStore.targetRect;
    const step = tutorialStore.currentStep;
    if (!rect || !step) {
      return { display: "none", position: "bottom" as const };
    }

    const position = step.position;
    let left = 0;
    let top = 0;

    switch (position) {
      case "right":
        left = rect.right + SPOTLIGHT_PADDING + TOOLTIP_OFFSET;
        top = rect.top + rect.height / 2;
        break;
      case "left":
        left = rect.left - SPOTLIGHT_PADDING - TOOLTIP_OFFSET;
        top = rect.top + rect.height / 2;
        break;
      case "bottom":
        left = rect.left + rect.width / 2;
        top = rect.bottom + SPOTLIGHT_PADDING + TOOLTIP_OFFSET;
        break;
      case "top":
        left = rect.left + rect.width / 2;
        top = rect.top - SPOTLIGHT_PADDING - TOOLTIP_OFFSET;
        break;
    }

    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;
    const width = Math.min(tooltipRect.width, TOOLTIP_MAX_WIDTH);
    const height = Math.min(tooltipRect.height, TOOLTIP_MAX_HEIGHT);

    // Clamp horizontally using center-based transform for left/center/right placements
    const clampedLeft = Math.min(
      Math.max(left, TOOLTIP_VIEWPORT_PADDING + width / 2),
      viewportWidth - TOOLTIP_VIEWPORT_PADDING - width / 2
    );

    // Clamp vertically depending on transform
    let clampedTop = top;
    if (position === "bottom") {
      clampedTop = Math.min(
        Math.max(top, TOOLTIP_VIEWPORT_PADDING),
        viewportHeight - TOOLTIP_VIEWPORT_PADDING - height
      );
    } else if (position === "top") {
      // top transform anchors the tooltip's bottom to the target point
      clampedTop = Math.min(
        Math.max(top, TOOLTIP_VIEWPORT_PADDING + height),
        viewportHeight - TOOLTIP_VIEWPORT_PADDING
      );
    } else {
      // left/right center align
      clampedTop = Math.min(
        Math.max(top, TOOLTIP_VIEWPORT_PADDING + height / 2),
        viewportHeight - TOOLTIP_VIEWPORT_PADDING - height / 2
      );
    }

    return {
      left: `${clampedLeft}px`,
      top: `${clampedTop}px`,
      position,
    };
  });

  // Compute cursor position (center of spotlight with slight offset)
  let cursorStyle = $derived.by(() => {
    const rect = tutorialStore.targetRect;
    if (!rect) {
      return { display: "none" };
    }

    return {
      left: `${rect.left + rect.width / 2 + 10}px`,
      top: `${rect.top + rect.height / 2 + 10}px`,
    };
  });

  async function handleSkip() {
    accountsStore.stopAuth();
    tutorialStore.skip();
    await settingsStore.update({ setupCompleted: true });
  }

  async function handleNext() {
    if (tutorialStore.isLastStep) {
      tutorialStore.complete();
      await settingsStore.update({ setupCompleted: true });
    } else {
      tutorialStore.next();
    }
  }

  function handleBack() {
    tutorialStore.back();
  }

  // Login handlers
  async function startLogin() {
    await accountsStore.startAuth();
  }

  async function copyCode() {
    if (accountsStore.deviceCode) {
      await navigator.clipboard.writeText(accountsStore.deviceCode.userCode);
      copiedCode = true;
      setTimeout(() => (copiedCode = false), 2000);
    }
  }

  async function copyAndOpen() {
    if (accountsStore.deviceCode) {
      await navigator.clipboard.writeText(accountsStore.deviceCode.userCode);
      copiedCode = true;
      setTimeout(() => (copiedCode = false), 2000);
      await openUrl(accountsStore.deviceCode.verificationUri);
    }
  }

  function skipLogin() {
    accountsStore.stopAuth();
    tutorialStore.next();
  }
</script>

{#if tutorialStore.isActive}
  <!-- Check if this is a login step -->
  {#if tutorialStore.isLoginStep}
    <!-- Login Step UI - Centered modal -->
    <div class="fixed inset-0 z-[9998] bg-black/75 flex items-center justify-center p-4">
      <div class="bg-card border-2 border-border rounded-lg max-w-md w-full p-6 space-y-6 animate-in fade-in zoom-in-95 duration-200">
        <!-- Header -->
        <div class="flex items-center justify-between">
          <div>
            <h2 class="text-xl font-bold">{tutorialStore.currentStep?.title}</h2>
            <p class="text-sm text-muted-foreground mt-1">{tutorialStore.currentStep?.description}</p>
          </div>
          <button
            onclick={handleSkip}
            class="text-muted-foreground hover:text-foreground transition-colors"
            aria-label="Skip tutorial"
          >
            <X class="h-5 w-5" />
          </button>
        </div>

        <!-- Step indicator -->
        <div class="flex items-center justify-center gap-1.5">
          {#each Array.from({ length: tutorialStore.totalSteps }, (_, i) => i) as i (i)}
            <div
              class="w-2 h-2 rounded-full transition-all duration-300"
              class:bg-primary={i === tutorialStore.currentStepIndex}
              class:scale-125={i === tutorialStore.currentStepIndex}
              class:bg-muted={i !== tutorialStore.currentStepIndex}
            ></div>
          {/each}
        </div>

        <!-- Login Content -->
        <div class="space-y-4">
          {#if accountsStore.accounts.length > 0}
            <!-- Already logged in -->
            <div class="text-center py-4">
              <Check class="h-12 w-12 text-green-500 mx-auto mb-3" />
              <p class="font-medium">Already signed in!</p>
              <p class="text-sm text-muted-foreground mt-1">
                Welcome, {accountsStore.activeAccount?.username || accountsStore.accounts[0]?.username}
              </p>
            </div>
          {:else if !accountsStore.isAuthenticating}
            <!-- Not started yet -->
            <div class="text-center py-4">
              <p class="text-sm text-muted-foreground mb-4">
                You need a Microsoft account with Minecraft to play.
              </p>
              <Button onclick={startLogin} class="w-full">
                Sign in with Microsoft
              </Button>
            </div>
          {:else if accountsStore.deviceCode}
            <!-- Device code flow -->
            <div class="space-y-4">
              <div class="text-center">
                <p class="text-sm text-muted-foreground mb-2">Enter this code at Microsoft:</p>
                <div class="bg-muted rounded-lg p-4 font-mono text-2xl tracking-widest font-bold">
                  {accountsStore.deviceCode.userCode}
                </div>
              </div>

              <div class="flex gap-2">
                <Button variant="outline" onclick={copyCode} class="flex-1">
                  {#if copiedCode}
                    <Check class="h-4 w-4 mr-2" />
                    Copied!
                  {:else}
                    <Copy class="h-4 w-4 mr-2" />
                    Copy Code
                  {/if}
                </Button>
                <Button onclick={copyAndOpen} class="flex-1">
                  <ExternalLink class="h-4 w-4 mr-2" />
                  Open & Sign In
                </Button>
              </div>

              <div class="flex items-center justify-center gap-2 text-sm text-muted-foreground">
                <Loader2 class="h-4 w-4 animate-spin" />
                <span>Waiting for sign in...</span>
              </div>
            </div>
          {:else}
            <!-- Loading -->
            <div class="flex items-center justify-center py-8">
              <Loader2 class="h-8 w-8 animate-spin text-primary" />
            </div>
          {/if}

          {#if accountsStore.authError}
            <div class="bg-destructive/10 border border-destructive text-destructive text-sm p-3 rounded">
              {accountsStore.authError}
            </div>
          {/if}
        </div>

        <!-- Footer -->
        <div class="flex items-center justify-between pt-2 border-t border-border">
          <Button variant="ghost" size="sm" onclick={skipLogin}>
            Skip for now
          </Button>
          {#if accountsStore.accounts.length > 0}
            <Button size="sm" onclick={handleNext}>
              Continue
              <ChevronRight class="h-4 w-4 ml-1" />
            </Button>
          {/if}
        </div>
      </div>
    </div>
  {:else}
    <!-- Regular spotlight step -->
    <!-- Dark overlay backdrop -->
    <div class="fixed inset-0 z-[9998] pointer-events-none">
      <!-- Spotlight element with box-shadow effect -->
      {#if tutorialStore.targetRect}
        <div
          class="spotlight absolute rounded-lg transition-all duration-400 ease-out"
          style:left={spotlightStyle.left}
          style:top={spotlightStyle.top}
          style:width={spotlightStyle.width}
          style:height={spotlightStyle.height}
        ></div>

        <!-- Animated cursor icon -->
        <div
          class="cursor-indicator absolute z-[10001] transition-all duration-500 ease-out pointer-events-none"
          style:left={cursorStyle.left}
          style:top={cursorStyle.top}
        >
          <MousePointer2 class="h-8 w-8 text-white drop-shadow-lg animate-pulse" />
        </div>
      {:else}
        <!-- No target found - show full overlay with message -->
        <div class="fixed inset-0 bg-black/75"></div>
      {/if}
    </div>

    <!-- Tooltip and controls -->
    {#if tutorialStore.currentStep}
      <div
        class="tutorial-tooltip fixed z-[10000] pointer-events-auto"
        class:tooltip-right={tooltipStyle.position === "right"}
        class:tooltip-left={tooltipStyle.position === "left"}
        class:tooltip-bottom={tooltipStyle.position === "bottom"}
        class:tooltip-top={tooltipStyle.position === "top"}
        class:tooltip-center={!tutorialStore.targetRect}
        style:left={tutorialStore.targetRect ? tooltipStyle.left : "50%"}
        style:top={tutorialStore.targetRect ? tooltipStyle.top : "50%"}
        bind:this={tooltipEl}
      >
        <div
          class="bg-card border border-border rounded-lg shadow-xl p-4 max-w-xs animate-in fade-in slide-in-from-bottom-2 duration-300 overflow-auto"
          style:max-width={`${TOOLTIP_MAX_WIDTH}px`}
          style:max-height={`${TOOLTIP_MAX_HEIGHT}px`}
        >
          <!-- Step indicator -->
          <div class="flex items-center justify-between mb-2">
            <span class="text-xs text-muted-foreground">
              Step {tutorialStore.currentStepIndex + 1} of {tutorialStore.totalSteps}
            </span>
            <button
              onclick={handleSkip}
              class="text-muted-foreground hover:text-foreground transition-colors"
              aria-label="Skip tutorial"
            >
              <X class="h-4 w-4" />
            </button>
          </div>

          <!-- Content -->
          <h3 class="font-semibold text-foreground mb-1">
            {tutorialStore.currentStep.title}
          </h3>
          <p class="text-sm text-muted-foreground mb-4">
            {tutorialStore.currentStep.description}
          </p>

          <!-- Step dots -->
          <div class="flex items-center justify-center gap-1.5 mb-4">
            {#each Array.from({ length: tutorialStore.totalSteps }, (_, i) => i) as i (i)}
              <div
                class="w-2 h-2 rounded-full transition-all duration-300"
                class:bg-primary={i === tutorialStore.currentStepIndex}
                class:scale-125={i === tutorialStore.currentStepIndex}
                class:bg-muted={i !== tutorialStore.currentStepIndex}
              ></div>
            {/each}
          </div>

          <!-- Navigation buttons -->
          <div class="flex items-center justify-between gap-2">
            <Button
              variant="ghost"
              size="sm"
              onclick={handleBack}
              disabled={tutorialStore.isFirstStep}
              class="gap-1"
            >
              <ChevronLeft class="h-4 w-4" />
              Back
            </Button>

            <Button size="sm" onclick={handleNext} class="gap-1">
              {tutorialStore.isLastStep ? "Finish" : "Next"}
              {#if !tutorialStore.isLastStep}
                <ChevronRight class="h-4 w-4" />
              {/if}
            </Button>
          </div>
        </div>
      </div>
    {/if}
  {/if}
{/if}

<style>
  .spotlight {
    box-shadow: 0 0 0 9999px rgba(0, 0, 0, 0.75);
    pointer-events: none;
  }

  .tutorial-tooltip {
    transform-origin: center;
  }

  .tooltip-right {
    transform: translateY(-50%);
  }

  .tooltip-left {
    transform: translate(-100%, -50%);
  }

  .tooltip-bottom {
    transform: translateX(-50%);
  }

  .tooltip-top {
    transform: translate(-50%, -100%);
  }

  .tooltip-center {
    transform: translate(-50%, -50%);
  }

  /* Smooth transitions */
  .duration-400 {
    transition-duration: 400ms;
  }
</style>
