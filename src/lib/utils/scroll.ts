/**
 * Scroll utilities for nested scroll containers
 *
 * Behavior:
 * - While scrolling, if you hit the bottom/top boundary, scroll stops (doesn't overflow to parent)
 * - If you stop scrolling and then start again while at the boundary, it propagates to parent
 */

/** Time in ms to consider a scroll gesture "ended" */
const SCROLL_END_DELAY = 150;

/**
 * Svelte action for nested scroll containers.
 * Manages scroll propagation at boundaries with gesture detection.
 *
 * Usage:
 * <div use:nestedScroll class="overflow-y-auto">
 *   ... scrollable content ...
 * </div>
 */
export function nestedScroll(node: HTMLElement) {
	let isScrolling = false;
	let scrollEndTimer: ReturnType<typeof setTimeout> | null = null;
	let wasAtBoundary = false;

	function handleWheel(e: WheelEvent) {
		const atTop = node.scrollTop <= 0;
		const atBottom = node.scrollTop + node.clientHeight >= node.scrollHeight - 1;
		const scrollingDown = e.deltaY > 0;
		const scrollingUp = e.deltaY < 0;

		const atBoundaryInScrollDirection =
			(atBottom && scrollingDown) || (atTop && scrollingUp);

		// Clear any existing timer
		if (scrollEndTimer) {
			clearTimeout(scrollEndTimer);
		}

		// If we're at a boundary in the scroll direction
		if (atBoundaryInScrollDirection) {
			// If this is a NEW scroll gesture (wasn't scrolling before) and we were already at boundary,
			// allow propagation to parent
			if (!isScrolling && wasAtBoundary) {
				// Let it propagate - don't stop
				isScrolling = true;
			} else {
				// We're in a continuous scroll that hit the boundary - block it
				e.preventDefault();
				e.stopPropagation();
				isScrolling = true;
				wasAtBoundary = true;
			}
		} else {
			// Not at boundary, normal scroll behavior
			e.stopPropagation();
			isScrolling = true;
			wasAtBoundary = false;
		}

		// Set timer to detect when scrolling stops
		scrollEndTimer = setTimeout(() => {
			isScrolling = false;
			// Check if we're at a boundary when scrolling ends
			const currentAtTop = node.scrollTop <= 0;
			const currentAtBottom =
				node.scrollTop + node.clientHeight >= node.scrollHeight - 1;
			wasAtBoundary = currentAtTop || currentAtBottom;
		}, SCROLL_END_DELAY);
	}

	// Need to use non-passive to allow preventDefault
	node.addEventListener('wheel', handleWheel, { passive: false });

	return {
		destroy() {
			node.removeEventListener('wheel', handleWheel);
			if (scrollEndTimer) {
				clearTimeout(scrollEndTimer);
			}
		},
	};
}

/**
 * Handle wheel events manually (for use without the action).
 * Note: This version is stateless and doesn't support the "new gesture" detection.
 * Use the `nestedScroll` action for full functionality.
 */
export function handleNestedScroll(e: WheelEvent) {
	const target = e.currentTarget as HTMLElement;
	if (!target) return;

	const atTop = target.scrollTop <= 0;
	const atBottom = target.scrollTop + target.clientHeight >= target.scrollHeight - 1;

	// If at a boundary and scrolling in that direction, prevent default
	if ((atBottom && e.deltaY > 0) || (atTop && e.deltaY < 0)) {
		e.preventDefault();
		e.stopPropagation();
		return;
	}

	// Otherwise, stop propagation so inner container handles the scroll
	e.stopPropagation();
}
