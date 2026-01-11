<script lang="ts">
	import { Search, Check } from '@lucide/svelte';
	import { Input } from '$lib/ui/input';
	import {
		ENTITY_ICONS,
		ICON_CATEGORIES,
		getIconsByCategory,
		searchIcons,
		getIconUrl,
		type EntityIcon,
		type IconCategory,
	} from '$lib/utils/icons';

	interface Props {
		selected: string | undefined;
		onSelect: (icon: EntityIcon) => void;
	}

	let { selected, onSelect }: Props = $props();

	let searchQuery = $state('');
	let activeCategory = $state<IconCategory>('Passive');

	const displayedIcons = $derived(() => {
		if (searchQuery.trim()) {
			return searchIcons(searchQuery);
		}
		return getIconsByCategory(activeCategory);
	});

	function isSelected(icon: EntityIcon): boolean {
		return selected === `entity:${icon.id}`;
	}
</script>

<div class="space-y-4">
	<!-- Search -->
	<div class="relative">
		<Search class="text-muted-foreground absolute top-1/2 left-3 z-10 h-4 w-4 -translate-y-1/2" />
		<Input type="text" placeholder="Search icons..." bind:value={searchQuery} class="pl-9" />
	</div>

	<!-- Category tabs -->
	{#if !searchQuery.trim()}
		<div class="flex flex-wrap gap-1">
			{#each ICON_CATEGORIES as category (category)}
				<button
					type="button"
					class="border-2 px-3 py-1.5 text-xs transition-colors {activeCategory === category
						? 'border-primary bg-primary/10 text-primary'
						: 'border-border hover:border-primary/50'}"
					onclick={() => (activeCategory = category)}
				>
					{category}
				</button>
			{/each}
		</div>
	{/if}

	<!-- Icon grid -->
	<div
		class="border-border h-48 overflow-y-auto border-2 p-2"
		style="overscroll-behavior: contain;"
	>
		{#if displayedIcons().length === 0}
			<div class="text-muted-foreground flex h-full items-center justify-center text-sm">
				No icons found
			</div>
		{:else}
			<div class="flex flex-wrap gap-1">
				{#each displayedIcons() as icon (icon.id)}
					<button
						type="button"
						class="hover:border-primary/50 relative border-2 p-1 transition-colors {isSelected(icon)
							? 'border-primary bg-primary/10'
							: 'border-transparent'}"
						onclick={() => onSelect(icon)}
						title={icon.name}
						style="line-height: 0;"
					>
						<img src={getIconUrl(icon)} alt={icon.name} class="pixelated h-8 w-8" />
						{#if isSelected(icon)}
							<div class="bg-primary absolute -top-1 -right-1 rounded-full p-0.5">
								<Check class="text-primary-foreground h-2.5 w-2.5" />
							</div>
						{/if}
					</button>
				{/each}
			</div>
		{/if}
	</div>

	<!-- Selected icon display -->
	{#if selected}
		{@const selectedIcon = ENTITY_ICONS.find((i) => `entity:${i.id}` === selected)}
		{#if selectedIcon}
			<div class="text-muted-foreground flex items-center gap-2 text-sm">
				<span>Selected:</span>
				<img src={getIconUrl(selectedIcon)} alt={selectedIcon.name} class="pixelated h-5 w-5" />
				<span class="text-foreground font-medium">{selectedIcon.name}</span>
			</div>
		{/if}
	{/if}
</div>
