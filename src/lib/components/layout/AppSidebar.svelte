<script lang="ts">
  import { page } from "$app/state";
  import { resolve } from "$app/paths";
  import * as Sidebar from "$lib/ui/sidebar";
  import { navItems } from "$lib/stores/navigation.svelte";
  import AccountSwitcher from "$lib/components/auth/AccountSwitcher.svelte";
</script>

<Sidebar.Root collapsible="icon" class="border-r border-sidebar-border">
  <Sidebar.Content>
    <Sidebar.Group>
      <Sidebar.GroupContent>
        <Sidebar.Menu>
          {#each navItems as item (item.id)}
            {@const isActive = page.url.pathname.startsWith(item.href)}
            <Sidebar.MenuItem>
              <Sidebar.MenuButton isActive={isActive} tooltipContent={item.label}>
                {#snippet child({ props })}
                  <a href={resolve(item.href)} {...props}>
                    <item.icon class="h-5 w-5" />
                    <span>{item.label}</span>
                  </a>
                {/snippet}
              </Sidebar.MenuButton>
            </Sidebar.MenuItem>
          {/each}
        </Sidebar.Menu>
      </Sidebar.GroupContent>
    </Sidebar.Group>
  </Sidebar.Content>

  <Sidebar.Footer>
    <AccountSwitcher />
  </Sidebar.Footer>
</Sidebar.Root>
