<script lang="ts">
  import "../app.css";
  import { SidebarProvider } from "$lib/ui/sidebar";
  import Titlebar from "$lib/components/layout/Titlebar.svelte";

  let { children } = $props();
</script>

<div class="flex h-screen flex-col overflow-hidden">
  <Titlebar />
  <SidebarProvider>
    <div class="flex flex-1 overflow-hidden">
      {@render children()}
    </div>
  </SidebarProvider>
</div>
