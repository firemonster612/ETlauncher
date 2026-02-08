<script lang="ts">
	import { page } from '$app/stores';
	import AccountSwitcher from '$lib/components/auth/AccountSwitcher.svelte';
	import { navItems } from '$lib/stores/navigation.svelte';
	import * as Sidebar from '$lib/ui/sidebar';
</script>

<Sidebar.Root collapsible="icon" class="border-sidebar-border border-r">
	<Sidebar.Content>
		<Sidebar.Group>
			<Sidebar.GroupContent>
				<Sidebar.Menu>
					{#each navItems as item (item.id)}
						{@const isActive = item.href === '/' ? $page.url.pathname === '/' : $page.url.pathname.startsWith(item.href)}
						<Sidebar.MenuItem>
							<Sidebar.MenuButton {isActive} tooltipContent={item.label}>
								{#snippet child({ props })}
									<a href={item.href} {...props}>
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
