<script lang="ts">
	import { ChevronLeft, ChevronRight, Loader2, X } from '@lucide/svelte';
	import { Button } from '$lib/ui/button';

	interface Props {
		open: boolean;
		src: string | null;
		filename?: string;
		isLoading?: boolean;
		canPrev?: boolean;
		canNext?: boolean;
		onClose: () => void;
		onPrev?: () => void;
		onNext?: () => void;
	}

	let {
		open,
		src,
		filename,
		isLoading = false,
		canPrev = true,
		canNext = true,
		onClose,
		onPrev,
		onNext,
	}: Props = $props();

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			onClose();
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (!open) return;
		if (e.key === 'Escape') onClose();
		if (e.key === 'ArrowLeft' && canPrev) onPrev?.();
		if (e.key === 'ArrowRight' && canNext) onNext?.();
	}

	function stopClickPropagation(e: MouseEvent) {
		e.stopPropagation();
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
	<div
		class="fixed inset-0 z-[60] flex items-center justify-center bg-black/80 p-6"
		onclick={handleBackdropClick}
		role="dialog"
		aria-label="Screenshot viewer"
		tabindex="-1"
		onkeydown={handleKeydown}
	>
		<div
			class="relative flex h-full max-h-[90vh] w-full max-w-6xl items-center justify-center"
			onclick={stopClickPropagation}
			onkeydown={(e) => e.stopPropagation()}
			role="presentation"
			tabindex="-1"
		>
			{#if isLoading}
				<div class="text-muted-foreground flex items-center gap-3">
					<Loader2 class="h-6 w-6 animate-spin" />
					<span>Loading image...</span>
				</div>
			{:else if src}
				<img
					{src}
					alt={filename ?? 'Screenshot'}
					class="max-h-full max-w-full rounded-lg object-contain shadow-2xl"
				/>
			{:else}
				<p class="text-muted-foreground">Unable to load image.</p>
			{/if}

			<div class="pointer-events-auto absolute top-4 right-4 z-20 flex gap-2">
				<Button variant="secondary" size="icon" onclick={() => onClose()} aria-label="Close">
					<X class="h-5 w-5" />
				</Button>
			</div>

			<div class="pointer-events-none absolute inset-y-0 left-2 z-10 flex items-center">
				<Button
					variant="secondary"
					size="icon"
					class="pointer-events-auto"
					onclick={() => onPrev?.()}
					disabled={!canPrev}
					aria-label="Previous"
				>
					<ChevronLeft class="h-5 w-5" />
				</Button>
			</div>
			<div class="pointer-events-none absolute inset-y-0 right-2 z-10 flex items-center">
				<Button
					variant="secondary"
					size="icon"
					class="pointer-events-auto"
					onclick={() => onNext?.()}
					disabled={!canNext}
					aria-label="Next"
				>
					<ChevronRight class="h-5 w-5" />
				</Button>
			</div>

			{#if filename}
				<div
					class="pointer-events-none absolute right-0 bottom-4 left-0 text-center text-sm text-white/80"
				>
					{filename}
				</div>
			{/if}
		</div>
	</div>
{/if}
