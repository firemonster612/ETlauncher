<script lang="ts">
	import { Zap, Plus, Download, Package } from '@lucide/svelte';
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
</script>

<div class="space-y-2">
	<!-- Header -->
	<div class="flex items-center gap-2">
		<Zap class="text-primary h-4 w-4" />
		<h2 class="text-sm font-bold uppercase tracking-wider">Quick Actions</h2>
	</div>

	<!-- Actions Grid -->
	<div class="grid grid-cols-2 gap-3">
		<Button variant="outline" class="h-auto flex-col gap-2 py-4" onclick={createInstance}>
			<Plus class="h-6 w-6" />
			<span>Create Instance</span>
		</Button>
		<Button variant="outline" class="h-auto flex-col gap-2 py-4" onclick={importInstance}>
			<Download class="h-6 w-6" />
			<span>Import Instance</span>
		</Button>
		<Button variant="outline" class="h-auto flex-col gap-2 py-4" onclick={browseModpacks}>
			<Package class="h-6 w-6" />
			<span>Browse Modpacks</span>
		</Button>
	</div>
</div>
