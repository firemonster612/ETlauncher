<script lang="ts">
	import { Check, Ban } from '@lucide/svelte';
	import CapeThumbnail from './CapeThumbnail.svelte';
	import type { CapeInfo } from '$lib/types';

	interface Props {
		capes: CapeInfo[];
		selectedCapeId?: string | null;
		onSelect: (cape: CapeInfo | null) => void;
	}

	let { capes, selectedCapeId, onSelect }: Props = $props();
</script>

<div class="space-y-2">
	<h3 class="text-sm font-medium">Cape</h3>

	{#if capes.length === 0}
		<div class="text-muted-foreground border-border border-2 border-dashed p-4 text-center text-sm">
			No capes available for this account.
		</div>
	{:else}
		<div class="flex flex-wrap gap-2">
			<!-- No Cape option -->
			<button
				type="button"
				class="group relative flex flex-col items-center border-2 p-2 transition-colors {selectedCapeId === null
					? 'border-primary bg-primary/10'
					: 'border-border hover:border-primary/50'}"
				onclick={() => onSelect(null)}
				title="No Cape"
			>
				<div class="bg-muted flex h-12 w-8 items-center justify-center">
					<Ban class="text-muted-foreground h-4 w-4" />
				</div>
				<span class="mt-1 text-xs">None</span>
				{#if selectedCapeId === null}
					<div class="bg-primary absolute -top-1 -right-1 rounded-full p-0.5">
						<Check class="text-primary-foreground h-3 w-3" />
					</div>
				{/if}
			</button>

			{#each capes as cape (cape.id)}
				{@const isSelected = selectedCapeId === cape.id}
				<button
					type="button"
					class="group relative flex flex-col items-center border-2 p-2 transition-colors {isSelected
						? 'border-primary bg-primary/10'
						: 'border-border hover:border-primary/50'}"
					onclick={() => onSelect(cape)}
					title={cape.alias}
				>
					<CapeThumbnail
						url={cape.url}
						alt={cape.alias}
						class="h-12 w-8"
					/>
					<span class="mt-1 max-w-16 truncate text-xs">{cape.alias}</span>
					{#if isSelected}
						<div class="bg-primary absolute -top-1 -right-1 rounded-full p-0.5">
							<Check class="text-primary-foreground h-3 w-3" />
						</div>
					{/if}
				</button>
			{/each}
		</div>
	{/if}
</div>
