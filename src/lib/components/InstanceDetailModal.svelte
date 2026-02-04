<script lang="ts">
	import { X, Loader2, LayoutDashboard, Package, Image, Database, RefreshCw } from '@lucide/svelte';
	import { Button } from '$lib/ui/button';
	import { nestedScroll } from '$lib/utils/scroll';
	import ScreenshotLightbox from '$lib/components/ScreenshotLightbox.svelte';
	import InstanceDetailHero from '$lib/components/instance-detail/InstanceDetailHero.svelte';
	import InstanceDetailOverview from '$lib/components/instance-detail/InstanceDetailOverview.svelte';
	import InstanceDetailContent from '$lib/components/instance-detail/InstanceDetailContent.svelte';
	import InstanceDetailGallery from '$lib/components/instance-detail/InstanceDetailGallery.svelte';
	import InstanceDetailData from '$lib/components/instance-detail/InstanceDetailData.svelte';
	import InstanceDetailVersion from '$lib/components/instance-detail/InstanceDetailVersion.svelte';
	import BackgroundLayer from '$lib/components/BackgroundLayer.svelte';
	import * as instanceDetailService from '$lib/services/instance-detail';
	import * as instanceService from '$lib/services/instance';
	import * as contentService from '$lib/services/content';
	import * as updateService from '$lib/services/update';
	import { accountsStore } from '$lib/stores/accounts.svelte';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import type { Instance, InstanceDetail, ContentType } from '$lib/types';

	// Check if custom background is active
	const hasCustomBackground = $derived(
		settingsStore.settings?.background?.type && settingsStore.settings.background.type !== 'none'
	);

	type TabId = 'overview' | 'content' | 'version' | 'gallery' | 'data';

	interface Props {
		instance: Instance | null;
		open: boolean;
		status: string | null;
		onClose: () => void;
		onLaunch: (instanceId: string) => void;
		onKill: (instanceId: string) => void;
		onOpenSettings: (instance: Instance) => void;
		onOpenContentBrowser: (instance: Instance, contentType?: ContentType) => void;
		onInstanceUpdated?: (instance: Instance) => void;
	}

	let {
		instance,
		open,
		status,
		onClose,
		onLaunch,
		onKill,
		onOpenSettings,
		onOpenContentBrowser,
		onInstanceUpdated,
	}: Props = $props();

	// Tab state
	let activeTab = $state<TabId>('overview');

	// Data states
	let detail = $state<InstanceDetail | null>(null);
	let isLoading = $state(false);
	let error = $state<string | null>(null);
	let modCount = $state(0);
	let hasUpdate = $state(false);

	// Screenshot lightbox states (for overview tab filmstrip)
	let lightboxIndex = $state<number | null>(null);
	let lightboxData = $state<string | null>(null);
	let lightboxLoading = $state(false);
	let screenshotPreviews = $state<Record<string, string>>({});

	// Derived values
	const activeAccountId = $derived(accountsStore.activeAccount?.id ?? null);
	const activeAccountName = $derived(accountsStore.activeAccount?.username ?? null);
	const activeAccountSkinUrl = $derived(accountsStore.activeAccount?.skinUrl ?? null);

	const supportsQuickPlay = $derived(
		instance ? checkQuickPlaySupport(instance.minecraftVersion) : false
	);

	// Reset and load when modal opens
	$effect(() => {
		if (open && instance) {
			activeTab = 'overview';
			screenshotPreviews = {};
			detail = null;
			modCount = 0;
			hasUpdate = false;
			loadDetail();
			loadModCount();
			checkForUpdates();
		} else {
			detail = null;
			error = null;
			screenshotPreviews = {};
		}
	});

	async function loadDetail() {
		if (!instance) return;
		isLoading = true;
		error = null;

		try {
			detail = await instanceDetailService.getInstanceDetail(instance.id);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load instance details';
			console.error('Failed to load instance detail:', e);
		} finally {
			isLoading = false;
		}
	}

	async function loadModCount() {
		if (!instance) return;
		try {
			const scanResult = await contentService.scanInstalledContent(instance.id, 'mod');
			modCount = scanResult.items.length;
		} catch (e) {
			console.error('Failed to scan mods:', e);
		}
	}

	async function checkForUpdates() {
		if (!instance) return;
		try {
			if (instance.modpackPlatform) {
				const check = await updateService.checkModpackInstanceUpdates(instance.id);
				hasUpdate = check.hasUpdate;
			} else {
				const check = await updateService.checkInstanceUpdates(instance.id);
				hasUpdate = check.hasMcUpdate;
			}
		} catch (e) {
			// Silently fail update check
			console.error('Failed to check for updates:', e);
		}
	}

	function checkQuickPlaySupport(version: string): boolean {
		const parts = version
			.split('.')
			.map((p) => p.replace(/[^0-9].*$/, ''))
			.filter(Boolean)
			.map((p) => parseInt(p, 10))
			.filter((n) => !Number.isNaN(n));
		const major = parts[0] ?? 0;
		const minor = parts[1] ?? 0;
		return major > 1 || (major === 1 && minor >= 20);
	}

	// Screenshot preview and lightbox handling
	async function loadScreenshotPreview(filename: string) {
		if (!instance || screenshotPreviews[filename]) return;
		try {
			const data = await instanceDetailService.getScreenshotData(instance.id, filename);
			screenshotPreviews = {
				...screenshotPreviews,
				[filename]: `data:image/png;base64,${data}`,
			};
		} catch (e) {
			console.error('Failed to load screenshot preview:', e);
		}
	}

	async function openScreenshotLightbox(index: number) {
		if (!detail || !instance) return;
		lightboxIndex = index;
		await loadLightboxImage(index);
	}

	async function loadLightboxImage(index: number) {
		if (!detail || !instance) return;
		const shot = detail.recentScreenshots[index];
		if (!shot) return;

		lightboxLoading = true;
		try {
			const data = await instanceDetailService.getScreenshotData(instance.id, shot.filename);
			lightboxData = `data:image/png;base64,${data}`;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load screenshot';
			console.error('Failed to load screenshot data:', e);
		} finally {
			lightboxLoading = false;
		}
	}

	function closeScreenshotLightbox() {
		lightboxIndex = null;
		lightboxData = null;
	}

	function goPrevScreenshot() {
		if (!detail || lightboxIndex === null || lightboxIndex === 0) return;
		const nextIndex = lightboxIndex - 1;
		lightboxIndex = nextIndex;
		loadLightboxImage(nextIndex);
	}

	function goNextScreenshot() {
		if (!detail || lightboxIndex === null) return;
		const nextIndex = lightboxIndex + 1;
		if (nextIndex >= detail.recentScreenshots.length) return;
		lightboxIndex = nextIndex;
		loadLightboxImage(nextIndex);
	}

	const canPrevScreenshot = $derived(lightboxIndex !== null && lightboxIndex > 0);
	const canNextScreenshot = $derived(
		detail && lightboxIndex !== null && lightboxIndex < detail.recentScreenshots.length - 1
	);

	// Event handlers
	function handleKeydown(e: KeyboardEvent) {
		if (!open) return;
		if (e.key === 'Escape') {
			onClose();
		}
	}

	function handleOpenFolder() {
		if (instance) {
			instanceService.openInstanceFolder(instance.id);
		}
	}

	function handleNavigateToTab(tab: string) {
		activeTab = tab as TabId;
	}

	// Tab definitions
	const tabs: { id: TabId; label: string; icon: typeof LayoutDashboard }[] = [
		{ id: 'overview', label: 'Overview', icon: LayoutDashboard },
		{ id: 'content', label: 'Content', icon: Package },
		{ id: 'version', label: 'Version', icon: RefreshCw },
		{ id: 'gallery', label: 'Gallery', icon: Image },
		{ id: 'data', label: 'Worlds & Servers', icon: Database },
	];
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open && instance}
	<!-- Backdrop with custom background support -->
	<div
		class="fixed inset-x-0 top-[var(--titlebar-height)] z-50 h-[calc(100vh-var(--titlebar-height))]"
		style="overflow: clip;"
		data-fullscreen-backdrop
	>
		{#if hasCustomBackground}
			<!-- Render background layer inside modal when custom background is active -->
			<BackgroundLayer absolute />
		{:else}
			<div class="absolute inset-0 bg-black/50"></div>
		{/if}
		<button
			class="absolute inset-0"
			onclick={onClose}
			aria-label="Close instance detail"
		></button>
	</div>

	<!-- Full-page panel -->
	<div
		class="bg-card border-border fixed inset-x-0 top-[var(--titlebar-height)] z-50 flex h-[calc(100vh-var(--titlebar-height))] w-full max-w-none flex-col overflow-hidden border-l-2 shadow-2xl"
		data-modal-content
	>
		<!-- Close Button (absolute positioned) -->
		<Button
			variant="ghost"
			size="icon"
			class="absolute top-4 right-4 z-10"
			onclick={onClose}
			aria-label="Close"
		>
			<X class="h-5 w-5" />
		</Button>

		<!-- Hero Header -->
		<InstanceDetailHero
			{instance}
			{status}
			{activeAccountName}
			{activeAccountSkinUrl}
			totalPlayTime={detail?.totalPlayTime ?? 0}
			{hasUpdate}
			onLaunch={() => onLaunch(instance.id)}
			onKill={() => onKill(instance.id)}
			onOpenSettings={() => onOpenSettings(instance)}
			onOpenFolder={handleOpenFolder}
			onCheckUpdate={() => (activeTab = 'version')}
		/>

		<!-- Tab Navigation -->
		<div class="border-border flex gap-2 border-b px-6 py-3">
			{#each tabs as tab (tab.id)}
				{@const Icon = tab.icon}
				<Button
					size="sm"
					variant={activeTab === tab.id ? 'default' : 'secondary'}
					onclick={() => (activeTab = tab.id)}
				>
					<Icon class="mr-1.5 h-4 w-4" />
					{tab.label}
				</Button>
			{/each}
		</div>

		<!-- Tab Content -->
		<div class="flex-1 overflow-hidden">
			{#if isLoading}
				<div class="text-muted-foreground flex h-full items-center justify-center gap-3">
					<Loader2 class="h-6 w-6 animate-spin" />
					<span>Loading instance details...</span>
				</div>
			{:else if error && activeTab === 'overview'}
				<div class="p-6">
					<div
						class="border-destructive/60 bg-destructive/10 text-destructive rounded border px-4 py-3 text-sm"
					>
						{error}
					</div>
				</div>
			{:else}
				<div class="h-full overflow-y-auto p-6" use:nestedScroll>
					{#if activeTab === 'overview' && detail}
						<InstanceDetailOverview
							{instance}
							{detail}
							{modCount}
							{activeAccountId}
							{supportsQuickPlay}
							{screenshotPreviews}
							onLoadScreenshotPreview={loadScreenshotPreview}
							onOpenScreenshotLightbox={openScreenshotLightbox}
							onNavigateToTab={handleNavigateToTab}
						/>
					{:else if activeTab === 'content'}
						<InstanceDetailContent
							{instance}
							onOpenContentBrowser={(contentType) => {
								onOpenContentBrowser(instance, contentType);
							}}
						/>
					{:else if activeTab === 'version'}
						<InstanceDetailVersion
							{instance}
							onUpdated={(updatedInstance) => {
								// Refresh the detail and notify parent
								loadDetail();
								loadModCount();
								checkForUpdates();
								onInstanceUpdated?.(updatedInstance);
							}}
						/>
					{:else if activeTab === 'gallery'}
						<InstanceDetailGallery instanceId={instance.id} />
					{:else if activeTab === 'data'}
						<InstanceDetailData
							instanceId={instance.id}
							minecraftVersion={instance.minecraftVersion}
							{activeAccountId}
						/>
					{/if}
				</div>
			{/if}
		</div>
	</div>

	<!-- Screenshot Lightbox -->
	<ScreenshotLightbox
		open={lightboxIndex !== null}
		src={lightboxData}
		filename={lightboxIndex !== null && detail
			? detail.recentScreenshots[lightboxIndex]?.filename
			: undefined}
		isLoading={lightboxLoading}
		canPrev={!!canPrevScreenshot}
		canNext={!!canNextScreenshot}
		onClose={closeScreenshotLightbox}
		onPrev={goPrevScreenshot}
		onNext={goNextScreenshot}
	/>
{/if}
