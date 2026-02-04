<script lang="ts">
	import { Button } from '$lib/ui/button';
	import { Input } from '$lib/ui/input';
	import { X, Search, Trash2, Check } from '@lucide/svelte';
	import { getSkinData, skinDataToUrl, deleteSkinFromLibrary } from '$lib/services/skin';
	import { alertDialogStore } from '$lib/stores/alertDialog.svelte';
	import SkinViewer3D from './SkinViewer3D.svelte';
	import type { SavedSkin } from '$lib/types';

	interface Props {
		skins: SavedSkin[];
		selectedSkinId: string | null;
		onSelect: (skin: SavedSkin, data: Uint8Array, previewUrl: string) => void;
		onDelete: (skinId: string) => void;
		onClose: () => void;
	}

	let { skins, selectedSkinId, onSelect, onDelete, onClose }: Props = $props();

	let searchQuery = $state('');
	let skinUrls = $state<Record<string, string>>({});

	// Filter skins based on search
	const filteredSkins = $derived(
		searchQuery.trim()
			? skins.filter(s => s.name.toLowerCase().includes(searchQuery.toLowerCase()))
			: skins
	);

	// Load skin preview URLs
	$effect(() => {
		for (const skin of skins) {
			if (!skinUrls[skin.id]) {
				loadSkinUrl(skin);
			}
		}
	});

	async function loadSkinUrl(skin: SavedSkin) {
		try {
			const data = await getSkinData(skin.id);
			const url = skinDataToUrl(data);
			skinUrls = { ...skinUrls, [skin.id]: url };
		} catch (err) {
			console.error('Failed to load skin preview:', err);
		}
	}

	async function handleSelect(skin: SavedSkin) {
		try {
			const data = await getSkinData(skin.id);
			const url = skinUrls[skin.id] || skinDataToUrl(data);
			onSelect(skin, data, url);
			onClose();
		} catch (err) {
			console.error('Failed to select skin:', err);
		}
	}

	async function handleDelete(skin: SavedSkin, event: Event) {
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

	function handleBackdropClick(event: MouseEvent) {
		if (event.target === event.currentTarget) {
			onClose();
		}
	}
</script>

<!-- Fullscreen modal backdrop -->
<div 
	class="fixed inset-0 z-[60] flex items-center justify-center bg-black/70"
	onclick={handleBackdropClick}
	onkeydown={(e) => e.key === 'Escape' && onClose()}
	role="button"
	tabindex="-1"
>
	<div class="bg-card border-border m-4 flex max-h-[85vh] w-full max-w-4xl flex-col border-2">
		<!-- Header -->
		<div class="border-border flex items-center justify-between border-b-2 p-4">
			<h2 class="text-lg font-bold">Skin Library</h2>
			<Button variant="ghost" size="icon" onclick={onClose}>
				<X class="h-4 w-4" />
			</Button>
		</div>

		<!-- Search bar -->
		<div class="border-border border-b-2 p-4">
			<div class="relative">
				<Search class="text-muted-foreground absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2" />
				<Input
					bind:value={searchQuery}
					placeholder="Search skins..."
					class="pl-10"
				/>
			</div>
		</div>

		<!-- Skin grid -->
		<div class="flex-1 overflow-y-auto p-4">
			{#if filteredSkins.length === 0}
				<div class="text-muted-foreground flex h-32 items-center justify-center">
					{#if searchQuery}
						No skins found matching "{searchQuery}"
					{:else}
						No skins in library
					{/if}
				</div>
			{:else}
				<div class="grid grid-cols-4 gap-4 sm:grid-cols-5 md:grid-cols-6 lg:grid-cols-7">
					{#each filteredSkins as skin (skin.id)}
						{@const isSelected = selectedSkinId === skin.id}
						{@const previewUrl = skinUrls[skin.id]}
						<button
							type="button"
							class="group relative flex flex-col items-center border-2 p-2 transition-colors {isSelected
								? 'border-primary bg-primary/10'
								: 'border-border hover:border-primary/50'}"
							onclick={() => handleSelect(skin)}
							title="{skin.name} ({skin.variant})"
						>
							{#if previewUrl}
								<div class="pointer-events-none">
									<SkinViewer3D
										skinUrl={previewUrl}
										slim={skin.variant === 'slim'}
										width={60}
										height={90}
										animation="none"
									/>
								</div>
							{:else}
								<div class="bg-muted h-[90px] w-[60px] animate-pulse"></div>
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
								onclick={(e) => handleDelete(skin, e)}
							>
								<Trash2 class="h-3 w-3" />
							</Button>
							
							<span class="mt-2 w-full truncate text-center text-xs">{skin.name}</span>
							<span class="text-muted-foreground text-[10px]">{skin.variant}</span>
						</button>
					{/each}
				</div>
			{/if}
		</div>

		<!-- Footer with count -->
		<div class="border-border text-muted-foreground border-t-2 px-4 py-2 text-sm">
			{filteredSkins.length} of {skins.length} skin{skins.length !== 1 ? 's' : ''}
		</div>
	</div>
</div>
