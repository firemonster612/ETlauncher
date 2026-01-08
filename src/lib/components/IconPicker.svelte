<script lang="ts">
  import { Search, Check } from "@lucide/svelte";
  import { Input } from "$lib/ui/input";
  import {
    ENTITY_ICONS,
    ICON_CATEGORIES,
    getIconsByCategory,
    searchIcons,
    getIconUrl,
    type EntityIcon,
    type IconCategory,
  } from "$lib/utils/icons";

  interface Props {
    selected: string | undefined;
    onSelect: (icon: EntityIcon) => void;
  }

  let { selected, onSelect }: Props = $props();

  let searchQuery = $state("");
  let activeCategory = $state<IconCategory>("Passive");

  const displayedIcons = $derived(() => {
    if (searchQuery.trim()) {
      return searchIcons(searchQuery);
    }
    return getIconsByCategory(activeCategory);
  });

  function isSelected(icon: EntityIcon): boolean {
    return selected === `entity:${icon.id}`;
  }
</script>

<div class="space-y-4">
  <!-- Search -->
  <div class="relative">
    <Search class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground z-10" />
    <Input
      type="text"
      placeholder="Search icons..."
      bind:value={searchQuery}
      class="pl-9"
    />
  </div>

  <!-- Category tabs -->
  {#if !searchQuery.trim()}
    <div class="flex gap-1 flex-wrap">
      {#each ICON_CATEGORIES as category (category)}
        <button
          type="button"
          class="px-3 py-1.5 text-xs border-2 transition-colors {activeCategory === category
            ? 'border-primary bg-primary/10 text-primary'
            : 'border-border hover:border-primary/50'}"
          onclick={() => (activeCategory = category)}
        >
          {category}
        </button>
      {/each}
    </div>
  {/if}

  <!-- Icon grid -->
  <div class="h-48 overflow-y-auto border-2 border-border p-2" style="overscroll-behavior: contain;">
    {#if displayedIcons().length === 0}
      <div class="h-full flex items-center justify-center text-muted-foreground text-sm">
        No icons found
      </div>
    {:else}
      <div class="flex flex-wrap gap-1">
        {#each displayedIcons() as icon (icon.id)}
          <button
            type="button"
            class="relative border-2 transition-colors hover:border-primary/50 p-1 {isSelected(icon)
              ? 'border-primary bg-primary/10'
              : 'border-transparent'}"
            onclick={() => onSelect(icon)}
            title={icon.name}
            style="line-height: 0;"
          >
            <img
              src={getIconUrl(icon)}
              alt={icon.name}
              class="w-8 h-8 pixelated"
            />
            {#if isSelected(icon)}
              <div class="absolute -top-1 -right-1 bg-primary rounded-full p-0.5">
                <Check class="h-2.5 w-2.5 text-primary-foreground" />
              </div>
            {/if}
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Selected icon display -->
  {#if selected}
    {@const selectedIcon = ENTITY_ICONS.find(i => `entity:${i.id}` === selected)}
    {#if selectedIcon}
      <div class="flex items-center gap-2 text-sm text-muted-foreground">
        <span>Selected:</span>
        <img src={getIconUrl(selectedIcon)} alt={selectedIcon.name} class="w-5 h-5 pixelated" />
        <span class="font-medium text-foreground">{selectedIcon.name}</span>
      </div>
    {/if}
  {/if}
</div>
