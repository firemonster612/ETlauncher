<script lang="ts">
	import { onMount } from 'svelte';
	import { SkinViewer, WalkingAnimation, IdleAnimation } from 'skinview3d';

	interface Props {
		skinUrl?: string;
		capeUrl?: string;
		slim?: boolean;
		width?: number;
		height?: number;
		animation?: 'walk' | 'idle' | 'none';
	}

	let { skinUrl, capeUrl, slim = false, width = 200, height = 300, animation = 'idle' }: Props = $props();

	let containerEl: HTMLDivElement;
	let viewer: SkinViewer | null = null;

	onMount(() => {
		// Create the viewer
		viewer = new SkinViewer({
			canvas: document.createElement('canvas'),
			width,
			height,
			skin: skinUrl,
			cape: capeUrl,
			model: slim ? 'slim' : 'default',
		});

		// Set up animation
		if (animation === 'walk') {
			viewer.animation = new WalkingAnimation();
		} else if (animation === 'idle') {
			viewer.animation = new IdleAnimation();
		}

		// Enable controls for rotation
		viewer.controls.enableRotate = true;
		viewer.controls.enableZoom = false;
		viewer.controls.enablePan = false;

		// Add canvas to container (required by skinview3d library)
		// eslint-disable-next-line svelte/no-dom-manipulating
		containerEl.appendChild(viewer.canvas);

		// Cleanup on destroy
		return () => {
			if (viewer) {
				viewer.dispose();
				viewer = null;
			}
		};
	});

	// Update skin when URL changes
	$effect(() => {
		if (viewer && skinUrl !== undefined) {
			viewer.loadSkin(skinUrl, { model: slim ? 'slim' : 'default' });
		}
	});

	// Update cape when URL changes
	$effect(() => {
		if (viewer) {
			if (capeUrl) {
				viewer.loadCape(capeUrl);
			} else {
				viewer.resetCape();
			}
		}
	});

	// Update model type when slim changes
	$effect(() => {
		if (viewer && skinUrl) {
			viewer.loadSkin(skinUrl, { model: slim ? 'slim' : 'default' });
		}
	});

	// Update animation when it changes
	$effect(() => {
		if (viewer) {
			if (animation === 'walk') {
				viewer.animation = new WalkingAnimation();
			} else if (animation === 'idle') {
				viewer.animation = new IdleAnimation();
			} else {
				viewer.animation = null;
			}
		}
	});

	// Update size when dimensions change
	$effect(() => {
		if (viewer) {
			viewer.width = width;
			viewer.height = height;
		}
	});
</script>

<div
	bind:this={containerEl}
	class="flex items-center justify-center overflow-hidden"
	style="width: {width}px; height: {height}px;"
></div>
