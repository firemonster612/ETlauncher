<script lang="ts">
	import {
		Search,
		Plus,
		Loader2,
		Trash2,
		Package,
		Sparkles,
		Image,
		RefreshCw,
		PowerOff,
		Power,
		Square,
		SquareCheck,
		Minus,
		Link,
		FileJson,
		Globe,
	} from '@lucide/svelte';
	import { Button } from '$lib/ui/button';
	import { SvelteSet } from 'svelte/reactivity';
	import { nestedScroll } from '$lib/utils/scroll';
	import { Input } from '$lib/ui/input';
	import * as contentService from '$lib/services/content';
	import type { Instance, ContentType, ScanResult, DetectedMod } from '$lib/types';

	interface Props {
		instance: Instance;
		onOpenContentBrowser: (contentType: ContentType) => void;
	}

	let { instance, onOpenContentBrowser }: Props = $props();

	// Tab state for content types
	let activeContentType = $state<ContentType>('mod');

	// Scan results for each content type
	let modScanResult = $state<ScanResult | null>(null);
	let shaderScanResult = $state<ScanResult | null>(null);
	let resourcepackScanResult = $state<ScanResult | null>(null);
	let datapackScanResult = $state<ScanResult | null>(null);
	let worldScanResult = $state<ScanResult | null>(null);

	// Loading states
	let isLoading = $state(false);
	let isBulkActioning = $state(false);
	let error = $state<string | null>(null);

	// Search
	let search = $state('');

	// Selection state
	let selectedItems = new SvelteSet<string>();

	// Remove confirmation
	let showRemoveConfirm = $state(false);

	// Dependents popover state
	let showDependentsFor = $state<string | null>(null);

	// Load content on mount and when instance changes
	let lastLoadedInstanceId = $state<string | null>(null);

	$effect(() => {
		if (instance.id !== lastLoadedInstanceId) {
			lastLoadedInstanceId = instance.id;
			loadAllContent();
		}
	});

	// Clear selection when changing content type
	$effect(() => {
		void activeContentType;
		selectedItems = new SvelteSet<string>();
	});

	async function loadAllContent() {
		isLoading = true;
		error = null;

		try {
			const [mods, shaders, resourcepacks, datapacks, worlds] = await Promise.all([
				contentService.scanInstalledContent(instance.id, 'mod'),
				contentService.scanInstalledContent(instance.id, 'shader'),
				contentService.scanInstalledContent(instance.id, 'resourcepack'),
				contentService.scanInstalledContent(instance.id, 'datapack'),
				contentService.scanInstalledContent(instance.id, 'world'),
			]);
			modScanResult = mods;
			shaderScanResult = shaders;
			resourcepackScanResult = resourcepacks;
			datapackScanResult = datapacks;
			worldScanResult = worlds;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to scan content';
			console.error('Failed to scan content:', e);
		} finally {
			isLoading = false;
		}
	}

	async function handleRefresh() {
		selectedItems = new SvelteSet<string>();
		await loadAllContent();
	}

	// Get current scan result based on active tab
	const scanResultsByType = $derived({
		mod: modScanResult,
		shader: shaderScanResult,
		resourcepack: resourcepackScanResult,
		datapack: datapackScanResult,
		world: worldScanResult,
	});
	const currentScanResult = $derived(scanResultsByType[activeContentType]);

	const installedItems = $derived(currentScanResult?.items ?? []);

	// Filter items based on search
	const visibleItems = $derived(
		installedItems.filter((item) => {
			const query = search.trim().toLowerCase();
			if (!query) return true;
			const displayName = getItemDisplayName(item).toLowerCase();
			const filename = item.filename.toLowerCase();
			return displayName.includes(query) || filename.includes(query);
		})
	);

	// Selection helpers
	const selectedEnabledItems = $derived(
		installedItems.filter((item) => selectedItems.has(item.filename) && !item.isDisabled)
	);

	const selectedDisabledItems = $derived(
		installedItems.filter((item) => selectedItems.has(item.filename) && item.isDisabled)
	);

	const allSelected = $derived(
		visibleItems.length > 0 && visibleItems.every((item) => selectedItems.has(item.filename))
	);

	const someSelected = $derived(
		visibleItems.some((item) => selectedItems.has(item.filename)) && !allSelected
	);

	// Find items that other mods depend on (have dependents)
	const selectedItemsWithDependents = $derived(
		installedItems.filter(
			(item) => selectedItems.has(item.filename) && item.dependencyOf.length > 0
		)
	);

	// Counts for tabs
	const modCount = $derived(modScanResult?.items.length ?? 0);
	const shaderCount = $derived(shaderScanResult?.items.length ?? 0);
	const resourcepackCount = $derived(resourcepackScanResult?.items.length ?? 0);
	const datapackCount = $derived(datapackScanResult?.items.length ?? 0);
	const worldCount = $derived(worldScanResult?.items.length ?? 0);

	function getItemDisplayName(item: DetectedMod): string {
		if (item.modrinthProject?.name) {
			return item.modrinthProject.name;
		}
		if (item.curseforgeProject?.name) {
			return item.curseforgeProject.name;
		}
		// Fall back to filename without extension
		return item.filename.replace(/\.(jar|zip)$/i, '');
	}

	function getItemVersion(item: DetectedMod): string | null {
		if (item.modrinthProject?.versionNumber) {
			return item.modrinthProject.versionNumber;
		}
		return null;
	}

	function formatBytes(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
	}

	function toggleItemSelection(filename: string) {
		if (selectedItems.has(filename)) {
			selectedItems.delete(filename);
		} else {
			selectedItems.add(filename);
		}
		// Trigger reactivity
		selectedItems = new SvelteSet(selectedItems);
	}

	function toggleSelectAll() {
		if (visibleItems.length === 0) return;

		if (allSelected) {
			for (const item of visibleItems) {
				selectedItems.delete(item.filename);
			}
		} else {
			for (const item of visibleItems) {
				selectedItems.add(item.filename);
			}
		}
		// Trigger reactivity
		selectedItems = new SvelteSet(selectedItems);
	}

	async function handleDisableSelected() {
		if (selectedEnabledItems.length === 0) return;

		isBulkActioning = true;
		error = null;

		try {
			const filenames = selectedEnabledItems.map((item) => item.filename);
			await contentService.disableContent(instance.id, filenames, activeContentType);
			await loadAllContent();
			selectedItems = new SvelteSet<string>();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to disable content';
		} finally {
			isBulkActioning = false;
		}
	}

	async function handleEnableSelected() {
		if (selectedDisabledItems.length === 0) return;

		isBulkActioning = true;
		error = null;

		try {
			const filenames = selectedDisabledItems.map((item) => item.filename);
			await contentService.enableContent(instance.id, filenames, activeContentType);
			await loadAllContent();
			selectedItems = new SvelteSet<string>();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to enable content';
		} finally {
			isBulkActioning = false;
		}
	}

	async function handleRemoveSelected() {
		if (selectedItems.size === 0) return;

		isBulkActioning = true;
		error = null;

		try {
			for (const filename of selectedItems) {
				await contentService.uninstallContentByFilename(instance.id, filename, activeContentType);
			}
			await loadAllContent();
			selectedItems = new SvelteSet<string>();
			showRemoveConfirm = false;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to remove content';
		} finally {
			isBulkActioning = false;
		}
	}

	function getContentTypeLabel(type: ContentType): string {
		switch (type) {
			case 'mod':
				return 'Mods';
			case 'shader':
				return 'Shaders';
			case 'resourcepack':
				return 'Resource Packs';
			case 'datapack':
				return 'Datapacks';
			case 'world':
				return 'Worlds';
		}
	}
</script>

<div class="flex h-full flex-col gap-4">
	<!-- Header with Add Button and Content Type Tabs -->
	<div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
		<!-- Content Type Tabs -->
		<div class="flex gap-2">
			<Button
				size="sm"
				variant={activeContentType === 'mod' ? 'default' : 'secondary'}
				onclick={() => (activeContentType = 'mod')}
			>
				<Package class="mr-1.5 h-4 w-4" />
				Mods ({modCount})
			</Button>
			<Button
				size="sm"
				variant={activeContentType === 'shader' ? 'default' : 'secondary'}
				onclick={() => (activeContentType = 'shader')}
			>
				<Sparkles class="mr-1.5 h-4 w-4" />
				Shaders ({shaderCount})
			</Button>
			<Button
				size="sm"
				variant={activeContentType === 'resourcepack' ? 'default' : 'secondary'}
				onclick={() => (activeContentType = 'resourcepack')}
			>
				<Image class="mr-1.5 h-4 w-4" />
				Resource Packs ({resourcepackCount})
			</Button>
			<Button
				size="sm"
				variant={activeContentType === 'datapack' ? 'default' : 'secondary'}
				onclick={() => (activeContentType = 'datapack')}
			>
				<FileJson class="mr-1.5 h-4 w-4" />
				Datapacks ({datapackCount})
			</Button>
			<Button
				size="sm"
				variant={activeContentType === 'world' ? 'default' : 'secondary'}
				onclick={() => (activeContentType = 'world')}
			>
				<Globe class="mr-1.5 h-4 w-4" />
				Worlds ({worldCount})
			</Button>
		</div>

		<!-- Add Content Button -->
		<div class="flex gap-2">
			<Button variant="outline" size="sm" onclick={handleRefresh} disabled={isLoading}>
				<RefreshCw class="mr-1.5 h-4 w-4 {isLoading ? 'animate-spin' : ''}" />
				Refresh
			</Button>
			<Button onclick={() => onOpenContentBrowser(activeContentType)}>
				<Plus class="mr-1.5 h-4 w-4" />
				Add {getContentTypeLabel(activeContentType)}
			</Button>
		</div>
	</div>

	<!-- Error Display -->
	{#if error}
		<div
			class="border-destructive/60 bg-destructive/10 text-destructive rounded border px-4 py-3 text-sm"
		>
			{error}
		</div>
	{/if}

	<!-- Content List -->
	<div class="flex min-h-0 flex-1 flex-col">
		{#if isLoading}
			<div class="text-muted-foreground flex flex-1 items-center justify-center gap-3">
				<Loader2 class="h-6 w-6 animate-spin" />
				<span>Scanning installed content...</span>
			</div>
		{:else if installedItems.length === 0}
			<div class="border-border bg-card/50 flex-1 border-2 border-dashed p-12 text-center">
				<Package class="text-muted-foreground/50 mx-auto h-12 w-12" />
				<p class="text-muted-foreground mt-4 text-sm">
					No {getContentTypeLabel(activeContentType).toLowerCase()} installed
				</p>
				<Button
					variant="outline"
					size="sm"
					class="mt-4"
					onclick={() => onOpenContentBrowser(activeContentType)}
				>
					Browse {getContentTypeLabel(activeContentType).toLowerCase()}
				</Button>
			</div>
		{:else}
			<!-- Search -->
			<div class="mb-3">
				<div class="flex gap-2">
					<div class="relative flex-1">
						<Search
							class="text-muted-foreground absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2"
						/>
						<Input
							value={search}
							oninput={(e) => (search = e.currentTarget.value)}
							placeholder="Search installed {getContentTypeLabel(
								activeContentType
							).toLowerCase()}..."
							class="pl-9"
						/>
					</div>
					{#if search.trim()}
						<Button variant="outline" size="sm" onclick={() => (search = '')}>Clear</Button>
					{/if}
				</div>
			</div>

			<!-- Select All Header -->
			<div class="border-border mb-3 flex items-center justify-between border-b pb-2">
				<button
					class="text-muted-foreground hover:text-foreground flex items-center gap-2 text-sm"
					onclick={toggleSelectAll}
				>
					{#if allSelected}
						<SquareCheck class="text-primary h-4 w-4" />
					{:else if someSelected}
						<div class="relative">
							<Square class="h-4 w-4" />
							<Minus class="absolute top-1/2 left-1/2 h-2 w-2 -translate-x-1/2 -translate-y-1/2" />
						</div>
					{:else}
						<Square class="h-4 w-4" />
					{/if}
					Select All
				</button>
				<div class="text-muted-foreground text-xs">
					{#if search.trim()}
						{visibleItems.length} of {installedItems.length} items
					{:else}
						{installedItems.length} items
					{/if}
					{#if visibleItems.filter((i) => i.isDisabled).length > 0}
						&bull; {visibleItems.filter((i) => i.isDisabled).length} disabled
					{/if}
				</div>
			</div>

			<!-- Items List -->
			<div class="flex-1 space-y-2 overflow-y-auto" use:nestedScroll>
				{#if visibleItems.length === 0 && search.trim()}
					<div class="border-border bg-card/50 border-2 border-dashed p-8 text-center">
						<p class="text-muted-foreground text-sm">No matches for "{search.trim()}"</p>
						<Button variant="outline" size="sm" class="mt-3" onclick={() => (search = '')}>
							Clear search
						</Button>
					</div>
				{/if}

				{#each visibleItems as item (item.filename)}
					{@const isSelected = selectedItems.has(item.filename)}
					<button
						class="flex w-full gap-3 border-2 p-3 text-left transition-colors {isSelected
							? 'border-primary bg-primary/5'
							: 'border-border bg-background hover:border-primary/50'} {item.isDisabled
							? 'opacity-60'
							: ''}"
						onclick={() => toggleItemSelection(item.filename)}
					>
						<!-- Checkbox -->
						<div class="flex-shrink-0 pt-0.5">
							{#if isSelected}
								<SquareCheck class="text-primary h-5 w-5" />
							{:else}
								<Square class="text-muted-foreground h-5 w-5" />
							{/if}
						</div>

						<!-- Content -->
						<div class="min-w-0 flex-1">
							<div class="flex items-start justify-between gap-2">
								<div class="flex min-w-0 items-center gap-2">
									<h3 class="truncate font-medium">{getItemDisplayName(item)}</h3>
									{#if item.isDisabled}
										<span
											class="flex flex-shrink-0 items-center gap-1 rounded bg-amber-500/20 px-1.5 py-0.5 text-xs text-amber-500"
										>
											<PowerOff class="h-3 w-3" />
											Disabled
										</span>
									{/if}
									{#if item.dependencyOf.length > 0}
										<span
											class="flex flex-shrink-0 items-center gap-1 rounded bg-blue-500/20 px-1.5 py-0.5 text-xs text-blue-500"
											title="Required by: {item.dependencyOf.join(', ')}"
										>
											<Link class="h-3 w-3" />
											Dependency
										</span>
									{/if}
								</div>
								<div class="flex flex-shrink-0 items-center gap-1">
									{#if item.modrinthProject}
										<span
											class="rounded border border-green-500/50 bg-green-500/20 px-1.5 py-0.5 text-xs text-green-500"
										>
											modrinth
										</span>
									{:else if item.curseforgeProject}
										<span
											class="rounded border border-orange-500/50 bg-orange-500/20 px-1.5 py-0.5 text-xs text-orange-500"
										>
											curseforge
										</span>
									{:else}
										<span
											class="bg-muted text-muted-foreground border-muted rounded border px-1.5 py-0.5 text-xs"
										>
											unknown
										</span>
									{/if}
								</div>
							</div>
							<p class="text-muted-foreground mt-0.5 truncate text-xs">{item.filename}</p>
							<div class="text-muted-foreground mt-1 flex flex-wrap items-center gap-3 text-xs">
								{#if getItemVersion(item)}
									<span>v{getItemVersion(item)}</span>
								{/if}
								<span>{formatBytes(item.size)}</span>
								{#if item.dependencyOf.length > 0}
									<div class="relative">
										<!-- svelte-ignore a11y_no_static_element_interactions -->
										<!-- svelte-ignore a11y_click_events_have_key_events -->
										<span
											class="cursor-pointer text-blue-500 underline decoration-blue-500/50 underline-offset-2 hover:decoration-blue-500"
											onclick={(e) => {
												e.stopPropagation();
												showDependentsFor =
													showDependentsFor === item.filename ? null : item.filename;
											}}
										>
											Required by {item.dependencyOf.length} mod{item.dependencyOf.length === 1
												? ''
												: 's'}
										</span>
										{#if showDependentsFor === item.filename}
											<!-- svelte-ignore a11y_no_static_element_interactions -->
											<!-- svelte-ignore a11y_click_events_have_key_events -->
											<div
												class="border-border bg-popover absolute bottom-full left-0 z-10 mb-1 w-64 border p-3 shadow-lg"
												onclick={(e) => e.stopPropagation()}
											>
												<div class="mb-2 flex items-center justify-between">
													<span class="text-sm font-medium">Required by:</span>
													<!-- svelte-ignore a11y_no_static_element_interactions -->
													<!-- svelte-ignore a11y_click_events_have_key_events -->
													<span
														class="text-muted-foreground hover:text-foreground cursor-pointer text-xs"
														onclick={(e) => {
															e.stopPropagation();
															showDependentsFor = null;
														}}
													>
														Close
													</span>
												</div>
												<ul class="space-y-1">
													{#each item.dependencyOf as parentFilename (parentFilename)}
														<li class="text-muted-foreground truncate text-xs">
															{parentFilename}
														</li>
													{/each}
												</ul>
											</div>
										{/if}
									</div>
								{/if}
							</div>
						</div>
					</button>
				{/each}
			</div>

			<!-- Action Bar (shown when items are selected) -->
			{#if selectedItems.size > 0}
				<div class="border-border bg-background border-t pt-4">
					<div class="mb-2 flex items-center justify-between">
						<span class="text-muted-foreground text-sm">
							{selectedItems.size} item{selectedItems.size === 1 ? '' : 's'} selected
						</span>
						<button
							class="text-muted-foreground hover:text-foreground text-xs"
							onclick={() => (selectedItems = new SvelteSet<string>())}
						>
							Clear selection
						</button>
					</div>
					<div class="flex gap-2">
						{#if selectedEnabledItems.length > 0}
							<Button
								variant="secondary"
								size="sm"
								class="flex-1"
								disabled={isBulkActioning}
								onclick={handleDisableSelected}
							>
								{#if isBulkActioning}
									<Loader2 class="mr-2 h-4 w-4 animate-spin" />
								{:else}
									<PowerOff class="mr-2 h-4 w-4" />
								{/if}
								Disable ({selectedEnabledItems.length})
							</Button>
						{/if}
						{#if selectedDisabledItems.length > 0}
							<Button
								variant="secondary"
								size="sm"
								class="flex-1"
								disabled={isBulkActioning}
								onclick={handleEnableSelected}
							>
								{#if isBulkActioning}
									<Loader2 class="mr-2 h-4 w-4 animate-spin" />
								{:else}
									<Power class="mr-2 h-4 w-4" />
								{/if}
								Enable ({selectedDisabledItems.length})
							</Button>
						{/if}
						<Button
							variant="destructive"
							size="sm"
							class="flex-1"
							disabled={isBulkActioning}
							onclick={() => (showRemoveConfirm = true)}
						>
							<Trash2 class="mr-2 h-4 w-4" />
							Remove ({selectedItems.size})
						</Button>
					</div>
				</div>
			{/if}
		{/if}
	</div>
</div>

<!-- Remove Confirmation Modal -->
{#if showRemoveConfirm}
	<div class="fixed inset-0 z-[100] flex items-center justify-center bg-black/50">
		<div class="bg-card border-border mx-4 w-full max-w-md space-y-4 border-2 p-6">
			<h2 class="text-lg font-bold">Remove Content</h2>
			<p class="text-muted-foreground text-sm">
				Are you sure you want to remove {selectedItems.size} item{selectedItems.size === 1
					? ''
					: 's'}? This action cannot be undone.
			</p>

			{#if selectedItemsWithDependents.length > 0}
				<div
					class="flex flex-col gap-2 border border-amber-500/60 bg-amber-500/10 px-4 py-3 text-sm text-amber-500"
				>
					<div class="flex items-center gap-2 font-medium">
						<Link class="h-4 w-4" />
						Dependency Warning
					</div>
					<p class="text-xs text-amber-500/80">
						{selectedItemsWithDependents.length === 1
							? 'One of the selected items is'
							: `${selectedItemsWithDependents.length} of the selected items are`} a dependency of other
						mods. Removing
						{selectedItemsWithDependents.length === 1 ? 'it' : 'them'} may cause issues.
					</p>
					<ul class="mt-1 list-inside list-disc text-xs text-amber-500/80">
						{#each selectedItemsWithDependents.slice(0, 5) as item (item.filename)}
							<li>
								<span class="font-medium">{getItemDisplayName(item)}</span>
								<span class="text-amber-500/60">
									required by {item.dependencyOf.length === 1
										? item.dependencyOf[0]
										: `${item.dependencyOf.length} mods`}
								</span>
							</li>
						{/each}
						{#if selectedItemsWithDependents.length > 5}
							<li class="text-amber-500/60">
								...and {selectedItemsWithDependents.length - 5} more
							</li>
						{/if}
					</ul>
				</div>
			{/if}

			<div class="flex gap-2 pt-2">
				<Button
					variant="outline"
					class="flex-1"
					onclick={() => (showRemoveConfirm = false)}
					disabled={isBulkActioning}
				>
					Cancel
				</Button>
				<Button
					variant="destructive"
					class="flex-1"
					onclick={handleRemoveSelected}
					disabled={isBulkActioning}
				>
					{#if isBulkActioning}
						<Loader2 class="mr-2 h-4 w-4 animate-spin" />
						Removing...
					{:else}
						Remove {selectedItemsWithDependents.length > 0 ? 'Anyway' : ''}
					{/if}
				</Button>
			</div>
		</div>
	</div>
{/if}
