<script lang="ts">
	import { Button } from '$lib/ui/button';
	import { Trash2, Check } from '@lucide/svelte';
	import { getSkinData, skinDataToUrl, deleteSkinFromLibrary } from '$lib/services/skin';
	import { alertDialogStore } from '$lib/stores/alertDialog.svelte';
	import SkinFaceThumbnail from './SkinFaceThumbnail.svelte';
	import type { SavedSkin } from '$lib/types';

	interface Props {
		skins: SavedSkin[];
		selectedSkinId?: string;
		onSelect: (skin: SavedSkin, previewUrl: string) => void;
		onDelete: (skinId: string) => void;
	}

	let { skins, selectedSkinId, onSelect, onDelete }: Props = $props();

	// Cache skin preview URLs
	let skinUrls = $state<Record<string, string>>({});

	// Load skin preview URLs
	async function loadSkinUrl(skin: SavedSkin) {
		if (skinUrls[skin.id]) return;

		try {
			const data = await getSkinData(skin.id);
			const url = skinDataToUrl(data);
			skinUrls = { ...skinUrls, [skin.id]: url };
		} catch (err) {
			console.error('Failed to load skin preview:', err);
		}
	}

	// Load all skin URLs when component mounts or skins change
	$effect(() => {
		skins.forEach(loadSkinUrl);
	});

	async function selectSkin(skin: SavedSkin) {
		let url = skinUrls[skin.id];
		if (!url) {
			const data = await getSkinData(skin.id);
			url = skinDataToUrl(data);
			skinUrls = { ...skinUrls, [skin.id]: url };
		}
		onSelect(skin, url);
	}

	async function confirmDelete(skin: SavedSkin, event: Event) {
		event.stopPropagation();

		const confirmed = await alertDialogStore.confirm({
			title: 'Delete Skin',
			message: `Are you sure you want to delete "${skin.name}" from your library?`,
			type: 'warning',
			confirmText: 'Delete',
			cancelText: 'Cancel',
		});

		if (confirmed) {
			try {
				await deleteSkinFromLibrary(skin.id);
				// Clean up URL
				if (skinUrls[skin.id]) {
					URL.revokeObjectURL(skinUrls[skin.id]);
					const newUrls = { ...skinUrls };
					delete newUrls[skin.id];
					skinUrls = newUrls;
				}
				onDelete(skin.id);
			} catch (err) {
				console.error('Failed to delete skin:', err);
			}
		}
	}
</script>

<div class="space-y-2">
	<h3 class="text-sm font-medium">Skin Library</h3>

	{#if skins.length === 0}
		<div class="text-muted-foreground border-border border-2 border-dashed p-4 text-center text-sm">
			No saved skins. Import a skin to get started.
		</div>
	{:else}
		<div class="grid grid-cols-4 gap-2">
			{#each skins as skin (skin.id)}
				{@const isSelected = selectedSkinId === skin.id}
				{@const previewUrl = skinUrls[skin.id]}
				<button
					type="button"
					class="group relative border-2 p-1 transition-colors {isSelected
						? 'border-primary bg-primary/10'
						: 'border-border hover:border-primary/50'}"
					onclick={() => selectSkin(skin)}
					title="{skin.name} ({skin.variant})"
				>
					{#if previewUrl}
						<SkinFaceThumbnail
							url={previewUrl}
							alt={skin.name}
							class="aspect-square w-full"
						/>
					{:else}
						<div class="bg-muted aspect-square w-full animate-pulse"></div>
					{/if}

					{#if isSelected}
						<div class="bg-primary absolute -top-1 -right-1 rounded-full p-0.5">
							<Check class="text-primary-foreground h-3 w-3" />
						</div>
					{/if}

					<!-- Delete button -->
					<Button
						variant="destructive"
						size="icon"
						class="absolute -bottom-1 -right-1 h-5 w-5 opacity-0 group-hover:opacity-100"
						onclick={(e) => confirmDelete(skin, e)}
					>
						<Trash2 class="h-3 w-3" />
					</Button>

					<!-- Skin name tooltip on hover -->
					<div
						class="bg-background/90 pointer-events-none absolute inset-x-0 bottom-0 truncate px-1 py-0.5 text-center text-xs opacity-0 transition-opacity group-hover:opacity-100"
					>
						{skin.name}
					</div>
				</button>
			{/each}
		</div>
	{/if}
</div>
