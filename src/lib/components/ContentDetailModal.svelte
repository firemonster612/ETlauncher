<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Package, Download, ExternalLink, X, Maximize2, ChevronLeft } from '@lucide/svelte';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import { renderMarkdown } from '$lib/utils/markdown';
	import { formatDownloads, getPlatformColor, getLoaderColor } from '$lib/utils/format';
	import { nestedScroll } from '$lib/utils/scroll';
	import { Button } from '$lib/ui/button';
	import ScreenshotLightbox from '$lib/components/ScreenshotLightbox.svelte';
	import DescriptionModal from '$lib/components/DescriptionModal.svelte';
	import type { ContentGalleryImage, LoaderType } from '$lib/types';

	interface Props {
		// Basic item info
		name: string;
		author: string;
		description?: string;
		body?: string;
		iconUrl?: string | null;
		platform: string;
		downloads: number;
		url?: string | null;
		loaders?: (LoaderType | string)[];
		gallery?: ContentGalleryImage[];

		// Navigation
		showBackButton?: boolean;
		backButtonTitle?: string;
		onBack?: () => void;
		onClose: () => void;

		// Tabs
		tabs?: { id: string; label: string; disabled?: boolean }[];
		activeTab?: string;
		onTabChange?: (tab: string) => void;

		// Loading states
		isLoadingDescription?: boolean;
		descriptionError?: string | null;

		// Custom slots for extensibility
		headerExtra?: Snippet;
		tabContent?: Snippet<[string]>;
		sidebar?: Snippet;
		footer?: Snippet;
	}

	let {
		name,
		author,
		description,
		body,
		iconUrl,
		platform,
		downloads,
		url,
		loaders = [],
		gallery = [],
		showBackButton = false,
		backButtonTitle,
		onBack,
		onClose,
		tabs = [
			{ id: 'about', label: 'About' },
			{ id: 'gallery', label: 'Gallery' },
		],
		activeTab = 'about',
		onTabChange,
		isLoadingDescription = false,
		descriptionError = null,
		headerExtra,
		tabContent,
		sidebar,
		footer,
	}: Props = $props();

	// Internal state
	let lightboxIndex = $state<number | null>(null);
	let descriptionExpanded = $state(false);

	// Filter out unknown/vanilla loaders for display
	const displayLoaders = $derived(loaders.filter((l) => l && l !== 'unknown' && l !== 'vanilla'));

	function openLightbox(index: number) {
		lightboxIndex = index;
	}

	function closeLightbox() {
		lightboxIndex = null;
	}

	function prevLightbox() {
		if (lightboxIndex !== null && lightboxIndex > 0) {
			lightboxIndex--;
		}
	}

	function nextLightbox() {
		if (lightboxIndex !== null && lightboxIndex < gallery.length - 1) {
			lightboxIndex++;
		}
	}

	async function handleDescriptionLinkClick(e: MouseEvent) {
		const target = e.target as HTMLElement | null;
		const anchor = target?.closest('a') as HTMLAnchorElement | null;
		if (!anchor) return;

		const href = anchor.getAttribute('href');
		if (!href) return;

		// Don't intercept anchor links
		if (href.startsWith('#')) return;

		e.preventDefault();

		try {
			await openUrl(href);
		} catch (err) {
			console.error('Failed to open URL:', err);
		}
	}

	function handleTabClick(tabId: string) {
		if (onTabChange) {
			onTabChange(tabId);
		}
	}
</script>

<!-- Modal Overlay -->
<div
	class="fixed inset-x-0 top-[var(--titlebar-height)] z-50 flex h-[calc(100vh-var(--titlebar-height))] items-center justify-center bg-black/50 p-4"
>
	<div
		class="bg-card border-border flex max-h-[90vh] w-full max-w-6xl flex-col overflow-hidden rounded-lg border-2 shadow-2xl"
		data-modal-content
	>
		<!-- Header -->
		<div class="border-border flex-shrink-0 border-b p-5">
			<div class="flex items-start gap-4">
				{#if showBackButton && onBack}
					<button
						class="text-muted-foreground hover:text-foreground mr-1 -ml-1 flex-shrink-0 self-center"
						onclick={onBack}
						title={backButtonTitle}
					>
						<ChevronLeft class="h-6 w-6" />
					</button>
				{/if}

				{#if iconUrl}
					<img src={iconUrl} alt={name} class="h-20 w-20 flex-shrink-0 rounded object-cover" />
				{:else}
					<div class="bg-muted flex h-20 w-20 flex-shrink-0 items-center justify-center rounded">
						<Package class="text-muted-foreground/50 h-10 w-10" />
					</div>
				{/if}

				<div class="min-w-0 flex-1 space-y-1">
					<div class="flex items-start justify-between gap-2">
						<h2 class="truncate text-xl font-bold">{name}</h2>
						<button
							class="text-muted-foreground hover:text-foreground flex-shrink-0"
							onclick={onClose}
						>
							<X class="h-5 w-5" />
						</button>
					</div>
					<p class="text-muted-foreground">{author}</p>
					<div class="mt-2 flex flex-wrap items-center gap-2">
						<span class="rounded border px-1.5 py-0.5 text-xs {getPlatformColor(platform)}">
							{platform}
						</span>
						{#each displayLoaders as loader (loader)}
							<span class="rounded px-1.5 py-0.5 text-xs {getLoaderColor(loader)}">
								{loader}
							</span>
						{/each}
						<span class="text-muted-foreground flex items-center gap-1 text-xs">
							<Download class="h-3 w-3" />
							{formatDownloads(downloads)}
						</span>
						{#if headerExtra}
							{@render headerExtra()}
						{/if}
					</div>
					{#if description}
						<p class="text-muted-foreground line-clamp-2 pt-1 text-sm">
							{description}
						</p>
					{/if}
				</div>
			</div>
		</div>

		<!-- Main Content Area -->
		<div
			class="grid min-h-0 flex-1 grid-cols-1 gap-4 overflow-hidden p-5 md:grid-cols-[2.5fr_1fr] xl:grid-cols-[3fr_1fr]"
		>
			<!-- Left Column: Tabs and Content -->
			<div class="min-h-0 space-y-4 overflow-y-auto pr-1" use:nestedScroll>
				<!-- Tab Buttons -->
				<div class="flex items-center gap-2">
					{#each tabs as tab (tab.id)}
						{@const isActive = activeTab === tab.id}
						{@const isDisabled = tab.disabled || (tab.id === 'gallery' && gallery.length === 0)}
						<Button
							size="sm"
							variant={isActive ? 'default' : 'secondary'}
							disabled={isDisabled}
							onclick={() => handleTabClick(tab.id)}
						>
							{tab.label}
						</Button>
					{/each}
				</div>

				<!-- Tab Content -->
				{#if tabContent}
					{@render tabContent(activeTab)}
				{:else if activeTab === 'gallery'}
					<!-- Default Gallery Tab -->
					{#if gallery.length > 0}
						<div class="space-y-2">
							<div class="flex items-center justify-between">
								<h3 class="text-sm font-semibold">Gallery</h3>
								<span class="text-muted-foreground text-xs">
									{gallery.length} images
								</span>
							</div>
							<div class="grid gap-3 sm:grid-cols-2">
								{#each gallery as image, idx (image.rawUrl ?? image.url)}
									<button
										type="button"
										class="border-border bg-muted/50 relative aspect-video cursor-pointer overflow-hidden rounded-lg border-2 text-left"
										onclick={() => openLightbox(idx)}
									>
										<img
											src={image.rawUrl ?? image.url}
											alt={image.title ?? name}
											class="h-full w-full object-cover"
											loading="lazy"
										/>
										{#if image.title || image.description}
											<div
												class="absolute inset-x-0 bottom-0 space-y-1 bg-gradient-to-t from-black/80 via-black/40 to-transparent p-2 text-xs text-white"
											>
												{#if image.title}
													<div class="truncate leading-tight font-semibold">{image.title}</div>
												{/if}
												{#if image.description}
													<p class="line-clamp-2 leading-snug opacity-90">{image.description}</p>
												{/if}
											</div>
										{/if}
									</button>
								{/each}
							</div>
						</div>
					{:else}
						<p class="text-muted-foreground text-sm">No gallery available.</p>
					{/if}
				{:else if activeTab === 'about'}
					<!-- Default About Tab -->
					<div class="border-border bg-background/70 space-y-2 rounded-lg border-2 p-4">
						<div class="flex items-center justify-between gap-2">
							<h3 class="text-sm font-semibold">About</h3>
							<div class="flex items-center gap-2">
								<Button
									size="sm"
									variant="secondary"
									onclick={() => (descriptionExpanded = true)}
									disabled={!body && !description}
								>
									<Maximize2 class="mr-1 h-4 w-4" />
									Expand
								</Button>
								{#if url}
									<a
										href={url}
										target="_blank"
										rel="noopener noreferrer"
										class="bg-muted hover:bg-muted/80 inline-flex items-center gap-1.5 rounded px-2.5 py-1.5 text-xs transition-colors"
									>
										<ExternalLink class="h-3.5 w-3.5" />
										View on {platform}
									</a>
								{/if}
							</div>
						</div>

						{#if isLoadingDescription}
							<div class="text-muted-foreground flex items-center gap-2 text-sm">
								<span
									class="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent"
								></span>
								Loading description...
							</div>
						{:else}
							{#if descriptionError}
								<div
									class="bg-destructive/10 border-destructive text-destructive rounded border-2 p-3 text-sm"
								>
									{descriptionError}
								</div>
							{/if}
							{#if body || description}
								<!-- svelte-ignore a11y_click_events_have_key_events -->
								<!-- svelte-ignore a11y_no_static_element_interactions -->
								<div
									class="[&_a]:text-primary text-sm leading-relaxed [&_a]:underline [&_h1]:text-lg [&_h1]:font-semibold [&_h2]:text-base [&_h2]:font-semibold [&_img]:my-2 [&_img]:max-w-full [&_img]:rounded-md [&_ol]:list-decimal [&_ol]:pl-5 [&_p]:mb-3 [&_ul]:list-disc [&_ul]:pl-5"
									onclick={handleDescriptionLinkClick}
								>
									<!-- eslint-disable-next-line svelte/no-at-html-tags -->
									{@html renderMarkdown(body, description)}
								</div>
							{:else}
								<p class="text-muted-foreground text-sm">No description available.</p>
							{/if}
						{/if}
					</div>
				{/if}
			</div>

			<!-- Right Column: Sidebar -->
			<div class="min-h-0 space-y-3 overflow-y-auto pr-1" use:nestedScroll>
				{#if sidebar}
					{@render sidebar()}
				{/if}
			</div>
		</div>

		<!-- Footer -->
		{#if footer}
			<div class="border-border flex-shrink-0 border-t">
				{@render footer()}
			</div>
		{/if}
	</div>
</div>

<!-- Lightbox for Gallery -->
<ScreenshotLightbox
	open={lightboxIndex !== null}
	src={lightboxIndex !== null
		? (gallery[lightboxIndex]?.rawUrl ?? gallery[lightboxIndex]?.url ?? null)
		: null}
	filename={lightboxIndex !== null
		? (gallery[lightboxIndex]?.title ?? `Image ${lightboxIndex + 1}`)
		: undefined}
	canPrev={lightboxIndex !== null && lightboxIndex > 0}
	canNext={lightboxIndex !== null && lightboxIndex < gallery.length - 1}
	onClose={closeLightbox}
	onPrev={prevLightbox}
	onNext={nextLightbox}
/>

<!-- Description Modal -->
<DescriptionModal
	open={descriptionExpanded}
	title={`${name} — Description`}
	html={renderMarkdown(body, description)}
	onClose={() => (descriptionExpanded = false)}
/>
