<script lang="ts">
	import { Ban, Check, FolderOpen, Loader2, Upload, X } from '@lucide/svelte';
	import { open } from '@tauri-apps/plugin-dialog';
	import { onMount } from 'svelte';
	
	import { getMinecraftProfile, hideCape, setCape, uploadSkin } from '$lib/services/account';
	import { getDefaultSkin, getOfflineCapeData, getOfflineSkinData, removeOfflineCape, setOfflineCape, setOfflineSkin } from '$lib/services/auth';
	import { applySkinFromLibrary, getSkinData, getSkinLibrary, readSkinFile, saveSkinToLibrary, skinDataToUrl } from '$lib/services/skin';
	import type { CapeInfo, MinecraftAccount, MinecraftProfile, SavedSkin } from '$lib/types';
	import { Button } from '$lib/ui/button';
	import CapeSelector from './CapeSelector.svelte';
	import CapeThumbnail from './CapeThumbnail.svelte';
	import SkinLibraryModal from './SkinLibraryModal.svelte';
	import SkinUploader from './SkinUploader.svelte';
	import SkinViewer3D from './SkinViewer3D.svelte';

	// Default skin URLs (official Minecraft texture URLs from Mojang, used for online accounts)
	const STEVE_SKIN_URL = 'https://textures.minecraft.net/texture/1a4af718455d4aab528e7a61f86fa25e6a369d1768dcb13f7df319a713eb810b';
	const ALEX_SKIN_URL = 'https://textures.minecraft.net/texture/83cee5ca6afcdb171285aa00e8049c297b2dbeba0efb8ff970a5677a1b644032';

	// Local blob URLs for bundled default skins (used for offline accounts)
	let steveSkinUrl = $state<string>(STEVE_SKIN_URL);
	let alexSkinUrl = $state<string>(ALEX_SKIN_URL);

	type DefaultSkinType = 'steve' | 'alex';

	interface Props {
		account: MinecraftAccount;
		onClose: () => void;
		onProfileUpdated?: (profile: MinecraftProfile) => void;
		onOfflineSkinUpdated?: () => void;
	}

	let { account, onClose, onProfileUpdated, onOfflineSkinUpdated }: Props = $props();

	const isOffline = $derived(account.accountType === 'offline');

	// State
	let isLoading = $state(true);
	let isSaving = $state(false);
	let error = $state<string | null>(null);
	let showLibraryModal = $state(false);

	// Profile data
	let profile = $state<MinecraftProfile | null>(null);
	let savedSkins = $state<SavedSkin[]>([]);

	// Cache for library skin preview URLs
	let skinUrls = $state<Record<string, string>>({});

	// Current selection - will be populated from profile data in loadData
	let currentSkinUrl = $state<string | undefined>(undefined);
	let currentCapeUrl = $state<string | undefined>(undefined);
	let currentSlim = $state(false);

	// Selected items (for preview before applying)
	let selectedSkinId = $state<string | null>(null); // For library skins
	let selectedDefaultSkin = $state<DefaultSkinType | null>(null); // For default skins (current/steve/alex)
	let selectedSkinData = $state<Uint8Array | null>(null);
	let selectedSkinVariant = $state<'classic' | 'slim'>('classic');
	let selectedCapeId = $state<string | null | undefined>(undefined); // undefined = not changed, null = no cape
	let previewSkinUrl = $state<string | undefined>(undefined);
	// Use empty string to explicitly mean "no cape", undefined means "not changed"
	let previewCapeUrl = $state<string | undefined>(undefined);

	// Offline cape upload state
	let offlineCapeData = $state<Uint8Array | null>(null); // New cape data to upload
	let offlineCapeRemoved = $state(false); // Whether the user removed the cape

	// Computed preview URLs
	const displaySkinUrl = $derived(previewSkinUrl || currentSkinUrl);
	// Empty string means explicitly no cape, undefined means use current
	const displayCapeUrl = $derived(
		previewCapeUrl !== undefined
			? (previewCapeUrl === '' ? undefined : previewCapeUrl)
			: currentCapeUrl
	);
	const displaySlim = $derived(
		selectedDefaultSkin === 'alex' ? true :
		selectedDefaultSkin === 'steve' ? false :
		selectedSkinData ? selectedSkinVariant === 'slim' : currentSlim
	);

	// Check if there are pending changes
	const hasChanges = $derived(
		selectedSkinData !== null || selectedDefaultSkin !== null || selectedCapeId !== undefined ||
		offlineCapeData !== null || offlineCapeRemoved
	);

	onMount(async () => {
		await loadData();
	});

	async function loadData() {
		isLoading = true;
		error = null;

		try {
			if (isOffline) {
				// Offline account: load skin + cape from local storage + skin library + bundled defaults
				const [skinDataUrl, capeDataUrl, skinsData, steveData, alexData] = await Promise.all([
					getOfflineSkinData(account.id),
					getOfflineCapeData(account.id),
					getSkinLibrary(),
					getDefaultSkin('steve'),
					getDefaultSkin('alex'),
				]);

				savedSkins = skinsData;

				// Create blob URLs from the bundled default skins (works offline)
				steveSkinUrl = skinDataToUrl(steveData);
				alexSkinUrl = skinDataToUrl(alexData);

				if (skinDataUrl) {
					currentSkinUrl = skinDataUrl;
				}
				if (capeDataUrl) {
					currentCapeUrl = capeDataUrl;
				}
				currentSlim = account.offlineSkinVariant === 'slim';
			} else {
				// Online account: load Mojang profile + skin library
				const [profileData, skinsData] = await Promise.all([
					getMinecraftProfile(account.id),
					getSkinLibrary(),
				]);

				profile = profileData;
				savedSkins = skinsData;

				// Get current skin variant
				const activeSkin = profile.skins.find((s) => s.state === 'ACTIVE');
				if (activeSkin) {
					currentSkinUrl = activeSkin.url;
					currentSlim = activeSkin.variant === 'slim';
				}

				// Get active cape
				const activeCape = profile.capes.find((c) => c.state === 'ACTIVE');
				if (activeCape) {
					currentCapeUrl = activeCape.url;
				} else {
					currentCapeUrl = undefined;
				}

				// Auto-save current skin to library if the library is empty (first-time user experience)
				if (savedSkins.length === 0 && activeSkin?.url) {
					try {
						const skinData = await fetchSkinFromUrl(activeSkin.url);
						const variant = activeSkin.variant === 'slim' ? 'slim' : 'classic';
						const skin = await saveSkinToLibrary('Current Skin', variant, skinData);
						savedSkins = [skin];
					} catch (err) {
						console.error('Failed to auto-save current skin to library:', err);
					}
				}
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load profile data';
		} finally {
			isLoading = false;
		}
	}

	function handleSkinImported(skin: SavedSkin, previewUrl: string) {
		// Add to saved skins list
		savedSkins = [skin, ...savedSkins];

		// Select the newly imported skin (clear default selection)
		selectedSkinId = skin.id;
		selectedDefaultSkin = null;
		selectedSkinVariant = skin.variant as 'classic' | 'slim';
		previewSkinUrl = previewUrl;

		// Load the skin data for uploading
		getSkinData(skin.id).then((data) => {
			selectedSkinData = data;
		});
	}

	function handlePreviewSkin(data: Uint8Array, variant: 'classic' | 'slim') {
		selectedSkinData = data;
		selectedSkinVariant = variant;
		previewSkinUrl = skinDataToUrl(data);
	}

	async function handleSelectDefaultSkin(type: DefaultSkinType) {
		selectedDefaultSkin = type;
		selectedSkinId = null; // Clear library selection
		selectedSkinData = null; // Will be loaded when applying

		// Set preview URL and variant based on type
		if (type === 'steve') {
			previewSkinUrl = steveSkinUrl;
			selectedSkinVariant = 'classic';
		} else if (type === 'alex') {
			previewSkinUrl = alexSkinUrl;
			selectedSkinVariant = 'slim';
		}
	}

	// Load skin preview URL for library skins
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

	// Load all skin URLs when savedSkins changes
	$effect(() => {
		savedSkins.forEach(loadSkinUrl);
	});

	async function handleSelectLibrarySkin(skin: SavedSkin) {
		selectedSkinId = skin.id;
		selectedDefaultSkin = null; // Clear default selection
		selectedSkinVariant = skin.variant as 'classic' | 'slim';

		// Use cached URL or load it
		let url = skinUrls[skin.id];
		if (!url) {
			const data = await getSkinData(skin.id);
			url = skinDataToUrl(data);
			skinUrls = { ...skinUrls, [skin.id]: url };
		}
		previewSkinUrl = url;

		try {
			selectedSkinData = await getSkinData(skin.id);
		} catch (err) {
			console.error('Failed to load skin data:', err);
		}
	}

	function handleLibraryModalSelect(skin: SavedSkin, data: Uint8Array, url: string) {
		selectedSkinId = skin.id;
		selectedDefaultSkin = null;
		selectedSkinVariant = skin.variant as 'classic' | 'slim';
		selectedSkinData = data;
		previewSkinUrl = url;
		skinUrls = { ...skinUrls, [skin.id]: url };
	}

	function handleLibraryModalDelete(skinId: string) {
		savedSkins = savedSkins.filter(s => s.id !== skinId);
		if (selectedSkinId === skinId) {
			selectedSkinId = null;
			selectedSkinData = null;
			previewSkinUrl = undefined;
		}
	}

	async function fetchSkinFromUrl(url: string): Promise<Uint8Array> {
		const response = await fetch(url);
		if (!response.ok) {
			throw new Error(`Failed to fetch skin: ${response.statusText}`);
		}
		const buffer = await response.arrayBuffer();
		return new Uint8Array(buffer);
	}

	async function handleOfflineCapeUpload() {
		try {
			const selected = await open({
				multiple: false,
				filters: [{ name: 'PNG Images', extensions: ['png'] }],
			});
			if (!selected) return;

			const filePath = Array.isArray(selected) ? selected[0] : selected;
			const data = await readSkinFile(filePath);

			offlineCapeData = data;
			offlineCapeRemoved = false;
			previewCapeUrl = skinDataToUrl(data);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to read cape file';
		}
	}

	function handleOfflineCapeRemove() {
		offlineCapeData = null;
		offlineCapeRemoved = true;
		previewCapeUrl = ''; // Empty string = explicitly no cape
	}

	function handleSelectCape(cape: CapeInfo | null) {
		if (cape === null) {
			selectedCapeId = null;
			previewCapeUrl = ''; // Empty string = explicitly no cape
		} else {
			selectedCapeId = cape.id;
			previewCapeUrl = cape.url;
		}
	}

	async function applyChanges() {
		if (!hasChanges) return;

		isSaving = true;
		error = null;

		try {
			if (isOffline) {
				// Offline account: save skin locally
				let skinData = selectedSkinData;

				if (selectedDefaultSkin && !skinData) {
					// Load the bundled default skin (no network needed)
					skinData = await getDefaultSkin(selectedDefaultSkin);
				}

				if (skinData) {
					await setOfflineSkin(account.id, skinData, selectedSkinVariant);
				}

				// Apply offline cape changes
				if (offlineCapeData) {
					await setOfflineCape(account.id, offlineCapeData);
				} else if (offlineCapeRemoved) {
					await removeOfflineCape(account.id);
				}

				onOfflineSkinUpdated?.();
			} else {
				// Online account: upload to Mojang
				let updatedProfile: MinecraftProfile | null = null;

				// Apply skin change
				if (selectedDefaultSkin) {
					let skinUrl: string;
					let variant: 'classic' | 'slim';

					if (selectedDefaultSkin === 'steve') {
						skinUrl = STEVE_SKIN_URL;
						variant = 'classic';
					} else {
						skinUrl = ALEX_SKIN_URL;
						variant = 'slim';
					}

					const skinData = await fetchSkinFromUrl(skinUrl);
					updatedProfile = await uploadSkin(account.id, variant, skinData);
				} else if (selectedSkinData) {
					if (selectedSkinId) {
						updatedProfile = await applySkinFromLibrary(account.id, selectedSkinId);
					} else {
						updatedProfile = await uploadSkin(account.id, selectedSkinVariant, selectedSkinData);
					}
				}

				// Apply cape change
				if (selectedCapeId !== undefined) {
					if (selectedCapeId === null) {
						updatedProfile = await hideCape(account.id);
					} else {
						updatedProfile = await setCape(account.id, selectedCapeId);
					}
				}

				if (updatedProfile) {
					onProfileUpdated?.(updatedProfile);
				}
			}

			onClose();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to apply changes';
		} finally {
			isSaving = false;
		}
	}

	function cancel() {
		// Clean up any preview URLs
		if (previewSkinUrl && previewSkinUrl.startsWith('blob:')) {
			URL.revokeObjectURL(previewSkinUrl);
		}
		onClose();
	}

	// Get the active cape ID for the selector
	const activeCapeId = $derived(() => {
		if (selectedCapeId !== undefined) return selectedCapeId;
		const activeCape = profile?.capes.find((c) => c.state === 'ACTIVE');
		return activeCape?.id || null;
	});
</script>

<!-- Modal backdrop -->
<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
	<div class="bg-card border-border mx-4 flex max-h-[90vh] w-full max-w-3xl flex-col overflow-hidden border-2">
		<!-- Header -->
		<div class="border-border flex items-center justify-between border-b-2 p-4">
			<h2 class="text-lg font-bold">Manage Skin & Cape</h2>
			<Button variant="ghost" size="icon" onclick={cancel}>
				<X class="h-4 w-4" />
			</Button>
		</div>

		<!-- Content -->
		<div class="flex-1 overflow-y-auto overflow-x-hidden p-4">
			{#if isLoading}
				<div class="flex h-64 items-center justify-center">
					<Loader2 class="text-muted-foreground h-8 w-8 animate-spin" />
				</div>
			{:else if error}
				<div class="bg-destructive/10 border-destructive text-destructive border-2 p-4 text-sm">
					{error}
					<button class="ml-2 underline" onclick={loadData}>Retry</button>
				</div>
			{:else}
				<div class="flex gap-6">
					<!-- 3D Preview -->
					<div class="flex flex-col items-center gap-2">
						<div class="border-border bg-background border-2">
							<SkinViewer3D
								skinUrl={displaySkinUrl}
								capeUrl={displayCapeUrl || undefined}
								slim={displaySlim}
								width={200}
								height={300}
								animation="idle"
							/>
						</div>
						<p class="text-muted-foreground text-xs">Drag to rotate</p>
					</div>

					<!-- Right panel -->
					<div class="min-w-0 flex-1 space-y-6 overflow-y-auto overflow-x-hidden">
						<!-- Default Skins -->
						<div class="space-y-2">
							<h3 class="text-sm font-medium">Default Skins</h3>
							<div class="grid grid-cols-2 gap-3">
								<!-- Steve -->
								<button
									type="button"
									class="group relative flex flex-col items-center border-2 p-1 transition-colors {selectedDefaultSkin === 'steve'
										? 'border-primary bg-primary/10'
										: 'border-border hover:border-primary/50'}"
									onclick={() => handleSelectDefaultSkin('steve')}
									title="Steve (Classic)"
								>
									<div class="pointer-events-none">
										<SkinViewer3D
											skinUrl={steveSkinUrl}
											slim={false}
											width={70}
											height={100}
											animation="none"
										/>
									</div>
									{#if selectedDefaultSkin === 'steve'}
										<div class="bg-primary absolute -top-1 -right-1 rounded-full p-0.5">
											<Check class="text-primary-foreground h-3 w-3" />
										</div>
									{/if}
									<span class="mt-1 text-xs">Steve</span>
								</button>

								<!-- Alex -->
								<button
									type="button"
									class="group relative flex flex-col items-center border-2 p-1 transition-colors {selectedDefaultSkin === 'alex'
										? 'border-primary bg-primary/10'
										: 'border-border hover:border-primary/50'}"
									onclick={() => handleSelectDefaultSkin('alex')}
									title="Alex (Slim)"
								>
									<div class="pointer-events-none">
										<SkinViewer3D
											skinUrl={alexSkinUrl}
											slim={true}
											width={70}
											height={100}
											animation="none"
										/>
									</div>
									{#if selectedDefaultSkin === 'alex'}
										<div class="bg-primary absolute -top-1 -right-1 rounded-full p-0.5">
											<Check class="text-primary-foreground h-3 w-3" />
										</div>
									{/if}
									<span class="mt-1 text-xs">Alex</span>
								</button>
							</div>
						</div>

						<!-- Skin Library -->
						<div class="space-y-2">
							<div class="flex items-center justify-between">
								<h3 class="text-sm font-medium">Skin Library</h3>
								{#if savedSkins.length > 0}
									<span class="text-muted-foreground text-xs">{savedSkins.length} skin{savedSkins.length !== 1 ? 's' : ''}</span>
								{/if}
							</div>
							
							{#if savedSkins.length === 0}
								<p class="text-muted-foreground py-2 text-center text-sm">
									No saved skins. Import a skin below to add it to your library.
								</p>
							{:else}
								<!-- Show up to 4 recent skins as quick picks -->
								<div class="grid grid-cols-4 gap-2">
									{#each savedSkins.slice(0, 4) as skin (skin.id)}
										{@const isSelected = selectedSkinId === skin.id && !selectedDefaultSkin}
										{@const previewUrl = skinUrls[skin.id]}
										<button
											type="button"
											class="group relative flex flex-col items-center border-2 p-1 transition-colors {isSelected
												? 'border-primary bg-primary/10'
												: 'border-border hover:border-primary/50'}"
											onclick={() => handleSelectLibrarySkin(skin)}
											title="{skin.name} ({skin.variant})"
										>
											{#if previewUrl}
												<div class="pointer-events-none">
													<SkinViewer3D
														skinUrl={previewUrl}
														slim={skin.variant === 'slim'}
														width={50}
														height={70}
														animation="none"
													/>
												</div>
											{:else}
												<div class="bg-muted h-[70px] w-[50px] animate-pulse"></div>
											{/if}
											{#if isSelected}
												<div class="bg-primary absolute -top-1 -right-1 rounded-full p-0.5">
													<Check class="text-primary-foreground h-3 w-3" />
												</div>
											{/if}
											<span class="mt-1 w-full truncate text-center text-[10px]">{skin.name}</span>
										</button>
									{/each}
								</div>
								
								<!-- Browse Library button -->
								<Button 
									variant="outline" 
									class="w-full" 
									onclick={() => showLibraryModal = true}
								>
									<FolderOpen class="mr-2 h-4 w-4" />
									Browse Library ({savedSkins.length})
								</Button>
							{/if}
						</div>

						<!-- Import new skin -->
						<SkinUploader
							onSkinImported={handleSkinImported}
							onPreviewSkin={handlePreviewSkin}
						/>

						<!-- Cape -->
						{#if isOffline}
							<!-- Offline cape upload -->
							<div class="space-y-2">
								<h3 class="text-sm font-medium">Cape</h3>
								<div class="flex flex-wrap gap-2">
									<!-- No Cape option -->
									<button
										type="button"
										class="group relative flex flex-col items-center border-2 p-2 transition-colors {offlineCapeRemoved || (!currentCapeUrl && !offlineCapeData)
											? 'border-primary bg-primary/10'
											: 'border-border hover:border-primary/50'}"
										onclick={handleOfflineCapeRemove}
										title="No Cape"
									>
										<div class="bg-muted flex h-12 w-8 items-center justify-center">
											<Ban class="text-muted-foreground h-4 w-4" />
										</div>
										<span class="mt-1 text-xs">None</span>
										{#if offlineCapeRemoved || (!currentCapeUrl && !offlineCapeData)}
											<div class="bg-primary absolute -top-1 -right-1 rounded-full p-0.5">
												<Check class="text-primary-foreground h-3 w-3" />
											</div>
										{/if}
									</button>

									<!-- Current cape thumbnail (if exists and not removed) -->
									{#if currentCapeUrl && !offlineCapeRemoved && !offlineCapeData}
										<button
											type="button"
											class="group relative flex flex-col items-center border-2 border-primary bg-primary/10 p-2 transition-colors"
											title="Current Cape"
										>
											<CapeThumbnail
												url={currentCapeUrl}
												alt="Current Cape"
												class="h-12 w-8"
											/>
											<span class="mt-1 text-xs">Current</span>
											<div class="bg-primary absolute -top-1 -right-1 rounded-full p-0.5">
												<Check class="text-primary-foreground h-3 w-3" />
											</div>
										</button>
									{/if}

									<!-- Uploaded cape preview (pending) -->
									{#if offlineCapeData && previewCapeUrl}
										<button
											type="button"
											class="group relative flex flex-col items-center border-2 border-primary bg-primary/10 p-2 transition-colors"
											title="New Cape"
										>
											<CapeThumbnail
												url={previewCapeUrl}
												alt="New Cape"
												class="h-12 w-8"
											/>
											<span class="mt-1 text-xs">New</span>
											<div class="bg-primary absolute -top-1 -right-1 rounded-full p-0.5">
												<Check class="text-primary-foreground h-3 w-3" />
											</div>
										</button>
									{/if}
								</div>

								<Button variant="outline" class="w-full" onclick={handleOfflineCapeUpload}>
									<Upload class="mr-2 h-4 w-4" />
									Upload Cape (PNG)
								</Button>
							</div>
						{:else if profile}
							<!-- Online cape selector -->
							<CapeSelector
								capes={profile.capes}
								selectedCapeId={activeCapeId()}
								onSelect={handleSelectCape}
							/>
						{/if}
					</div>
				</div>
			{/if}
		</div>

		<!-- Footer -->
		<div class="border-border flex justify-end gap-2 border-t-2 p-4">
			<Button variant="outline" onclick={cancel} disabled={isSaving}>
				Cancel
			</Button>
			<Button onclick={applyChanges} disabled={!hasChanges || isSaving} class="min-w-[120px]">
				{#if isSaving}
					<Loader2 class="mr-2 h-4 w-4 animate-spin" />
					Saving...
				{:else}
					Apply Changes
				{/if}
			</Button>
		</div>
	</div>
</div>

<!-- Skin Library Modal -->
{#if showLibraryModal}
	<SkinLibraryModal
		skins={savedSkins}
		selectedSkinId={selectedSkinId}
		onSelect={handleLibraryModalSelect}
		onDelete={handleLibraryModalDelete}
		onClose={() => showLibraryModal = false}
	/>
{/if}
