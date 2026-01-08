<script lang="ts">
  import { ChevronLeft, ChevronRight, Loader2, X } from "@lucide/svelte";
  import { Button } from "$lib/ui/button";

  interface Props {
    open: boolean;
    src: string | null;
    filename?: string;
    isLoading?: boolean;
    canPrev?: boolean;
    canNext?: boolean;
    onClose: () => void;
    onPrev?: () => void;
    onNext?: () => void;
  }

  let {
    open,
    src,
    filename,
    isLoading = false,
    canPrev = true,
    canNext = true,
    onClose,
    onPrev,
    onNext,
  }: Props = $props();

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!open) return;
    if (e.key === "Escape") onClose();
    if (e.key === "ArrowLeft" && canPrev) onPrev?.();
    if (e.key === "ArrowRight" && canNext) onNext?.();
  }

  function stopClickPropagation(e: MouseEvent) {
    e.stopPropagation();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <div
    class="fixed inset-0 bg-black/80 z-[60] flex items-center justify-center p-6"
    onclick={handleBackdropClick}
    role="dialog"
    aria-label="Screenshot viewer"
    tabindex="-1"
    onkeydown={handleKeydown}
  >
    <div
      class="relative w-full h-full max-w-6xl max-h-[90vh] flex items-center justify-center"
      onclick={stopClickPropagation}
      onkeydown={(e) => e.stopPropagation()}
      role="presentation"
      tabindex="-1"
    >
      {#if isLoading}
        <div class="flex items-center gap-3 text-muted-foreground">
          <Loader2 class="h-6 w-6 animate-spin" />
          <span>Loading image...</span>
        </div>
      {:else if src}
        <img src={src} alt={filename ?? "Screenshot"} class="max-h-full max-w-full object-contain rounded-lg shadow-2xl" />
      {:else}
        <p class="text-muted-foreground">Unable to load image.</p>
      {/if}

      <div class="absolute top-4 right-4 flex gap-2">
        <Button variant="secondary" size="icon" onclick={() => onClose()} aria-label="Close">
          <X class="h-5 w-5" />
        </Button>
      </div>

      <div class="absolute inset-y-0 left-2 flex items-center">
        <Button
          variant="secondary"
          size="icon"
          onclick={() => onPrev?.()}
          disabled={!canPrev}
          aria-label="Previous"
        >
          <ChevronLeft class="h-5 w-5" />
        </Button>
      </div>
      <div class="absolute inset-y-0 right-2 flex items-center">
        <Button
          variant="secondary"
          size="icon"
          onclick={() => onNext?.()}
          disabled={!canNext}
          aria-label="Next"
        >
          <ChevronRight class="h-5 w-5" />
        </Button>
      </div>

      {#if filename}
        <div class="absolute bottom-4 left-0 right-0 text-center text-sm text-white/80">
          {filename}
        </div>
      {/if}
    </div>
  </div>
{/if}
