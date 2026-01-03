<script lang="ts">
  import AppSidebar from "$lib/components/layout/AppSidebar.svelte";
  import { SidebarInset } from "$lib/ui/sidebar";

  let { children } = $props();
</script>

<AppSidebar />
<SidebarInset>
  <main class="flex-1 overflow-auto p-6">
    {@render children()}
  </main>
</SidebarInset>
