<script lang="ts">
	import { Loader2, Search, Trash2, Calendar, Image } from '@lucide/svelte';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { alertDialogStore } from '$lib/stores/alertDialog.svelte';
	import { Button } from '$lib/ui/button';
	import { Input } from '$lib/ui/input';
	import * as Select from '$lib/ui/select';
	import ScreenshotLightbox from '$lib/components/ScreenshotLightbox.svelte';
	import * as instanceDetailService from '$lib/services/instance-detail';
	import type { DateFilter, Screenshot } from '$lib/types';

	interface Props {
		instanceId: string;
	}

	let { instanceId }: Props = $props();

	let screenshots = $state<Screenshot[]>([]);
	let isLoading = $state(false);
	let search = $state('');
	let dateFilter = $state<DateFilter>('all');
	let error = $state<string | null>(null);
	let previewSources = $state<Record<string, string>>({});

	let lightboxIndex = $state<number | null>(null);
	let lightboxData = $state<string | null>(null);
	let lightboxLoading = $state(false);

	let lastLoadedId = $state<string | null>(null);
	const screenshotDateFormatter = new Intl.DateTimeFormat(undefined, {
		year: 'numeric',
		month: 'short',
		day: 'numeric',
		hour: '2-digit',
		minute: '2-digit',
	});
	const formatDate = (timestamp: number) => screenshotDateFormatter.format(timestamp);

	$effect(() => {
		if (instanceId && instanceId !== lastLoadedId) {
			loadScreenshots();
		}
	});

	const filteredScreenshots = $derived(
		screenshots.filter((shot) => {
			const query = search.trim().toLowerCase();
			const matchesSearch =
				query.length === 0 ||
				shot.filename.toLowerCase().includes(query) ||
				formatDate(shot.takenAt).toLowerCase().includes(query);

			const matchesDate = (() => {
				if (dateFilter === 'all') return true;
				const now = Date.now();
				const msInDay = 24 * 60 * 60 * 1000;
				if (dateFilter === 'today') {
					const start = now - (now % msInDay);
					return shot.takenAt >= start;
				}
				if (dateFilter === 'week') {
					const weekAgo = now - 7 * 24 * 60 * 60 * 1000;
					return shot.takenAt >= weekAgo;
				}
				if (dateFilter === 'month') {
					const monthAgo = now - 30 * 24 * 60 * 60 * 1000;
					return shot.takenAt >= monthAgo;
				}
				return true;
			})();

			return matchesSearch && matchesDate;
		})
	);

	async function loadScreenshots() {
		if (!instanceId) return;
		isLoading = true;
		error = null;
		lightboxIndex = null;
		lightboxData = null;

		try {
			const response = await instanceDetailService.getInstanceScreenshots(instanceId);
			screenshots = response.screenshots;
			lastLoadedId = instanceId;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load screenshots';
			console.error('Failed to load screenshots:', e);
		} finally {
			isLoading = false;
		}
	}

	async function openLightbox(index: number) {
		lightboxIndex = index;
		await loadLightboxImage(index);
	}

	async function loadLightboxImage(index: number) {
		const shot = filteredScreenshots[index];
		if (!shot) return;

		lightboxLoading = true;
		try {
			const data = await instanceDetailService.getScreenshotData(instanceId, shot.filename);
			const dataUrl = `data:image/png;base64,${data}`;
			lightboxData = dataUrl;
			previewSources = { ...previewSources, [shot.filename]: dataUrl };
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load screenshot';
			console.error('Failed to load screenshot data:', e);
		} finally {
			lightboxLoading = false;
		}
	}

	function closeLightbox() {
		lightboxIndex = null;
		lightboxData = null;
	}

	async function loadPreview(shot: Screenshot) {
		if (previewSources[shot.filename]) return;
		try {
			const data = await instanceDetailService.getScreenshotData(instanceId, shot.filename);
			previewSources = {
				...previewSources,
				[shot.filename]: `data:image/png;base64,${data}`,
			};
		} catch (e) {
			console.error('Failed to load screenshot preview', e);
		}
	}

	function goPrev() {
		if (lightboxIndex === null || lightboxIndex === 0) return;
		const nextIndex = lightboxIndex - 1;
		lightboxIndex = nextIndex;
		loadLightboxImage(nextIndex);
	}

	function goNext() {
		if (lightboxIndex === null) return;
		const nextIndex = lightboxIndex + 1;
		if (nextIndex >= filteredScreenshots.length) return;
		lightboxIndex = nextIndex;
		loadLightboxImage(nextIndex);
	}

	const canPrev = $derived(lightboxIndex !== null && lightboxIndex > 0);
	const canNext = $derived(
		lightboxIndex !== null && lightboxIndex < filteredScreenshots.length - 1
	);

	async function handleDelete(shot: Screenshot, event: MouseEvent) {
		event.stopPropagation();

		const confirmed = await alertDialogStore.confirm({
			title: 'Delete Screenshot',
			message: `Delete screenshot "${shot.filename}"?`,
			type: 'warning',
			confirmText: 'Delete',
			cancelText: 'Cancel',
		});

		if (!confirmed) return;

		try {
			await instanceDetailService.deleteScreenshot(instanceId, shot.filename);
			screenshots = screenshots.filter((s) => s.filename !== shot.filename);
			if (
				lightboxIndex !== null &&
				filteredScreenshots[lightboxIndex]?.filename === shot.filename
			) {
				closeLightbox();
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to delete screenshot';
			console.error('Failed to delete screenshot:', e);
		}
	}
</script>

<div class="flex h-full flex-col gap-4">
	<!-- Header with Search and Filter -->
	<div class="flex flex-col gap-3 md:flex-row md:items-center">
		<div class="relative flex-1">
			<Input
				placeholder="Search screenshots..."
				value={search}
				oninput={(e) => (search = e.currentTarget.value)}
				class="pl-10"
			/>
			<Search class="text-muted-foreground absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2" />
		</div>

		<Select.Root
			type="single"
			value={dateFilter}
			onValueChange={(v) => (dateFilter = v as DateFilter)}
		>
			<Select.Trigger class="border-border bg-background h-10 min-w-[160px] border">
				<Calendar class="mr-2 h-4 w-4" />
				{{
					all: 'All dates',
					today: 'Today',
					week: 'This week',
					month: 'This month',
					custom: 'Custom',
				}[dateFilter]}
			</Select.Trigger>
			<Select.Content class="border-border bg-card border">
				<Select.Item value="all" label="All dates">All dates</Select.Item>
				<Select.Item value="today" label="Today">Today</Select.Item>
				<Select.Item value="week" label="This week">This week</Select.Item>
				<Select.Item value="month" label="This month">This month</Select.Item>
			</Select.Content>
		</Select.Root>
	</div>

	<!-- Error Display -->
	{#if error}
		<div
			class="border-destructive/60 bg-destructive/10 text-destructive rounded border px-4 py-3 text-sm"
		>
			{error}
		</div>
	{/if}

	<!-- Screenshots Grid -->
	<div class="flex-1 overflow-y-auto">
		{#if isLoading}
			<div class="text-muted-foreground flex items-center justify-center gap-3 py-12">
				<Loader2 class="h-6 w-6 animate-spin" />
				<span>Loading screenshots...</span>
			</div>
		{:else if filteredScreenshots.length === 0}
			<div class="text-muted-foreground flex flex-col items-center justify-center gap-3 py-12">
				<Image class="h-12 w-12 opacity-50" />
				{#if search || dateFilter !== 'all'}
					<p>No screenshots matching your filters</p>
				{:else}
					<p>No screenshots yet</p>
					<p class="text-sm">Press F2 in-game to take a screenshot</p>
				{/if}
			</div>
		{:else}
			<!-- Masonry Grid using CSS columns -->
			<div class="columns-1 gap-3 sm:columns-2 md:columns-3 lg:columns-4">
				{#each filteredScreenshots as shot, index (shot.filename)}
					<button
						class="border-border bg-muted/30 group relative mb-3 w-full break-inside-avoid overflow-hidden rounded-lg border"
						onclick={() => openLightbox(index)}
						title={shot.filename}
					>
						<img
							src={previewSources[shot.filename] ?? convertFileSrc(shot.path)}
							alt={shot.filename}
							class="w-full transition-transform group-hover:scale-[1.02]"
							loading="lazy"
							onerror={() => loadPreview(shot)}
						/>

						<!-- Hover Overlay -->
						<div
							class="absolute inset-0 flex flex-col justify-between bg-gradient-to-t from-black/70 via-transparent to-transparent p-3 opacity-0 transition-opacity group-hover:opacity-100"
						>
							<!-- Delete Button -->
							<div class="flex justify-end">
								<Button
									variant="destructive"
									size="icon"
									class="h-8 w-8"
									onclick={(e) => handleDelete(shot, e)}
									aria-label="Delete screenshot"
								>
									<Trash2 class="h-4 w-4" />
								</Button>
							</div>

							<!-- Info -->
							<div class="text-white">
								<p class="truncate text-sm font-medium">{shot.filename}</p>
								<p class="text-xs opacity-80">{formatDate(shot.takenAt)}</p>
							</div>
						</div>
					</button>
				{/each}
			</div>
		{/if}
	</div>

	<!-- Stats Footer -->
	{#if !isLoading && screenshots.length > 0}
		<div class="border-border text-muted-foreground border-t pt-3 text-sm">
			{#if search || dateFilter !== 'all'}
				Showing {filteredScreenshots.length} of {screenshots.length} screenshots
			{:else}
				{screenshots.length} screenshot{screenshots.length === 1 ? '' : 's'}
			{/if}
		</div>
	{/if}
</div>

<ScreenshotLightbox
	open={lightboxIndex !== null}
	src={lightboxData}
	filename={lightboxIndex !== null ? filteredScreenshots[lightboxIndex]?.filename : undefined}
	isLoading={lightboxLoading}
	{canPrev}
	{canNext}
	onClose={closeLightbox}
	onPrev={goPrev}
	onNext={goNext}
/>
