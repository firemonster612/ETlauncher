<script lang="ts">
	import { open } from '@tauri-apps/plugin-dialog';
	import { Button } from '$lib/ui/button';
	import { Input } from '$lib/ui/input';
	import { Upload, Loader2, X } from '@lucide/svelte';
	import { validateSkinImage, skinDataToUrl, saveSkinToLibrary, readSkinFile } from '$lib/services/skin';
	import SkinViewer3D from './SkinViewer3D.svelte';
	import type { SavedSkin } from '$lib/types';

	interface Props {
		onSkinImported: (skin: SavedSkin, previewUrl: string) => void;
		onPreviewSkin?: (data: Uint8Array, variant: 'classic' | 'slim') => void;
	}

	let { onSkinImported, onPreviewSkin }: Props = $props();

	interface PendingSkin {
		id: string;
		name: string;
		variant: 'classic' | 'slim';
		data: Uint8Array;
		previewUrl: string;
	}

	let isLoading = $state(false);
	let isSaving = $state(false);
	let error = $state<string | null>(null);
	let pendingSkins = $state<PendingSkin[]>([]);

	// Preview the first pending skin
	$effect(() => {
		if (pendingSkins.length > 0 && onPreviewSkin) {
			const first = pendingSkins[0];
			onPreviewSkin(first.data, first.variant);
		}
	});

	async function selectFiles() {
		try {
			error = null;

			const selected = await open({
				multiple: true,
				filters: [{ name: 'PNG Images', extensions: ['png'] }],
			});

			if (!selected) return;

			// Handle both single file (string) and multiple files (array)
			const files = Array.isArray(selected) ? selected : [selected];
			if (files.length === 0) return;

			isLoading = true;

			const newPendingSkins: PendingSkin[] = [];
			const errors: string[] = [];

			for (const filePath of files) {
				try {
					// Read file data via backend
					const skinData = await readSkinFile(filePath);

					// Create image to validate dimensions
					const url = skinDataToUrl(skinData);
					
					const validation = await new Promise<{ valid: boolean; error?: string; width: number; height: number }>((resolve) => {
						const img = new Image();
						img.onload = () => {
							const result = validateSkinImage(img.width, img.height);
							resolve({ ...result, width: img.width, height: img.height });
						};
						img.onerror = () => {
							resolve({ valid: false, error: 'Failed to load image', width: 0, height: 0 });
						};
						img.src = url;
					});

					if (!validation.valid) {
						const filename = filePath.split(/[/\\]/).pop() || 'Unknown';
						errors.push(`${filename}: ${validation.error}`);
						URL.revokeObjectURL(url);
						continue;
					}

					// Generate default name from filename
					const filename = filePath.split(/[/\\]/).pop() || 'Skin';
					const name = filename.replace(/\.png$/i, '');

					newPendingSkins.push({
						id: crypto.randomUUID(),
						name,
						variant: 'classic',
						data: skinData,
						previewUrl: url,
					});
				} catch (err) {
					const filename = filePath.split(/[/\\]/).pop() || 'Unknown';
					errors.push(`${filename}: ${err instanceof Error ? err.message : 'Failed to read'}`);
				}
			}

			if (newPendingSkins.length > 0) {
				pendingSkins = [...pendingSkins, ...newPendingSkins];
			}

			if (errors.length > 0) {
				error = errors.join('\n');
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to read files';
		} finally {
			isLoading = false;
		}
	}

	function removePendingSkin(id: string) {
		const skin = pendingSkins.find(s => s.id === id);
		if (skin) {
			URL.revokeObjectURL(skin.previewUrl);
		}
		pendingSkins = pendingSkins.filter(s => s.id !== id);
	}

	function updateSkinName(id: string, name: string) {
		pendingSkins = pendingSkins.map(s => s.id === id ? { ...s, name } : s);
	}

	function updateSkinVariant(id: string, variant: 'classic' | 'slim') {
		pendingSkins = pendingSkins.map(s => s.id === id ? { ...s, variant } : s);
		
		// Update preview if this is the first skin
		const updated = pendingSkins.find(s => s.id === id);
		if (updated && pendingSkins[0]?.id === id && onPreviewSkin) {
			onPreviewSkin(updated.data, variant);
		}
	}

	async function saveAllSkins() {
		if (pendingSkins.length === 0) return;

		const skinsToSave = pendingSkins.filter(s => s.name.trim());
		if (skinsToSave.length === 0) {
			error = 'Please enter a name for at least one skin';
			return;
		}

		try {
			isSaving = true;
			error = null;

			for (const skin of skinsToSave) {
				const savedSkin = await saveSkinToLibrary(skin.name.trim(), skin.variant, skin.data);
				onSkinImported(savedSkin, skin.previewUrl);
			}

			// Clean up remaining preview URLs
			for (const skin of pendingSkins) {
				if (!skinsToSave.includes(skin)) {
					URL.revokeObjectURL(skin.previewUrl);
				}
			}

			pendingSkins = [];
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to save skins';
		} finally {
			isSaving = false;
		}
	}

	function cancelImport() {
		for (const skin of pendingSkins) {
			URL.revokeObjectURL(skin.previewUrl);
		}
		pendingSkins = [];
		error = null;
	}
</script>

<div class="space-y-4">
	{#if error}
		<div class="bg-destructive/10 border-destructive text-destructive border-2 p-3 text-sm whitespace-pre-line">
			{error}
		</div>
	{/if}

	{#if pendingSkins.length > 0}
		<!-- Pending skins list -->
		<div class="space-y-3">
			<div class="flex items-center justify-between">
				<span class="text-sm font-medium">{pendingSkins.length} skin{pendingSkins.length > 1 ? 's' : ''} ready to import</span>
				<Button variant="outline" size="sm" onclick={selectFiles} disabled={isLoading || isSaving}>
					<Upload class="mr-1 h-3 w-3" />
					Add More
				</Button>
			</div>

			<div class="max-h-[200px] space-y-2 overflow-y-auto">
				{#each pendingSkins as skin (skin.id)}
					<div class="border-border flex items-center gap-2 border-2 p-2">
						<!-- Thumbnail -->
						<div class="pointer-events-none border border-border">
							<SkinViewer3D
								skinUrl={skin.previewUrl}
								slim={skin.variant === 'slim'}
								width={40}
								height={56}
								animation="none"
							/>
						</div>
						
						<!-- Name input -->
						<Input 
							value={skin.name}
							oninput={(e) => updateSkinName(skin.id, e.currentTarget.value)}
							placeholder="Skin name..."
							class="flex-1 h-8 text-sm"
						/>
						
						<!-- Variant toggle -->
						<div class="flex">
							<button
								type="button"
								class="border-2 border-r-0 px-2 py-1 text-xs {skin.variant === 'classic'
									? 'border-primary bg-primary/10 text-primary'
									: 'border-border hover:border-primary/50'}"
								onclick={() => updateSkinVariant(skin.id, 'classic')}
								title="Classic (4px arms)"
							>
								4px
							</button>
							<button
								type="button"
								class="border-2 px-2 py-1 text-xs {skin.variant === 'slim'
									? 'border-primary bg-primary/10 text-primary'
									: 'border-border hover:border-primary/50'}"
								onclick={() => updateSkinVariant(skin.id, 'slim')}
								title="Slim (3px arms)"
							>
								3px
							</button>
						</div>
						
						<!-- Remove button -->
						<Button 
							variant="ghost" 
							size="icon-sm"
							onclick={() => removePendingSkin(skin.id)}
							disabled={isSaving}
						>
							<X class="h-4 w-4" />
						</Button>
					</div>
				{/each}
			</div>

			<div class="flex gap-2">
				<Button variant="outline" class="flex-1" onclick={cancelImport} disabled={isSaving}>
					Cancel
				</Button>
				<Button
					class="flex-1"
					onclick={saveAllSkins}
					disabled={isSaving || pendingSkins.every(s => !s.name.trim())}
				>
					{#if isSaving}
						<Loader2 class="mr-2 h-4 w-4 animate-spin" />
						Saving...
					{:else}
						Save {pendingSkins.length > 1 ? `All ${pendingSkins.length}` : ''} to Library
					{/if}
				</Button>
			</div>
		</div>
	{:else}
		<!-- Select files button -->
		<Button variant="outline" class="w-full" onclick={selectFiles} disabled={isLoading}>
			{#if isLoading}
				<Loader2 class="mr-2 h-4 w-4 animate-spin" />
				Loading...
			{:else}
				<Upload class="mr-2 h-4 w-4" />
				Import Skins (PNG)
			{/if}
		</Button>
		<p class="text-muted-foreground text-center text-xs">
			Supports 64x64 or 64x32 PNG images
		</p>
	{/if}
</div>
