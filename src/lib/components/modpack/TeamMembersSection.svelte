<script lang="ts">
	import { User } from '@lucide/svelte';
	import type { ModpackTeamMember } from '$lib/types';

	interface Props {
		members: ModpackTeamMember[];
		authorFallback?: string;
	}

	let { members, authorFallback = 'Unknown' }: Props = $props();

	function capitalizeRole(role: string): string {
		return role.charAt(0).toUpperCase() + role.slice(1).toLowerCase();
	}
</script>

{#if members.length > 0}
	<div class="space-y-2">
		{#each members as member (member.username)}
			<div class="bg-muted/50 border-border flex items-center gap-3 border p-2">
				{#if member.avatarUrl}
					<img src={member.avatarUrl} alt="" class="h-8 w-8 rounded-full" />
				{:else}
					<div class="bg-card flex h-8 w-8 items-center justify-center rounded-full">
						<User class="text-muted-foreground h-4 w-4" />
					</div>
				{/if}
				<div class="flex min-w-0 flex-col">
					<span class="truncate text-sm font-medium">{member.name || member.username}</span>
					<span class="text-muted-foreground text-xs capitalize">{capitalizeRole(member.role)}</span
					>
				</div>
			</div>
		{/each}
	</div>
{:else}
	<div class="text-sm">{authorFallback}</div>
{/if}
