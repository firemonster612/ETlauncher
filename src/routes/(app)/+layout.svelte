<script lang="ts">
  import { onMount } from "svelte";
  import AppSidebar from "$lib/components/layout/AppSidebar.svelte";
  import { SidebarInset } from "$lib/ui/sidebar";
  import { launchStore } from "$lib/stores/launch.svelte";

  let { children } = $props();

  // Initialize launch store once at app level
  onMount(() => {
    launchStore.init();
  });
</script>

<AppSidebar />
<SidebarInset>
  <main class="flex-1 overflow-auto p-6">
    {@render children()}
  </main>
</SidebarInset>
