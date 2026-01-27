<script lang="ts">
	import { Zap, Plus, Download, Package, Settings, Users } from '@lucide/svelte';
	import { Button } from '$lib/ui/button';
	import { goto } from '$app/navigation';

	async function createInstance() {
		await goto('/instances?action=create');
	}

	async function importInstance() {
		await goto('/instances?action=import');
	}

	async function browseModpacks() {
		await goto('/modpacks');
	}

	async function openSettings() {
		await goto('/settings');
	}

	async function openAccounts() {
		await goto('/accounts');
	}
</script>

<div class="space-y-2">
	<!-- Header -->
	<div class="flex items-center gap-2">
		<Zap class="text-primary h-4 w-4" />
		<h2 class="text-sm font-bold uppercase tracking-wider">Quick Actions</h2>
	</div>

	<!-- Actions Row -->
	<div class="flex gap-3">
		<Button variant="outline" class="h-auto flex-1 flex-col gap-1.5 py-3" onclick={createInstance}>
			<Plus class="h-5 w-5" />
			<span class="text-xs">Create</span>
		</Button>
		<Button variant="outline" class="h-auto flex-1 flex-col gap-1.5 py-3" onclick={importInstance}>
			<Download class="h-5 w-5" />
			<span class="text-xs">Import</span>
		</Button>
		<Button variant="outline" class="h-auto flex-1 flex-col gap-1.5 py-3" onclick={browseModpacks}>
			<Package class="h-5 w-5" />
			<span class="text-xs">Modpacks</span>
		</Button>
		<Button variant="outline" class="h-auto flex-1 flex-col gap-1.5 py-3" onclick={openSettings}>
			<Settings class="h-5 w-5" />
			<span class="text-xs">Settings</span>
		</Button>
		<Button variant="outline" class="h-auto flex-1 flex-col gap-1.5 py-3" onclick={openAccounts}>
			<Users class="h-5 w-5" />
			<span class="text-xs">Accounts</span>
		</Button>
	</div>
</div>
