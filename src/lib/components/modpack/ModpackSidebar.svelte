<script lang="ts">
	import { Checkbox } from '$lib/ui/checkbox';
	import * as Select from '$lib/ui/select';
	import CollapsibleFilterCard from './CollapsibleFilterCard.svelte';
	import PlatformToggle from './PlatformToggle.svelte';
	import type { ModpackPlatform, ModpackSortBy, LoaderType } from '$lib/types';

	interface Props {
		// Platform state
		platforms: ModpackPlatform[];
		activePlatforms: ModpackPlatform[];
		onPlatformToggle: (platform: ModpackPlatform) => void;

		// Sort
		sortBy: ModpackSortBy;
		onSortChange: (sort: ModpackSortBy) => void;

		// MC Version
		mcVersion: string | null;
		mcVersions: string[];
		onMcVersionChange: (version: string | null) => void;
		showVersionFilter: boolean;

		// Loaders
		selectedLoaders: LoaderType[];
		onLoaderToggle: (loader: LoaderType) => void;
		showLoaderFilter: boolean;

		// Categories
		categories: string[];
		selectedCategories: string[];
		onCategoryToggle: (category: string) => void;
		showCategoryFilter: boolean;
	}

	let {
		platforms,
		activePlatforms,
		onPlatformToggle,
		sortBy,
		onSortChange,
		mcVersion,
		mcVersions,
		onMcVersionChange,
		showVersionFilter,
		selectedLoaders,
		onLoaderToggle,
		showLoaderFilter,
		categories,
		selectedCategories,
		onCategoryToggle,
		showCategoryFilter,
	}: Props = $props();

	const sortOptions: { value: ModpackSortBy; label: string }[] = [
		{ value: 'downloads', label: 'Downloads' },
		{ value: 'recentlyUpdated', label: 'Recently Updated' },
		{ value: 'relevance', label: 'Relevance' },
		{ value: 'name', label: 'Name' },
	];

	const loaderOptions: { value: LoaderType; label: string }[] = [
		{ value: 'fabric', label: 'Fabric' },
		{ value: 'forge', label: 'Forge' },
		{ value: 'neoforge', label: 'NeoForge' },
		{ value: 'quilt', label: 'Quilt' },
	];

	let loadersOpen = $state(true);
	let categoriesOpen = $state(true);

	// Computed filter counts for badges
	let platformCount = $derived(activePlatforms.length);
	let loaderCount = $derived(selectedLoaders.length);
	let categoryCount = $derived(selectedCategories.length);
</script>

<aside class="modpack-sidebar border-border bg-card space-y-4 border-r-2 p-4">
	<!-- Sources / Platforms -->
	<div class="space-y-2">
		<div class="flex items-center justify-between">
			<h3 class="text-muted-foreground text-xs font-semibold tracking-wider uppercase">Sources</h3>
			{#if platformCount > 0}
				<span class="bg-primary text-primary-foreground rounded-full px-1.5 text-[10px] font-bold">
					{platformCount}
				</span>
			{/if}
		</div>
		<div class="flex flex-wrap gap-1.5">
			{#each platforms as platform (platform)}
				<PlatformToggle
					{platform}
					active={activePlatforms.includes(platform)}
					onclick={() => onPlatformToggle(platform)}
				/>
			{/each}
		</div>
	</div>

	<!-- Sort -->
	<div class="space-y-2">
		<h3 class="text-muted-foreground text-xs font-semibold tracking-wider uppercase">Sort By</h3>
		<Select.Root
			type="single"
			value={sortBy}
			onValueChange={(v) => onSortChange(v as ModpackSortBy)}
		>
			<Select.Trigger class="border-border bg-background w-full border-2 text-sm">
				{sortOptions.find((o) => o.value === sortBy)?.label || 'Sort by'}
			</Select.Trigger>
			<Select.Content class="border-border bg-card border-2">
				{#each sortOptions as { value, label } (value)}
					<Select.Item {value} {label}>{label}</Select.Item>
				{/each}
			</Select.Content>
		</Select.Root>
	</div>

	<!-- MC Version -->
	{#if showVersionFilter}
		<div class="space-y-2">
			<h3 class="text-muted-foreground text-xs font-semibold tracking-wider uppercase">
				MC Version
			</h3>
			<Select.Root
				type="single"
				value={mcVersion || ''}
				onValueChange={(v) => onMcVersionChange(v || null)}
			>
				<Select.Trigger class="border-border bg-background w-full border-2 text-sm">
					{mcVersion || 'Any version'}
				</Select.Trigger>
				<Select.Content class="border-border bg-card max-h-[250px] border-2">
					<Select.Item value="" label="Any version">Any version</Select.Item>
					{#each mcVersions as version (version)}
						<Select.Item value={version} label={version}>{version}</Select.Item>
					{/each}
				</Select.Content>
			</Select.Root>
		</div>
	{/if}

	<!-- Loaders -->
	{#if showLoaderFilter}
		<CollapsibleFilterCard
			title="Loaders"
			bind:open={loadersOpen}
			badge={loaderCount > 0 ? loaderCount : undefined}
		>
			<div class="space-y-1">
				{#each loaderOptions as { value, label } (value)}
					<button
						type="button"
						class="hover:bg-muted/50 flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-sm"
						onclick={() => onLoaderToggle(value)}
					>
						<Checkbox checked={selectedLoaders.includes(value)} class="pointer-events-none" />
						<span>{label}</span>
					</button>
				{/each}
			</div>
		</CollapsibleFilterCard>
	{/if}

	<!-- Categories -->
	{#if showCategoryFilter && categories.length > 0}
		<CollapsibleFilterCard
			title="Categories"
			bind:open={categoriesOpen}
			badge={categoryCount > 0 ? categoryCount : undefined}
		>
			<div class="max-h-[200px] space-y-1 overflow-y-auto">
				{#each categories as category (category)}
					<button
						type="button"
						class="hover:bg-muted/50 flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-sm"
						onclick={() => onCategoryToggle(category)}
					>
						<Checkbox checked={selectedCategories.includes(category)} class="pointer-events-none" />
						<span class="capitalize">{category.replace(/-/g, ' ')}</span>
					</button>
				{/each}
			</div>
		</CollapsibleFilterCard>
	{/if}
</aside>
