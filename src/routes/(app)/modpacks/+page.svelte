<script lang="ts">
  import { Package, Search } from "@lucide/svelte";
  import { Button } from "$lib/ui/button";

  let search = $state("");
</script>

<div class="space-y-6">
  <h1 class="text-2xl">Modpacks</h1>

  <div class="flex gap-2">
    <Button>All</Button>
    <Button variant="secondary">Modrinth</Button>
    <Button variant="secondary">CurseForge</Button>
    <Button variant="secondary">FTB</Button>
    <Button variant="secondary">Technic</Button>
    <Button variant="secondary">ATLauncher</Button>
  </div>

  <div class="relative w-64">
    <Search class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
    <input
      type="text"
      placeholder="Search..."
      bind:value={search}
      class="w-full h-9 pl-9 pr-3 bg-card text-sm"
    />
  </div>

  <div class="border-2 border-dashed border-border bg-card/50 p-12 text-center">
    <Package class="mx-auto h-12 w-12 text-muted-foreground/50" />
    <p class="mt-4 text-sm text-muted-foreground">Coming soon</p>
  </div>
</div>
