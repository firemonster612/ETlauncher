<script lang="ts" generics="T">
	import { onMount, tick } from 'svelte';
	import type { Snippet } from 'svelte';

	interface Props {
		items: T[];
		itemHeight: number;
		minItemWidth: number;
		gap?: number;
		overscan?: number;
		scrollContainer: HTMLElement | null;
		onLoadMore?: () => void;
		loadMoreThreshold?: number;
		children: Snippet<[T, number]>;
	}

	let {
		items,
		itemHeight,
		minItemWidth,
		gap = 16,
		overscan = 3,
		scrollContainer,
		onLoadMore,
		loadMoreThreshold = 500,
		children,
	}: Props = $props();

	let gridRef = $state<HTMLElement | null>(null);
	let containerWidth = $state(0);
	let scrollTop = $state(0);
	let viewportHeight = $state(0);
	let gridOffsetTop = $state(0);

	// Calculate columns based on container width
	let columns = $derived(
		containerWidth === 0
			? 1
			: Math.max(1, Math.floor((containerWidth + gap) / (minItemWidth + gap)))
	);

	// Calculate item width to fill available space
	let itemWidth = $derived(
		columns === 1 ? containerWidth : (containerWidth - gap * (columns - 1)) / columns
	);

	// Calculate rows and total height
	let totalRows = $derived(Math.ceil(items.length / columns));
	let rowHeight = $derived(itemHeight + gap);
	let totalHeight = $derived(Math.max(0, totalRows * rowHeight - gap));

	// Calculate the scroll position relative to the grid
	let relativeScrollTop = $derived(Math.max(0, scrollTop - gridOffsetTop));

	// Calculate visible row range based on relative scroll position
	let visibleStartRow = $derived(Math.max(0, Math.floor(relativeScrollTop / rowHeight) - overscan));
	let visibleEndRow = $derived(
		Math.min(totalRows, Math.ceil((relativeScrollTop + viewportHeight) / rowHeight) + overscan)
	);

	// Get visible items
	let visibleItems = $derived.by(() => {
		if (columns === 0 || rowHeight === 0) return [];

		const result: { item: T; index: number; row: number; col: number }[] = [];

		for (let row = visibleStartRow; row < visibleEndRow; row++) {
			for (let col = 0; col < columns; col++) {
				const index = row * columns + col;
				if (index < items.length) {
					result.push({ item: items[index], index, row, col });
				}
			}
		}

		return result;
	});

	function handleScroll() {
		if (!scrollContainer || !gridRef) return;

		scrollTop = scrollContainer.scrollTop;

		// Update grid offset (it might change if content above changes)
		const scrollContainerRect = scrollContainer.getBoundingClientRect();
		const gridRect = gridRef.getBoundingClientRect();
		gridOffsetTop = gridRect.top - scrollContainerRect.top + scrollContainer.scrollTop;

		// Check if we need to load more
		if (onLoadMore) {
			const scrollBottom =
				scrollContainer.scrollHeight - scrollContainer.scrollTop - scrollContainer.clientHeight;
			if (scrollBottom < loadMoreThreshold) {
				onLoadMore();
			}
		}
	}

	function updateDimensions() {
		if (!scrollContainer || !gridRef) return;

		containerWidth = gridRef.clientWidth;
		viewportHeight = scrollContainer.clientHeight;

		const scrollContainerRect = scrollContainer.getBoundingClientRect();
		const gridRect = gridRef.getBoundingClientRect();
		gridOffsetTop = gridRect.top - scrollContainerRect.top + scrollContainer.scrollTop;
		scrollTop = scrollContainer.scrollTop;
	}

	// Update when scroll container changes
	$effect(() => {
		if (scrollContainer) {
			scrollContainer.addEventListener('scroll', handleScroll, { passive: true });
			updateDimensions();

			return () => {
				scrollContainer.removeEventListener('scroll', handleScroll);
			};
		}
	});

	// Update when items change
	$effect(() => {
		// Track items dependency
		if (items.length >= 0) {
			// Wait for DOM update then recalculate
			tick().then(updateDimensions);
		}
	});

	onMount(() => {
		if (!gridRef) return;

		const resizeObserver = new ResizeObserver(() => {
			updateDimensions();
		});

		resizeObserver.observe(gridRef);
		if (scrollContainer) {
			resizeObserver.observe(scrollContainer);
		}

		// Initial calculation
		updateDimensions();

		return () => resizeObserver.disconnect();
	});
</script>

<div bind:this={gridRef} class="virtual-grid" style="height: {totalHeight}px;">
	{#each visibleItems as { item, index, row, col } (index)}
		<div
			class="virtual-grid-item"
			style="
				transform: translate({col * (itemWidth + gap)}px, {row * rowHeight}px);
				width: {itemWidth}px;
				height: {itemHeight}px;
			"
		>
			{@render children(item, index)}
		</div>
	{/each}
</div>

<style>
	.virtual-grid {
		position: relative;
		width: 100%;
	}

	.virtual-grid-item {
		position: absolute;
		top: 0;
		left: 0;
		contain: layout style paint;
		will-change: transform;
	}
</style>
