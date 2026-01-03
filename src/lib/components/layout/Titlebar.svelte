<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { Minus, Square, X } from "@lucide/svelte";

  const appWindow = getCurrentWindow();

  function minimize() {
    appWindow.minimize();
  }

  function toggleMaximize() {
    appWindow.toggleMaximize();
  }

  function close() {
    appWindow.close();
  }

  function startDrag(e: MouseEvent) {
    // Only start drag on left click and not on buttons
    if (e.button === 0) {
      appWindow.startDragging();
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="flex h-9 items-center justify-between border-b-2 border-border bg-sidebar select-none"
  onmousedown={startDrag}
>
  <div class="px-4">
    <span class="text-xs font-bold tracking-widest uppercase">ETLauncher</span>
  </div>

  <div class="flex items-center">
    <button
      onmousedown={(e) => e.stopPropagation()}
      onclick={minimize}
      class="flex h-9 w-10 items-center justify-center text-muted-foreground transition-all hover:bg-primary/20 hover:text-primary"
      aria-label="Minimize"
    >
      <Minus class="h-4 w-4" strokeWidth={3} />
    </button>
    <button
      onmousedown={(e) => e.stopPropagation()}
      onclick={toggleMaximize}
      class="flex h-9 w-10 items-center justify-center text-muted-foreground transition-all hover:bg-primary/20 hover:text-primary"
      aria-label="Maximize"
    >
      <Square class="h-3.5 w-3.5" strokeWidth={3} />
    </button>
    <button
      onmousedown={(e) => e.stopPropagation()}
      onclick={close}
      class="flex h-9 w-10 items-center justify-center text-muted-foreground transition-all hover:bg-destructive hover:text-white"
      aria-label="Close"
    >
      <X class="h-4 w-4" strokeWidth={3} />
    </button>
  </div>
</div>
