<script lang="ts">
	import { X } from '@lucide/svelte';
	import FilterChip from './FilterChip.svelte';
	import type { ModpackPlatform, LoaderType } from '$lib/types';

	interface Props {
		activePlatforms: ModpackPlatform[];
		mcVersion: string | null;
		selectedLoaders: LoaderType[];
		selectedCategories: string[];
		onClear: () => void;
		onRemovePlatform: (platform: ModpackPlatform) => void;
		onRemoveMcVersion: () => void;
		onRemoveLoader: (loader: LoaderType) => void;
		onRemoveCategory: (category: string) => void;
	}

	let {
		activePlatforms,
		mcVersion,
		selectedLoaders,
		selectedCategories,
		onClear,
		onRemovePlatform,
		onRemoveMcVersion,
		onRemoveLoader,
		onRemoveCategory,
	}: Props = $props();

	// Check if there are any active filters
	let hasFilters = $derived(
		activePlatforms.length > 0 ||
			mcVersion !== null ||
			selectedLoaders.length > 0 ||
			selectedCategories.length > 0
	);

	function getPlatformLabel(platform: ModpackPlatform): string {
		switch (platform) {
			case 'modrinth':
				return 'Modrinth';
			case 'curseforge':
				return 'CurseForge';
			case 'ftb':
				return 'FTB';
			case 'technic':
				return 'Technic';
			case 'atlauncher':
				return 'ATLauncher';
			default:
				return platform;
		}
	}

	function getLoaderLabel(loader: LoaderType): string {
		switch (loader) {
			case 'fabric':
				return 'Fabric';
			case 'forge':
				return 'Forge';
			case 'neoforge':
				return 'NeoForge';
			case 'quilt':
				return 'Quilt';
			default:
				return loader;
		}
	}
</script>

{#if hasFilters}
	<div class="active-filters-bar">
		<span class="text-muted-foreground text-xs font-medium tracking-wider uppercase">
			Active Filters:
		</span>

		<!-- Platform chips -->
		{#each activePlatforms as platform (platform)}
			<FilterChip label={getPlatformLabel(platform)} onRemove={() => onRemovePlatform(platform)} />
		{/each}

		<!-- MC Version chip -->
		{#if mcVersion}
			<FilterChip label={`MC ${mcVersion}`} onRemove={onRemoveMcVersion} />
		{/if}

		<!-- Loader chips -->
		{#each selectedLoaders as loader (loader)}
			<FilterChip label={getLoaderLabel(loader)} onRemove={() => onRemoveLoader(loader)} />
		{/each}

		<!-- Category chips -->
		{#each selectedCategories as category (category)}
			<FilterChip label={category.replace(/-/g, ' ')} onRemove={() => onRemoveCategory(category)} />
		{/each}

		<!-- Clear all button -->
		<button
			type="button"
			class="text-muted-foreground hover:text-destructive ml-auto flex items-center gap-1 text-xs transition-colors"
			onclick={onClear}
		>
			<X class="h-3 w-3" />
			Clear all
		</button>
	</div>
{/if}
