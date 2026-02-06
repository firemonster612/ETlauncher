<script lang="ts">
	import { X } from '@lucide/svelte';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import { Button } from '$lib/ui/button';

	interface Props {
		open: boolean;
		title: string;
		html: string;
		onClose: () => void;
	}

	let { open, title, html, onClose }: Props = $props();

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) onClose();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (!open) return;
		if (e.key === 'Escape') onClose();
	}

	function stopClickPropagation(e: MouseEvent) {
		e.stopPropagation();
	}

	async function handleLinkClick(e: MouseEvent) {
		const target = e.target as HTMLElement | null;
		const anchor = target?.closest('a') as HTMLAnchorElement | null;
		if (!anchor) return;

		const href = anchor.getAttribute('href');
		if (!href || href.startsWith('#')) return;

		e.preventDefault();
		e.stopPropagation();
		try {
			await openUrl(href);
		} catch (err) {
			console.error('Failed to open URL:', href, err);
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
	<div
		class="fixed inset-0 z-[60] flex items-center justify-center bg-black/70 p-6"
		onclick={handleBackdropClick}
		role="dialog"
		aria-label={title}
		tabindex="-1"
		onkeydown={handleKeydown}
	>
		<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
		<div
			class="bg-card border-border relative flex max-h-[90vh] w-full max-w-7xl flex-col overflow-hidden rounded-lg border-2 shadow-2xl"
			onclick={stopClickPropagation}
			onkeydown={(e) => e.stopPropagation()}
			role="document"
		>
			<div class="border-border flex items-center justify-between gap-3 border-b px-5 py-4">
				<h3 class="truncate font-semibold">{title}</h3>
				<Button variant="secondary" size="icon" onclick={onClose} aria-label="Close">
					<X class="h-5 w-5" />
				</Button>
			</div>

		<div class="min-h-0 flex-1 overflow-y-auto p-5" onclick={handleLinkClick} onkeydown={(e) => e.stopPropagation()} role="presentation">
			<div class="prose-markdown">
				<!-- eslint-disable-next-line svelte/no-at-html-tags -->
				{@html html}
			</div>
		</div>
		</div>
	</div>
{/if}
