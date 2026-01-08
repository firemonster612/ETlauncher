<script lang="ts">
  import { X } from "@lucide/svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { Button } from "$lib/ui/button";

  interface Props {
    open: boolean;
    title: string;
    html: string;
    onClose: () => void;
  }

  let { open, title, html, onClose }: Props = $props();

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) onClose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!open) return;
    if (e.key === "Escape") onClose();
  }

  function stopClickPropagation(e: MouseEvent) {
    e.stopPropagation();
  }

  async function handleLinkClick(e: MouseEvent) {
    const target = e.target as HTMLElement | null;
    const anchor = target?.closest("a") as HTMLAnchorElement | null;
    if (!anchor) return;

    const href = anchor.getAttribute("href");
    if (!href || href.startsWith("#")) return;

    e.preventDefault();
    e.stopPropagation();
    try {
      await openUrl(href);
    } catch (err) {
      console.error("Failed to open URL:", href, err);
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <div
    class="fixed inset-0 bg-black/70 z-[60] flex items-center justify-center p-6"
    onclick={handleBackdropClick}
    role="dialog"
    aria-label={title}
    tabindex="-1"
    onkeydown={handleKeydown}
  >
    <div
      class="relative w-full max-w-5xl max-h-[90vh] bg-card border-2 border-border rounded-lg shadow-2xl overflow-hidden flex flex-col"
      onclick={stopClickPropagation}
      role="presentation"
      tabindex="-1"
    >
      <div class="flex items-center justify-between gap-3 px-5 py-4 border-b border-border">
        <h3 class="font-semibold truncate">{title}</h3>
        <Button variant="secondary" size="icon" onclick={onClose} aria-label="Close">
          <X class="h-5 w-5" />
        </Button>
      </div>

      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="flex-1 min-h-0 overflow-y-auto p-5" onclick={handleLinkClick}>
        <div class="text-sm leading-relaxed [&_p]:mb-3 [&_ul]:list-disc [&_ul]:pl-5 [&_ol]:list-decimal [&_ol]:pl-5 [&_img]:max-w-full [&_img]:rounded-md [&_img]:my-2 [&_h1]:text-lg [&_h2]:text-base [&_h1]:font-semibold [&_h2]:font-semibold [&_a]:text-primary [&_a]:underline">
          <!-- eslint-disable-next-line svelte/no-at-html-tags -->
          {@html html}
        </div>
      </div>
    </div>
  </div>
{/if}
