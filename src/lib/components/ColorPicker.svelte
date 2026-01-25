<script lang="ts">
	import { ChevronDown } from '@lucide/svelte';

	interface Props {
		hue: number;
		saturation: number; // 0-1 range
		onchange?: (hue: number, saturation: number) => void;
		oninput?: (hue: number, saturation: number) => void;
	}

	let { hue, saturation, onchange, oninput }: Props = $props();

	let isOpen = $state(false);
	let containerRef: HTMLDivElement | null = $state(null);
	let areaRef: HTMLDivElement | null = $state(null);
	let hueSliderRef: HTMLDivElement | null = $state(null);
	let isDraggingArea = $state(false);
	let isDraggingHue = $state(false);

	// Internal HSV state - these are the source of truth while picker is open
	let internalSat = $state(1);
	let internalVal = $state(1);
	let internalHue = $state(0);
	let initialized = $state(false);

	// Only sync from props when picker opens (not continuously)
	$effect(() => {
		if (isOpen && !initialized) {
			internalHue = hue;
			// Place cursor at top-right for high saturation, middle for low
			internalSat = Math.min(1, Math.sqrt(saturation));
			internalVal = Math.min(1, Math.sqrt(saturation) + 0.3);
			initialized = true;
		}
		if (!isOpen) {
			initialized = false;
		}
	});

	// Close on outside click
	function handleClickOutside(e: MouseEvent) {
		if (containerRef && !containerRef.contains(e.target as Node)) {
			isOpen = false;
		}
	}

	$effect(() => {
		if (isOpen) {
			document.addEventListener('mousedown', handleClickOutside);
			return () => document.removeEventListener('mousedown', handleClickOutside);
		}
	});

	function handleAreaMouseDown(e: MouseEvent) {
		e.preventDefault();
		isDraggingArea = true;
		updateAreaFromEvent(e);
		window.addEventListener('mousemove', handleAreaMouseMove);
		window.addEventListener('mouseup', handleAreaMouseUp);
	}

	function handleAreaMouseMove(e: MouseEvent) {
		if (!isDraggingArea) return;
		updateAreaFromEvent(e);
	}

	function handleAreaMouseUp() {
		isDraggingArea = false;
		window.removeEventListener('mousemove', handleAreaMouseMove);
		window.removeEventListener('mouseup', handleAreaMouseUp);
		const newSat = internalSat * internalVal;
		onchange?.(internalHue, newSat);
	}

	function updateAreaFromEvent(e: MouseEvent) {
		if (!areaRef) return;
		const rect = areaRef.getBoundingClientRect();
		internalSat = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
		internalVal = Math.max(0, Math.min(1, 1 - (e.clientY - rect.top) / rect.height));
		const newSat = internalSat * internalVal;
		oninput?.(internalHue, newSat);
	}

	function handleHueMouseDown(e: MouseEvent) {
		e.preventDefault();
		isDraggingHue = true;
		updateHueFromEvent(e);
		window.addEventListener('mousemove', handleHueMouseMove);
		window.addEventListener('mouseup', handleHueMouseUp);
	}

	function handleHueMouseMove(e: MouseEvent) {
		if (!isDraggingHue) return;
		updateHueFromEvent(e);
	}

	function handleHueMouseUp() {
		isDraggingHue = false;
		window.removeEventListener('mousemove', handleHueMouseMove);
		window.removeEventListener('mouseup', handleHueMouseUp);
		const currentSat = internalSat * internalVal;
		onchange?.(internalHue, currentSat);
	}

	function updateHueFromEvent(e: MouseEvent) {
		if (!hueSliderRef) return;
		const rect = hueSliderRef.getBoundingClientRect();
		internalHue = Math.max(0, Math.min(360, ((e.clientX - rect.left) / rect.width) * 360));
		const currentSat = internalSat * internalVal;
		oninput?.(internalHue, currentSat);
	}

	let pureHueColor = $derived(`hsl(${internalHue}, 100%, 50%)`);
	let cursorX = $derived(internalSat * 100);
	let cursorY = $derived((1 - internalVal) * 100);
	let huePosition = $derived((internalHue / 360) * 100);
	let previewColor = $derived(`hsl(${internalHue}, ${internalSat * 100}%, ${internalVal * 50}%)`);
</script>

<div bind:this={containerRef} class="relative inline-block">
	<!-- Color swatch trigger button -->
	<button
		type="button"
		class="border-border bg-muted/50 hover:border-primary/50 hover:bg-muted flex items-center gap-1.5 rounded border-2 px-2 py-1 transition-all"
		onclick={() => (isOpen = !isOpen)}
		aria-label="Pick color"
	>
		<div class="border-border/50 h-6 w-10 rounded border" style="background: {previewColor}"></div>
		<ChevronDown class="text-muted-foreground h-4 w-4" />
	</button>

	<!-- Popover -->
	{#if isOpen}
		<div
			class="border-border bg-card absolute top-full left-0 z-50 mt-2 w-56 rounded-lg border p-3 shadow-lg"
		>
			<!-- Saturation/Value Area -->
			<div
				bind:this={areaRef}
				class="border-border relative h-36 w-full cursor-crosshair overflow-hidden rounded-lg border"
				style="background: linear-gradient(to top, #000, transparent), linear-gradient(to right, #fff, {pureHueColor})"
				onmousedown={handleAreaMouseDown}
				role="slider"
				aria-label="Color saturation and brightness"
				aria-valuenow={Math.round(internalSat * 100)}
				aria-valuemin={0}
				aria-valuemax={100}
				tabindex="0"
			>
				<div
					class="pointer-events-none absolute h-4 w-4 -translate-x-1/2 -translate-y-1/2 rounded-full border-2 border-white"
					style="left: {cursorX}%; top: {cursorY}%; background: {previewColor}; box-shadow: 0 0 0 1px rgba(0,0,0,0.4), 0 2px 4px rgba(0,0,0,0.3)"
				></div>
			</div>

			<!-- Hue Slider -->
			<div
				bind:this={hueSliderRef}
				class="border-border relative mt-3 h-4 w-full cursor-pointer rounded-full border"
				style="background: linear-gradient(to right, hsl(0,100%,50%), hsl(60,100%,50%), hsl(120,100%,50%), hsl(180,100%,50%), hsl(240,100%,50%), hsl(300,100%,50%), hsl(360,100%,50%))"
				onmousedown={handleHueMouseDown}
				role="slider"
				aria-label="Color hue"
				aria-valuenow={Math.round(internalHue)}
				aria-valuemin={0}
				aria-valuemax={360}
				tabindex="0"
			>
				<div
					class="pointer-events-none absolute top-1/2 h-5 w-5 -translate-x-1/2 -translate-y-1/2 rounded-full border-2 border-white"
					style="left: {huePosition}%; background: hsl({internalHue}, 100%, 50%); box-shadow: 0 0 0 1px rgba(0,0,0,0.4), 0 2px 4px rgba(0,0,0,0.3)"
				></div>
			</div>
		</div>
	{/if}
</div>
