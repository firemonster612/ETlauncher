<script lang="ts">
	import { onMount } from 'svelte';
	import { SvelteSet } from 'svelte/reactivity';
	import { listen } from '@tauri-apps/api/event';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import { marked } from 'marked';
	import {
		Package,
		Search,
		Download,
		ExternalLink,
		Loader2,
		X,
		ChevronDown,
		ChevronLeft,
		Check,
		AlertTriangle,
		CheckCircle,
		Trash2,
		Power,
		PowerOff,
		SquareCheck,
		Square,
		Minus,
		Maximize2,
	} from '@lucide/svelte';
	import { Button } from '$lib/ui/button';
	import { Input } from '$lib/ui/input';
	import * as Select from '$lib/ui/select';
	import { Skeleton } from '$lib/ui/skeleton';
	import { contentStore } from '$lib/stores/content.svelte';
	import * as contentService from '$lib/services/content';
	import { getErrorMessage } from '$lib/utils/error';
	import ScreenshotLightbox from '$lib/components/ScreenshotLightbox.svelte';
	import DescriptionModal from '$lib/components/DescriptionModal.svelte';
	import type {
		Content,
		ContentDownloadProgressWithId,
		ContentType,
		ContentSortBy,
		ContentPlatform,
		LoaderType,
		ContentVersion,
		DetectedMod,
		QueueItemStatus,
		ScanResult,
	} from '$lib/types';

	interface Props {
		instanceId: string;
		instanceName: string;
		mcVersion: string;
		loaderType: LoaderType;
		onClose: () => void;
	}

	let { instanceId, instanceName, mcVersion, loaderType, onClose }: Props = $props();

	let searchInput = $state('');
	let selectedContentDetail = $state<Content | null>(null);
	let isLoadingDetail = $state(false);
	let detailError = $state<string | null>(null);
	let detailTab = $state<'about' | 'gallery'>('about');
	let galleryLightboxIndex = $state<number | null>(null);
	let descriptionExpanded = $state(false);
	let currentGallery = $derived(selectedContentDetail?.gallery ?? []);
	let detailRequestId = $state(0);

	// Navigation history for dependency drilling
	let contentHistory = $state<Content[]>([]);

	// View mode: "browse" for searching online, "installed" for local files
	let viewMode = $state<'browse' | 'installed'>('browse');

	// Search/filter for installed tab
	let installedSearchInput = $state('');

	// Multi-select state for installed tab
	let selectedItems = new SvelteSet<string>();

	// Derived: installed items list from scan result
	const installedItems = $derived(contentStore.scanResult?.items ?? []);

	const visibleInstalledItems = $derived(
		installedSearchInput.trim()
			? installedItems.filter((item) => {
					const query = installedSearchInput.trim().toLowerCase();
					const displayName = getItemDisplayName(item).toLowerCase();
					const filename = item.filename.toLowerCase();
					const mrSlug = item.modrinthProject?.slug?.toLowerCase() ?? '';
					const mrProjectId = item.modrinthProject?.projectId?.toLowerCase() ?? '';
					const cfName = item.curseforgeProject?.name?.toLowerCase() ?? '';
					const cfProjectId = String(item.curseforgeProject?.projectId ?? '').toLowerCase();

					return (
						displayName.includes(query) ||
						filename.includes(query) ||
						mrSlug.includes(query) ||
						mrProjectId.includes(query) ||
						cfName.includes(query) ||
						cfProjectId.includes(query)
					);
				})
			: installedItems
	);

	// Derived: check if all items are selected
	const allSelected = $derived(
		visibleInstalledItems.length > 0 &&
			visibleInstalledItems.filter((item) => selectedItems.has(item.filename)).length ===
				visibleInstalledItems.length
	);

	// Derived: check if some items are selected
	const someSelected = $derived(
		visibleInstalledItems.filter((item) => selectedItems.has(item.filename)).length > 0 &&
			visibleInstalledItems.filter((item) => selectedItems.has(item.filename)).length <
				visibleInstalledItems.length
	);

	// Derived: selected enabled items (for disable action)
	const selectedEnabledItems = $derived(
		installedItems.filter((item) => selectedItems.has(item.filename) && !item.isDisabled)
	);

	// Derived: selected disabled items (for enable action)
	const selectedDisabledItems = $derived(
		installedItems.filter((item) => selectedItems.has(item.filename) && item.isDisabled)
	);

	// Cached mod scan (used for helper detection so we don't flicker between tabs)
	let modScanResult = $state<ScanResult | null>(null);
	let isLoadingHelperScan = $state(false);

	// Check if current content type is blocked for vanilla instances
	// Mods and shaders require a mod loader; resource packs work on vanilla
	const isBlockedForVanilla = $derived(
		loaderType === 'vanilla' &&
			(contentStore.contentType === 'mod' || contentStore.contentType === 'shader')
	);

	// Helper detection for required companion mods (Fabric API / Iris / Oculus)
	// Uses exact slug matching to avoid false positives from addon mods
	function hasHelperInstalled(slugs: string[], filenamePatterns: RegExp[]): boolean {
		const items = modScanResult?.items ?? [];
		return items.some((item) => {
			const lowerFilename = item.filename.toLowerCase();
			const mrSlug = item.modrinthProject?.slug?.toLowerCase() ?? '';
			const cfSlug = item.curseforgeProject?.slug?.toLowerCase() ?? '';

			// Check exact slug match (Modrinth or CurseForge)
			const slugMatch = slugs.some((slug) => mrSlug === slug || cfSlug === slug);
			if (slugMatch) return true;

			// Check filename patterns for mods without API metadata
			return filenamePatterns.some((pattern) => pattern.test(lowerFilename));
		});
	}

	// Patterns: match "modname-" at start or "modname-mc" patterns (e.g., "iris-1.6.4.jar", "oculus-mc1.20.1-1.6.15a.jar")
	const hasFabricApi = $derived(hasHelperInstalled(['fabric-api'], [/^fabric-api-/]));
	const hasIris = $derived(hasHelperInstalled(['iris'], [/^iris-\d/, /^iris-mc/]));
	const hasOculus = $derived(hasHelperInstalled(['oculus'], [/^oculus-\d/, /^oculus-mc/]));
	const hasOptifine = $derived(
		hasHelperInstalled(['optifine'], [/^optifine/i, /^preview_optifine/i])
	);
	const hasShaderLoader = $derived(hasIris || hasOculus || hasOptifine);

	const shouldWarnFabricApi = $derived(
		!isLoadingHelperScan &&
			!!modScanResult &&
			viewMode === 'browse' &&
			contentStore.contentType === 'mod' &&
			(loaderType === 'fabric' || loaderType === 'quilt') &&
			!hasFabricApi
	);

	const shouldWarnShaders = $derived(
		!isLoadingHelperScan &&
			!!modScanResult &&
			viewMode === 'browse' &&
			contentStore.contentType === 'shader' &&
			!hasShaderLoader
	);

	// Installation state
	let isInstalling = $state(false);
	let installSuccess = $state<string | null>(null);
	let installError = $state<string | null>(null);
	let quickInstallName = $state<string | null>(null);
	let showQuickInstallProgress = $state(false);
	let quickInstallError = $state<string | null>(null);

	// Helper auto-install state (Fabric API / Iris / Oculus / OptiFine)
	let isInstallingHelper = $state(false);
	let helperInstallTarget = $state<'fabric-api' | 'iris' | 'oculus' | 'optifine' | null>(null);
	let helperInstallError = $state<string | null>(null);
	let helperInstallErrorFor = $state<'fabric-api' | 'iris' | 'oculus' | 'optifine' | null>(null);

	// Shader loader availability check
	// For Forge: tracks whether Oculus or OptiFine should be suggested
	let suggestedForgeShaderLoader = $state<'oculus' | 'optifine' | null>(null);
	let isCheckingShaderLoader = $state(false);
	let shaderLoaderAvailable = $state<boolean | null>(null);

	// Uninstall state
	let isUninstalling = $state(false);
	let uninstallError = $state<string | null>(null);
	let quickUninstallContentId = $state<string | null>(null);

	// Bulk action state
	let isBulkActioning = $state(false);
	let bulkActionError = $state<string | null>(null);
	let showRemoveConfirm = $state(false);

	// Initialize store with instance context on mount (once only)
	onMount(() => {
		contentStore.setInstanceContext(instanceId, mcVersion, loaderType);
		contentStore.search();
		refreshModScan();

		// Listen for download progress events (with queue ID)
		const unlistenProgressPromise = listen<ContentDownloadProgressWithId>(
			'content_download_progress',
			(event) => {
				// Update queue item progress
				contentStore.updateQueueItemProgress(event.payload.queueId, {
					filename: event.payload.filename,
					downloadedBytes: event.payload.downloadedBytes,
					totalBytes: event.payload.totalBytes,
					progressPercent: event.payload.progressPercent,
				});
				// Also update legacy single progress for backwards compat
				contentStore.setDownloadProgress(event.payload);
			}
		);

		// Listen for queue status changes
		const unlistenQueueStatusPromise = listen<{
			queueId: string;
			contentId: string;
			contentName: string;
			contentType: ContentType;
			status: QueueItemStatus;
			error?: string;
		}>('content_queue_status', async (event) => {
			// Refresh installed content BEFORE updating status when download completes
			// This ensures the "installed" badge shows immediately instead of a gap
			if (event.payload.status === 'completed') {
				await contentStore.refreshInstalledContent();
				// Refresh mod scan for helper detection (Iris/Oculus/OptiFine) when mods complete
				if (event.payload.contentType === 'mod') {
					await refreshModScan();
				}
			}
			contentStore.updateQueueItemStatus(
				event.payload.queueId,
				event.payload.contentId,
				event.payload.contentName,
				event.payload.contentType,
				event.payload.status,
				event.payload.error
			);
		});

		// Listen for slot available events to trigger queue processing
		const unlistenSlotAvailablePromise = listen('content_queue_slot_available', () => {
			contentService.tryProcessContentQueue();
		});

		return () => {
			// Clean up event listeners
			unlistenProgressPromise.then((unlisten) => unlisten());
			unlistenQueueStatusPromise.then((unlisten) => unlisten());
			unlistenSlotAvailablePromise.then((unlisten) => unlisten());
			contentStore.reset();
		};
	});

	// Debounced search
	let searchTimeout: ReturnType<typeof setTimeout>;
	function handleSearchInput(value: string) {
		searchInput = value;
		clearTimeout(searchTimeout);
		searchTimeout = setTimeout(() => {
			contentStore.setQuery(value);
			contentStore.search();
		}, 300);
	}

	async function handleContentTypeChange(type: ContentType) {
		await contentStore.setContentType(type);
		// Clear selection when changing content type
		selectedItems.clear();
		showQuickInstallProgress = false;
		quickInstallName = null;
		quickInstallError = null;
		if (viewMode === 'browse') {
			contentStore.search();
		}
	}

	function handleViewModeChange(mode: 'browse' | 'installed') {
		viewMode = mode;
		selectedItems.clear();
		bulkActionError = null;
	}

	// exactSlugs: only match if the slug exactly equals one of these values (no partial matching)
	const helperConfigs: Record<
		'fabric-api' | 'iris' | 'oculus',
		{ query: string; contentType: ContentType; exactSlugs: string[] }
	> = {
		'fabric-api': { query: 'fabric api', contentType: 'mod', exactSlugs: ['fabric-api'] },
		iris: { query: 'iris shaders', contentType: 'mod', exactSlugs: ['iris'] },
		oculus: { query: 'oculus', contentType: 'mod', exactSlugs: ['oculus'] },
	};

	async function refreshModScan() {
		isLoadingHelperScan = true;
		try {
			modScanResult = await contentService.scanInstalledContent(instanceId, 'mod');
		} catch (e) {
			console.error('Failed to scan mods for helper detection:', e);
		} finally {
			isLoadingHelperScan = false;
		}
	}

	// Check if the suggested shader loader is available for this MC version/loader
	async function checkShaderLoaderAvailability() {
		// Only check for Forge/NeoForge since Iris (Fabric) is widely available
		if (loaderType !== 'forge' && loaderType !== 'neoforge') {
			shaderLoaderAvailable = true;
			suggestedForgeShaderLoader = null;
			return;
		}

		isCheckingShaderLoader = true;
		try {
			// First, check if Oculus is available (preferred for Forge)
			const config = helperConfigs['oculus'];
			const searchResult = await contentService.searchContent({
				query: config.query,
				contentType: config.contentType,
				mcVersion,
				loader: loaderType,
				page: 0,
				pageSize: 10,
			});

			// Use exact slug matching to avoid false positives from addon mods
			const match = searchResult.items.find((content) => {
				const slug = content.slug.toLowerCase();
				return config.exactSlugs.some((exactSlug) => slug === exactSlug);
			});

			if (match) {
				const versions = await contentService.getContentVersions(
					match.platform,
					match.id,
					mcVersion,
					loaderType
				);

				if (versions.length > 0) {
					// Oculus is available
					shaderLoaderAvailable = true;
					suggestedForgeShaderLoader = 'oculus';
					return;
				}
			}

			// Oculus not available, check OptiFine as fallback
			try {
				const optifineAvailable = await contentService.checkOptifineAvailable(mcVersion);
				if (optifineAvailable) {
					shaderLoaderAvailable = true;
					suggestedForgeShaderLoader = 'optifine';
					return;
				}
			} catch (optifineError) {
				// OptiFine check failed (e.g., site down)
				const errorMsg =
					optifineError instanceof Error ? optifineError.message : 'OptiFine check failed';
				console.error('OptiFine availability check failed:', errorMsg);
				// Store the error message to show to user
				helperInstallError = errorMsg;
				helperInstallErrorFor = 'optifine';
			}

			// Neither Oculus nor OptiFine available
			shaderLoaderAvailable = false;
			suggestedForgeShaderLoader = null;
		} catch (e) {
			console.error('Failed to check shader loader availability:', e);
			shaderLoaderAvailable = false;
			suggestedForgeShaderLoader = null;
		} finally {
			isCheckingShaderLoader = false;
		}
	}

	async function installHelper(helper: 'fabric-api' | 'iris' | 'oculus') {
		isInstallingHelper = true;
		helperInstallTarget = helper;
		helperInstallError = null;
		helperInstallErrorFor = null;

		try {
			const config = helperConfigs[helper];

			const searchResult = await contentService.searchContent({
				query: config.query,
				contentType: config.contentType,
				mcVersion,
				loader: loaderType === 'vanilla' ? undefined : loaderType,
				page: 0,
				pageSize: 10,
			});

			// Use exact slug matching to avoid installing wrong mods (e.g., addon mods)
			const match = searchResult.items.find((content) => {
				const slug = content.slug.toLowerCase();
				return config.exactSlugs.some((exactSlug) => slug === exactSlug);
			});

			if (!match) {
				if (helper === 'oculus') {
					throw new Error(
						'Oculus is not available for this Minecraft version. No compatible shader loader found.'
					);
				}
				throw new Error('Could not find a compatible helper mod.');
			}

			const versions = await contentService.getContentVersions(
				match.platform,
				match.id,
				mcVersion,
				loaderType === 'vanilla' ? undefined : loaderType
			);

			const version = versions[0];
			if (!version) {
				if (helper === 'oculus') {
					throw new Error(
						'Oculus is not available for this Minecraft version. No compatible shader loader found.'
					);
				}
				throw new Error('No compatible version found for this Minecraft version/loader.');
			}

			await contentService.installContentWithDependencies(
				instanceId,
				match.platform,
				match,
				version,
				mcVersion,
				loaderType === 'vanilla' ? undefined : loaderType
			);

			await contentStore.refreshInstalledContent();
			await refreshModScan();
		} catch (e: unknown) {
			helperInstallError = e instanceof Error ? e.message : 'Failed to install helper mod.';
			helperInstallErrorFor = helper;
		} finally {
			isInstallingHelper = false;
			helperInstallTarget = null;
		}
	}

	// Install the appropriate shader loader based on loader type
	async function installShaderLoader() {
		if (loaderType === 'forge' || loaderType === 'neoforge') {
			if (suggestedForgeShaderLoader === 'optifine') {
				// Use dedicated OptiFine installer
				isInstallingHelper = true;
				helperInstallTarget = 'optifine';
				helperInstallError = null;
				helperInstallErrorFor = null;

				try {
					await contentService.installOptifine(instanceId);
					await contentStore.refreshInstalledContent();
					await refreshModScan();
				} catch (e: unknown) {
					helperInstallError = e instanceof Error ? e.message : 'Failed to install OptiFine.';
					helperInstallErrorFor = 'optifine';
				} finally {
					isInstallingHelper = false;
					helperInstallTarget = null;
				}
			} else {
				// Install Oculus via the standard helper installer
				await installHelper('oculus');
			}
		} else {
			// Install Iris for Fabric/Quilt
			await installHelper('iris');
		}
	}

	// Keep mod scan cache updated when the mods tab scans
	$effect(() => {
		if (contentStore.contentType === 'mod' && contentStore.scanResult) {
			modScanResult = contentStore.scanResult;
		}
	});

	// Check shader loader availability when viewing shaders without one installed
	$effect(() => {
		if (
			contentStore.contentType === 'shader' &&
			!hasShaderLoader &&
			shaderLoaderAvailable === null &&
			!isCheckingShaderLoader
		) {
			checkShaderLoaderAvailability();
		}
	});

	function toggleItemSelection(filename: string) {
		if (selectedItems.has(filename)) {
			selectedItems.delete(filename);
		} else {
			selectedItems.add(filename);
		}
	}

	function toggleSelectAll() {
		if (visibleInstalledItems.length === 0) return;

		if (allSelected) {
			for (const item of visibleInstalledItems) {
				selectedItems.delete(item.filename);
			}
			return;
		}

		for (const item of visibleInstalledItems) {
			selectedItems.add(item.filename);
		}
	}

	async function handleDisableSelected() {
		if (selectedEnabledItems.length === 0) return;

		isBulkActioning = true;
		bulkActionError = null;

		try {
			const filenames = selectedEnabledItems.map((item) => item.filename);
			await contentService.disableContent(instanceId, filenames, contentStore.contentType);
			// Refresh scan results
			await contentStore.refreshInstalledContent();
			selectedItems.clear();
		} catch (e: unknown) {
			bulkActionError = e instanceof Error ? e.message : 'Failed to disable content';
		} finally {
			isBulkActioning = false;
		}
	}

	async function handleEnableSelected() {
		if (selectedDisabledItems.length === 0) return;

		isBulkActioning = true;
		bulkActionError = null;

		try {
			const filenames = selectedDisabledItems.map((item) => item.filename);
			await contentService.enableContent(instanceId, filenames, contentStore.contentType);
			// Refresh scan results
			await contentStore.refreshInstalledContent();
			selectedItems.clear();
		} catch (e: unknown) {
			bulkActionError = e instanceof Error ? e.message : 'Failed to enable content';
		} finally {
			isBulkActioning = false;
		}
	}

	async function handleRemoveSelected() {
		if (selectedItems.size === 0) return;

		isBulkActioning = true;
		bulkActionError = null;

		try {
			// Remove each selected item
			for (const filename of selectedItems) {
				await contentService.uninstallContentByFilename(
					instanceId,
					filename,
					contentStore.contentType
				);
			}
			// Refresh scan results
			await contentStore.refreshInstalledContent();
			selectedItems.clear();
			showRemoveConfirm = false;
		} catch (e: unknown) {
			bulkActionError = e instanceof Error ? e.message : 'Failed to remove content';
		} finally {
			isBulkActioning = false;
		}
	}

	function getItemDisplayName(item: DetectedMod): string {
		// Try to get a nice name from Modrinth or CurseForge data
		if (item.modrinthProject?.name) {
			return item.modrinthProject.name;
		}
		if (item.curseforgeProject?.name) {
			return item.curseforgeProject.name;
		}

		// Fall back to filename without extension (and strip common trailing version patterns)
		const base = item.filename.replace(/\.(jar|zip)$/i, '');
		return base
			.replace(/[-_]?v?\d+(?:\.\d+){1,3}([+._-].*)?$/i, '')
			.replace(/[-_]?mc\d+(?:\.\d+){1,3}([+._-].*)?$/i, '')
			.replace(/[-_]+$/, '');
	}

	function getItemVersion(item: DetectedMod): string | null {
		if (item.modrinthProject?.versionNumber) {
			return item.modrinthProject.versionNumber;
		}
		return null;
	}

	function handlePlatformChange(platform: ContentPlatform) {
		contentStore.setPlatform(platform);
		contentStore.search();
	}

	function handleSortChange(sort: ContentSortBy) {
		contentStore.setSortBy(sort);
		contentStore.search();
	}

	function formatDownloads(downloads: number): string {
		if (downloads >= 1_000_000) {
			return `${(downloads / 1_000_000).toFixed(1)}M`;
		}
		if (downloads >= 1_000) {
			return `${(downloads / 1_000).toFixed(1)}K`;
		}
		return downloads.toString();
	}

	function formatBytes(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
	}

	function getPlatformColor(platform: ContentPlatform): string {
		switch (platform) {
			case 'modrinth':
				return 'bg-green-500/20 text-green-500 border-green-500/50';
			case 'curseforge':
				return 'bg-orange-500/20 text-orange-500 border-orange-500/50';
			default:
				return 'bg-muted text-muted-foreground border-muted';
		}
	}

	function getLoaderColor(loader: LoaderType): string {
		switch (loader) {
			case 'fabric':
				return 'bg-amber-500/20 text-amber-500';
			case 'forge':
				return 'bg-orange-500/20 text-orange-500';
			case 'neoforge':
				return 'bg-red-500/20 text-red-500';
			case 'quilt':
				return 'bg-purple-500/20 text-purple-500';
			default:
				return 'bg-muted/50 text-muted-foreground';
		}
	}

	function renderDescription(body?: string | null, fallback?: string): string {
		const source = (body && body.trim()) || fallback || '';
		if (!source) return '';

		try {
			return marked.parse(source) as string;
		} catch (e) {
			console.error('Failed to render description', e);
			return source.replace(/\n/g, '<br/>');
		}
	}

	async function handleDescriptionLinkClick(e: MouseEvent) {
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

	async function handleContentClick(content: Content) {
		installSuccess = null;
		installError = null;
		uninstallError = null;
		detailTab = 'about';
		galleryLightboxIndex = null;
		descriptionExpanded = false;
		showQuickInstallProgress = false;
		quickInstallName = null;
		quickInstallError = null;
		isLoadingDetail = true;
		detailError = null;
		contentHistory = [];
		selectedContentDetail = content;
		const requestId = ++detailRequestId;

		try {
			const detailed = await contentStore.selectContent(content);
			if (requestId !== detailRequestId) return;
			selectedContentDetail = detailed || content;
		} catch (e) {
			if (requestId !== detailRequestId) return;
			detailError = e instanceof Error ? e.message : 'Failed to load content details';
			selectedContentDetail = content;
		} finally {
			if (requestId === detailRequestId) {
				isLoadingDetail = false;
			}
		}
	}

	function closeContentDetail() {
		selectedContentDetail = null;
		contentHistory = [];
		detailError = null;
		detailTab = 'about';
		galleryLightboxIndex = null;
		descriptionExpanded = false;
		isLoadingDetail = false;
		contentStore.clearSelection();
	}

	function handleVersionSelect(version: ContentVersion) {
		if (selectedContentDetail) {
			contentStore.setSelectedVersion(version, selectedContentDetail.platform);
		}
	}

	async function handleDependencyClick(content: Content) {
		// Push current content to history
		if (selectedContentDetail) {
			contentHistory = [...contentHistory, selectedContentDetail];
		}
		detailTab = 'about';
		galleryLightboxIndex = null;
		descriptionExpanded = false;
		isLoadingDetail = true;
		detailError = null;
		selectedContentDetail = content;
		const requestId = ++detailRequestId;
		try {
			const detailed = await contentStore.selectContent(content);
			if (requestId !== detailRequestId) return;
			selectedContentDetail = detailed || content;
		} catch (e) {
			if (requestId !== detailRequestId) return;
			detailError = e instanceof Error ? e.message : 'Failed to load dependency';
			selectedContentDetail = content;
		} finally {
			if (requestId === detailRequestId) {
				isLoadingDetail = false;
			}
		}
	}

	async function handleBackClick() {
		if (contentHistory.length === 0) return;

		const previous = contentHistory[contentHistory.length - 1];
		contentHistory = contentHistory.slice(0, -1);

		detailTab = 'about';
		galleryLightboxIndex = null;
		descriptionExpanded = false;
		isLoadingDetail = true;
		detailError = null;
		selectedContentDetail = previous;
		const requestId = ++detailRequestId;
		try {
			const detailed = await contentStore.selectContent(previous);
			if (requestId !== detailRequestId) return;
			selectedContentDetail = detailed || previous;
		} catch (e) {
			if (requestId !== detailRequestId) return;
			detailError = e instanceof Error ? e.message : 'Failed to load previous content';
			selectedContentDetail = previous;
		} finally {
			if (requestId === detailRequestId) {
				isLoadingDetail = false;
			}
		}
	}

	function openGalleryLightbox(index: number) {
		galleryLightboxIndex = index;
	}

	function closeGalleryLightbox() {
		galleryLightboxIndex = null;
	}

	function prevGalleryLightbox() {
		if (galleryLightboxIndex === null) return;
		galleryLightboxIndex = Math.max(0, galleryLightboxIndex - 1);
	}

	function nextGalleryLightbox() {
		if (galleryLightboxIndex === null) return;
		galleryLightboxIndex = Math.min(currentGallery.length - 1, galleryLightboxIndex + 1);
	}

	async function handleInstall() {
		if (!selectedContentDetail || !contentStore.selectedVersion) return;

		isInstalling = true;
		installError = null;
		installSuccess = null;

		try {
			console.log(
				'[ContentBrowser] Queueing install:',
				selectedContentDetail.name,
				contentStore.selectedVersion.versionNumber
			);

			// Queue install with dependencies (non-blocking, returns queue IDs)
			const queueIds = await contentService.installContentWithDependencies(
				instanceId,
				selectedContentDetail.platform,
				selectedContentDetail,
				contentStore.selectedVersion,
				mcVersion,
				loaderType === 'vanilla' ? undefined : loaderType
			);

			console.log('[ContentBrowser] Queued', queueIds.length, 'items for installation');

			// Show success message briefly before closing
			installSuccess =
				queueIds.length > 1
					? `Queued ${selectedContentDetail.name} and ${queueIds.length - 1} dependencies`
					: `Queued ${selectedContentDetail.name} for installation`;

			// Auto-close after brief delay to show feedback
			setTimeout(() => {
				closeContentDetail();
				installSuccess = null;
			}, 1200);
		} catch (e: unknown) {
			console.error('[ContentBrowser] Queue failed:', e);
			installError = e instanceof Error ? e.message : String(e);
		} finally {
			isInstalling = false;
		}
	}

	async function handleUninstall() {
		if (!selectedContentDetail) return;

		const contentId = selectedContentDetail.id;
		const platform = selectedContentDetail.platform;

		// Find the matching installed item from scan result to get the filename
		// This must use the same matching logic as isContentInstalled (including slug fallback)
		const normalizedContentSlug = contentStore.normalizeSlug(selectedContentDetail.slug);
		let installedItem = contentStore.scanResult?.items.find((item) => {
			if (platform === 'modrinth') {
				// Direct match by Modrinth project ID
				if (item.modrinthProject?.projectId === contentId) return true;
				// Fallback: match by CurseForge slug
				if (
					item.curseforgeProject?.slug &&
					contentStore.normalizeSlug(item.curseforgeProject.slug) === normalizedContentSlug
				)
					return true;
				return false;
			} else if (platform === 'curseforge') {
				// Direct match by CurseForge project ID
				const contentIdNum = parseInt(contentId, 10);
				if (!isNaN(contentIdNum) && item.curseforgeProject?.projectId === contentIdNum) return true;
				// Fallback: match by Modrinth slug
				if (
					item.modrinthProject?.slug &&
					contentStore.normalizeSlug(item.modrinthProject.slug) === normalizedContentSlug
				)
					return true;
				return false;
			}
			return false;
		});

		if (!installedItem) {
			uninstallError = 'Could not find the installed file for this content';
			return;
		}

		isUninstalling = true;
		uninstallError = null;

		try {
			console.log(
				'[ContentBrowser] Uninstalling:',
				selectedContentDetail.name,
				'file:',
				installedItem.filename
			);

			await contentService.uninstallContentByFilename(
				instanceId,
				installedItem.filename,
				selectedContentDetail.contentType
			);

			console.log('[ContentBrowser] Uninstall complete');

			// Refresh installed content by re-scanning
			await contentStore.refreshInstalledContent();

			// Close the detail modal
			closeContentDetail();
		} catch (e: unknown) {
			console.error('[ContentBrowser] Uninstall failed:', e);
			uninstallError = e instanceof Error ? e.message : String(e);
		} finally {
			isUninstalling = false;
		}
	}

	const contentTypes: { value: ContentType; label: string }[] = [
		{ value: 'mod', label: 'Mods' },
		{ value: 'shader', label: 'Shaders' },
		{ value: 'resourcepack', label: 'Resource Packs' },
	];

	const platforms: { value: ContentPlatform; label: string }[] = [
		{ value: 'modrinth', label: 'Modrinth' },
		{ value: 'curseforge', label: 'CurseForge' },
	];

	const sortOptions: { value: ContentSortBy; label: string }[] = [
		{ value: 'downloads', label: 'Downloads' },
		{ value: 'recentlyUpdated', label: 'Recently Updated' },
		{ value: 'relevance', label: 'Relevance' },
	];

	const LOAD_MORE_THRESHOLD_PX = 300;

	function handleBrowseScroll(event: Event) {
		if (viewMode !== 'browse' || !contentStore.hasMore || contentStore.isSearching) return;

		const target = event.currentTarget as HTMLElement;
		const remaining = target.scrollHeight - target.scrollTop - target.clientHeight;
		if (remaining < LOAD_MORE_THRESHOLD_PX) {
			contentStore.loadMore();
		}
	}

	async function handleQuickInstall(content: Content) {
		// Don't install if already installed or queued
		if (contentStore.isContentInstalled(content)) return;
		if (contentStore.isContentQueued(content.id)) return;

		try {
			// Fetch the latest compatible version
			const shouldFilterByLoader = content.contentType === 'mod';
			const latestVersions = await contentService.getContentVersions(
				content.platform,
				content.id,
				mcVersion,
				shouldFilterByLoader && loaderType !== 'vanilla' ? loaderType : undefined
			);
			const latest = latestVersions[0];
			if (!latest) {
				throw new Error('No compatible versions found');
			}

			// Queue the install with dependency resolution (non-blocking)
			// Uses the same unified queue system as the modal install
			await contentService.installContentWithDependencies(
				instanceId,
				content.platform,
				content,
				latest,
				mcVersion,
				loaderType === 'vanilla' ? undefined : loaderType
			);
		} catch (e: unknown) {
			console.error('[ContentBrowser] Quick install failed:', e);
			// Show error inline for this content
			quickInstallError = getErrorMessage(e);
		}
	}

	async function handleQuickUninstall(content: Content) {
		if (isUninstalling) return;
		if (!contentStore.isContentInstalled(content)) return;

		// Find the matching installed item from scan result to get the filename
		// This must use the same matching logic as isContentInstalled (including slug fallback)
		const normalizedContentSlug = contentStore.normalizeSlug(content.slug);
		const installedItem = contentStore.scanResult?.items.find((item) => {
			if (content.platform === 'modrinth') {
				// Direct match by Modrinth project ID
				if (item.modrinthProject?.projectId === content.id) return true;
				// Fallback: match by CurseForge slug
				if (
					item.curseforgeProject?.slug &&
					contentStore.normalizeSlug(item.curseforgeProject.slug) === normalizedContentSlug
				)
					return true;
				return false;
			} else if (content.platform === 'curseforge') {
				// Direct match by CurseForge project ID
				const contentIdNum = parseInt(content.id, 10);
				if (!isNaN(contentIdNum) && item.curseforgeProject?.projectId === contentIdNum) return true;
				// Fallback: match by Modrinth slug
				if (
					item.modrinthProject?.slug &&
					contentStore.normalizeSlug(item.modrinthProject.slug) === normalizedContentSlug
				)
					return true;
				return false;
			}
			return false;
		});

		if (!installedItem) {
			console.error('[ContentBrowser] Could not find installed file for:', content.name);
			return;
		}

		isUninstalling = true;
		quickUninstallContentId = content.id;

		try {
			await contentService.uninstallContentByFilename(
				instanceId,
				installedItem.filename,
				content.contentType
			);
			await contentStore.refreshInstalledContent();
		} catch (e: unknown) {
			console.error('[ContentBrowser] Quick uninstall failed:', e);
		} finally {
			isUninstalling = false;
			quickUninstallContentId = null;
		}
	}

	function dismissQuickInstallProgress() {
		showQuickInstallProgress = false;
		quickInstallName = null;
		quickInstallError = null;
		contentStore.setDownloadProgress(null);
	}
</script>

<!-- Backdrop -->
<button
	type="button"
	class="fixed inset-x-0 top-[var(--titlebar-height)] z-50 h-[calc(100vh-var(--titlebar-height))] bg-black/50"
	onclick={onClose}
	aria-label={`Close content browser for ${instanceName}`}
></button>

<!-- Panel -->
<div
	class="bg-card border-border fixed inset-x-0 top-[var(--titlebar-height)] z-50 flex h-[calc(100vh-var(--titlebar-height))] w-full max-w-none flex-col overflow-hidden border-l-2 shadow-2xl"
>
	<!-- Close Button -->
	<button
		class="text-muted-foreground hover:text-foreground absolute top-2 right-2 z-10 p-2"
		onclick={onClose}
	>
		<X class="h-5 w-5" />
	</button>

	<!-- Browse / Installed Tabs -->
	<div class="border-border flex gap-1 border-b px-4 pt-2">
		<button
			class="relative px-4 py-2 text-sm font-medium transition-colors {viewMode === 'browse'
				? 'text-primary'
				: 'text-muted-foreground hover:text-foreground'}"
			onclick={() => handleViewModeChange('browse')}
		>
			Browse
			{#if viewMode === 'browse'}
				<div class="bg-primary absolute right-0 bottom-0 left-0 h-0.5"></div>
			{/if}
		</button>
		<button
			class="relative flex items-center gap-2 px-4 py-2 text-sm font-medium transition-colors {viewMode ===
			'installed'
				? 'text-primary'
				: 'text-muted-foreground hover:text-foreground'}"
			onclick={() => handleViewModeChange('installed')}
		>
			Installed
			{#if installedItems.length > 0}
				<span class="bg-muted rounded-full px-1.5 py-0.5 text-xs">
					{installedItems.length}
				</span>
			{/if}
			{#if viewMode === 'installed'}
				<div class="bg-primary absolute right-0 bottom-0 left-0 h-0.5"></div>
			{/if}
		</button>
	</div>

	<!-- Vanilla Instance Warning (only show in browse mode) -->
	{#if isBlockedForVanilla && viewMode === 'browse'}
		<div
			class="mx-4 mt-4 flex items-center gap-2 border-2 border-amber-500/50 bg-amber-500/10 p-3 text-sm text-amber-500"
		>
			<AlertTriangle class="h-4 w-4 flex-shrink-0" />
			<span>
				{contentStore.contentType === 'mod' ? 'Mods' : 'Shaders'} require a mod loader (Fabric, Forge,
				etc.). This is a vanilla instance. Switch to Resource Packs or add a mod loader to this instance.
			</span>
		</div>
	{/if}

	<!-- Fabric API warning -->
	{#if shouldWarnFabricApi}
		<div
			class="mx-4 mt-3 flex items-center gap-3 border-2 border-amber-500/50 bg-amber-500/10 p-3 text-sm text-amber-500"
		>
			<AlertTriangle class="h-4 w-4 flex-shrink-0" />
			<div class="flex-1">
				Fabric and Quilt mods usually need <span class="font-medium">Fabric API</span>. Install it
				before adding other mods.
			</div>
			<Button
				size="sm"
				variant="secondary"
				class="border-amber-500/50 text-amber-500"
				onclick={() => installHelper('fabric-api')}
				disabled={isInstallingHelper}
			>
				{#if isInstallingHelper && helperInstallTarget === 'fabric-api'}
					<Loader2 class="mr-2 h-4 w-4 animate-spin" />
					Installing...
				{:else}
					<Download class="mr-2 h-4 w-4" />
					Install Fabric API
				{/if}
			</Button>
		</div>
		{#if helperInstallError && helperInstallErrorFor === 'fabric-api'}
			<div class="text-destructive mx-4 mt-2 text-xs">
				{helperInstallError}
			</div>
		{/if}
	{/if}

	<!-- Shader loader warning -->
	{#if shouldWarnShaders}
		{#if isCheckingShaderLoader || ((loaderType === 'forge' || loaderType === 'neoforge') && shaderLoaderAvailable === null)}
			<!-- Checking availability (only show for Forge since we need to verify Oculus/OptiFine exists) -->
			<div
				class="bg-muted/50 border-border text-muted-foreground mx-4 mt-3 flex items-center gap-3 border-2 p-3 text-sm"
			>
				<Loader2 class="h-4 w-4 flex-shrink-0 animate-spin" />
				<div class="flex-1">Checking shader loader availability...</div>
			</div>
		{:else if shaderLoaderAvailable === false}
			<!-- No shader loader available for this version (Forge without Oculus/OptiFine support) -->
			<div
				class="bg-destructive/10 border-destructive/50 text-destructive mx-4 mt-3 flex items-center gap-3 border-2 p-3 text-sm"
			>
				<AlertTriangle class="h-4 w-4 flex-shrink-0" />
				<div class="flex-1">No shader loader available for Forge on this Minecraft version.</div>
			</div>
			{#if helperInstallError && helperInstallErrorFor === 'optifine'}
				<div class="text-destructive mx-4 mt-2 text-xs">
					{helperInstallError}
				</div>
			{/if}
		{:else}
			<!-- Shader loader available, show install option -->
			<div
				class="mx-4 mt-3 flex items-center gap-3 border-2 border-amber-500/50 bg-amber-500/10 p-3 text-sm text-amber-500"
			>
				<AlertTriangle class="h-4 w-4 flex-shrink-0" />
				<div class="flex-1">
					{#if loaderType === 'forge' || loaderType === 'neoforge'}
						Shaders need a shader loader like <span class="font-medium"
							>{suggestedForgeShaderLoader === 'optifine' ? 'OptiFine' : 'Oculus'}</span
						>. Install it to enable shaderpacks.
					{:else}
						Shaders need a shader loader like <span class="font-medium">Iris</span>. Install it to
						enable shaderpacks.
					{/if}
				</div>
				<Button
					size="sm"
					variant="secondary"
					class="border-amber-500/50 text-amber-500"
					onclick={() => installShaderLoader()}
					disabled={isInstallingHelper}
				>
					{#if isInstallingHelper && (helperInstallTarget === 'iris' || helperInstallTarget === 'oculus' || helperInstallTarget === 'optifine')}
						<Loader2 class="mr-2 h-4 w-4 animate-spin" />
						Installing...
					{:else}
						<Download class="mr-2 h-4 w-4" />
						{#if loaderType === 'forge' || loaderType === 'neoforge'}
							Install {suggestedForgeShaderLoader === 'optifine' ? 'OptiFine' : 'Oculus'}
						{:else}
							Install Iris
						{/if}
					{/if}
				</Button>
			</div>
			{#if helperInstallError && (helperInstallErrorFor === 'iris' || helperInstallErrorFor === 'oculus' || helperInstallErrorFor === 'optifine')}
				<div class="text-destructive mx-4 mt-2 text-xs">
					{helperInstallError}
				</div>
			{/if}
		{/if}
	{/if}

	{#if viewMode === 'browse'}
		<!-- Search and Filters -->
		<div class="border-border space-y-3 border-b p-4">
			<div class="flex gap-2">
				<div class="relative flex-1">
					<Search class="text-muted-foreground absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2" />
					<Input
						type="text"
						placeholder="Search {contentStore.contentType}s..."
						value={searchInput}
						oninput={(e) => handleSearchInput(e.currentTarget.value)}
						class="pl-9"
					/>
				</div>
			</div>
			<div class="flex flex-wrap gap-2">
				{#each platforms as { value, label } (value)}
					<Button
						variant={contentStore.platform === value ? 'default' : 'outline'}
						size="sm"
						onclick={() => handlePlatformChange(value)}
					>
						{label}
					</Button>
				{/each}
				<div class="ml-auto">
					<Select.Root
						type="single"
						value={contentStore.sortBy}
						onValueChange={(v) => handleSortChange(v as ContentSortBy)}
					>
						<Select.Trigger class="border-border bg-background h-8 border-2 text-xs">
							Sort: {sortOptions.find((o) => o.value === contentStore.sortBy)?.label}
						</Select.Trigger>
						<Select.Content class="border-border bg-card border-2">
							{#each sortOptions as { value, label } (value)}
								<Select.Item {value} {label}>{label}</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				</div>
			</div>
			<!-- Content Type Buttons -->
			<div class="flex gap-2" data-tutorial="content-browser-types">
				{#each contentTypes as { value, label } (value)}
					<Button
						variant={contentStore.contentType === value ? 'default' : 'secondary'}
						size="sm"
						onclick={() => handleContentTypeChange(value)}
					>
						{label}
					</Button>
				{/each}
			</div>
		</div>

		<!-- Error Display -->
		{#if contentStore.searchError}
			<div
				class="bg-destructive/10 border-destructive text-destructive mx-4 mt-4 border-2 p-3 text-sm"
			>
				{contentStore.searchError}
				<button class="ml-2 underline" onclick={() => contentStore.clearSearchError()}>
					Dismiss
				</button>
			</div>
		{/if}

		<!-- Scanning Indicator -->
		{#if contentStore.isScanning}
			<div
				class="bg-primary/10 border-primary/30 text-primary mx-4 mt-4 flex items-center gap-2 border-2 p-3 text-sm"
			>
				<Loader2 class="h-4 w-4 animate-spin" />
				Scanning installed mods...
			</div>
		{/if}

		<!-- Results -->
		<div class="flex-1 overflow-y-auto p-4" onscroll={handleBrowseScroll}>
			{#if contentStore.isSearching && contentStore.items.length === 0}
				<div class="flex items-center justify-center py-12">
					<Loader2 class="text-muted-foreground h-8 w-8 animate-spin" />
				</div>
			{:else if contentStore.items.length === 0}
				<div class="border-border bg-card/50 border-2 border-dashed p-12 text-center">
					<Package class="text-muted-foreground/50 mx-auto h-12 w-12" />
					<p class="text-muted-foreground mt-4 text-sm">
						{searchInput ? 'No results found' : `Search for ${contentStore.contentType}s`}
					</p>
				</div>
			{:else}
				<div class="text-muted-foreground mb-3 text-xs">
					{contentStore.totalCount} results
				</div>

				<div class="space-y-2">
					{#each contentStore.items as content (content.id)}
						{@const isInstalled = contentStore.isContentInstalled(content)}
						{@const isQueued = contentStore.isContentQueued(content.id)}
						{@const isDownloading = contentStore.isContentDownloading(content.id)}
						<div
							class="border-border bg-background hover:border-primary/50 relative w-full border-2 p-3 transition-colors"
						>
							<button
								type="button"
								class="flex w-full min-w-0 gap-3 text-left"
								onclick={() => handleContentClick(content)}
							>
								{#if content.iconUrl}
									<img
										src={content.iconUrl}
										alt={content.name}
										class="h-12 w-12 flex-shrink-0 rounded object-cover"
									/>
								{:else}
									<div
										class="bg-muted flex h-12 w-12 flex-shrink-0 items-center justify-center rounded"
									>
										<Package class="text-muted-foreground/50 h-6 w-6" />
									</div>
								{/if}
								<div class="min-w-0 flex-1 pr-16">
									<div class="flex min-w-0 items-center gap-2">
										<h3 class="truncate font-medium">{content.name}</h3>
										{#if isInstalled}
											<span
												class="flex flex-shrink-0 items-center gap-1 rounded bg-green-500/20 px-1.5 py-0.5 text-xs text-green-500"
											>
												<CheckCircle class="h-3 w-3" />
												Installed
											</span>
										{:else if isDownloading}
											<span
												class="flex flex-shrink-0 items-center gap-1 rounded bg-yellow-500/20 px-1.5 py-0.5 text-xs text-yellow-500"
											>
												<Loader2 class="h-3 w-3 animate-spin" />
												Installing
											</span>
										{:else if isQueued}
											<span
												class="flex flex-shrink-0 items-center gap-1 rounded bg-yellow-500/20 px-1.5 py-0.5 text-xs text-yellow-500"
											>
												<Loader2 class="h-3 w-3 animate-spin" />
												Pending
											</span>
										{/if}
									</div>
									<p class="text-muted-foreground truncate text-xs">{content.author}</p>
									<p class="text-muted-foreground mt-1 line-clamp-1 text-xs">
										{content.description}
									</p>
									<div class="text-muted-foreground mt-1 flex items-center gap-3 text-xs">
										<span class="flex items-center gap-1">
											<Download class="h-3 w-3" />
											{formatDownloads(content.downloads)}
										</span>
										{#each content.loaders
											.filter((l) => l !== 'unknown')
											.slice(0, 2) as loader (loader)}
											<span class="rounded px-1 {getLoaderColor(loader)}">{loader}</span>
										{/each}
									</div>
								</div>
							</button>

							<span
								class="pointer-events-none absolute top-3 right-3 rounded border px-1 py-0.5 text-[10px] {getPlatformColor(
									content.platform
								)}"
							>
								{content.platform}
							</span>

							{#if isInstalled}
								{@const isThisUninstalling =
									isUninstalling && quickUninstallContentId === content.id}
								<button
									class="text-muted-foreground hover:text-destructive hover:bg-destructive/10 absolute right-2 bottom-2 flex items-center gap-1 px-2 py-1 text-[10px] transition-colors disabled:cursor-not-allowed disabled:opacity-50"
									onclick={(e) => {
										e.stopPropagation();
										handleQuickUninstall(content);
									}}
									disabled={isUninstalling}
								>
									{#if isThisUninstalling}
										<Loader2 class="h-3 w-3 animate-spin" />
										Removing...
									{:else}
										<Trash2 class="h-3 w-3" />
										Uninstall
									{/if}
								</button>
							{:else if !isQueued}
								<button
									class="text-muted-foreground hover:text-primary hover:bg-primary/10 absolute right-2 bottom-2 flex items-center gap-1 px-2 py-1 text-[10px] transition-colors disabled:cursor-not-allowed disabled:opacity-50"
									onclick={(e) => {
										e.stopPropagation();
										handleQuickInstall(content);
									}}
									disabled={contentStore.isScanning}
								>
									<Download class="h-3 w-3" />
									Install
								</button>
							{/if}
						</div>
					{/each}
				</div>

				<!-- Load More -->
				{#if contentStore.hasMore}
					<div class="mt-4 flex justify-center">
						<Button
							variant="outline"
							size="sm"
							onclick={() => contentStore.loadMore()}
							disabled={contentStore.isSearching}
						>
							{#if contentStore.isSearching}
								<Loader2 class="mr-2 h-4 w-4 animate-spin" />
							{:else}
								<ChevronDown class="mr-2 h-4 w-4" />
							{/if}
							Load More
						</Button>
					</div>
				{/if}
			{/if}
		</div>
	{:else}
		<!-- Installed Tab Content -->

		<!-- Bulk Action Error -->
		{#if bulkActionError}
			<div
				class="bg-destructive/10 border-destructive text-destructive mx-4 mt-4 border-2 p-3 text-sm"
			>
				{bulkActionError}
				<button class="ml-2 underline" onclick={() => (bulkActionError = null)}> Dismiss </button>
			</div>
		{/if}

		<!-- Scanning Indicator -->
		{#if contentStore.isScanning}
			<div
				class="bg-primary/10 border-primary/30 text-primary mx-4 mt-4 flex items-center gap-2 border-2 p-3 text-sm"
			>
				<Loader2 class="h-4 w-4 animate-spin" />
				Scanning installed {contentStore.contentType}s...
			</div>
		{/if}

		<!-- Installed Items List -->
		<div class="flex-1 overflow-y-auto p-4">
			{#if installedItems.length === 0 && !contentStore.isScanning}
				<div class="border-border bg-card/50 border-2 border-dashed p-12 text-center">
					<Package class="text-muted-foreground/50 mx-auto h-12 w-12" />
					<p class="text-muted-foreground mt-4 text-sm">
						No {contentStore.contentType}s installed
					</p>
					<Button
						variant="outline"
						size="sm"
						class="mt-4"
						onclick={() => handleViewModeChange('browse')}
					>
						Browse {contentStore.contentType}s
					</Button>
				</div>
			{:else}
				<!-- Search -->
				<div class="mb-3">
					<div class="flex gap-2">
						<div class="relative flex-1">
							<Search
								class="text-muted-foreground absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2"
							/>
							<Input
								value={installedSearchInput}
								oninput={(e) => (installedSearchInput = e.currentTarget.value)}
								placeholder={`Search installed ${contentStore.contentType}s...`}
								class="pl-9"
							/>
						</div>
						{#if installedSearchInput.trim()}
							<Button variant="outline" size="sm" onclick={() => (installedSearchInput = '')}>
								Clear
							</Button>
						{/if}
					</div>
				</div>

				<!-- Select All Header -->
				<div class="border-border mb-3 flex items-center justify-between border-b pb-2">
					<button
						class="text-muted-foreground hover:text-foreground flex items-center gap-2 text-sm"
						onclick={toggleSelectAll}
					>
						{#if allSelected}
							<SquareCheck class="text-primary h-4 w-4" />
						{:else if someSelected}
							<div class="relative">
								<Square class="h-4 w-4" />
								<Minus
									class="absolute top-1/2 left-1/2 h-2 w-2 -translate-x-1/2 -translate-y-1/2"
								/>
							</div>
						{:else}
							<Square class="h-4 w-4" />
						{/if}
						Select All
					</button>
					<div class="text-muted-foreground text-xs">
						{#if installedSearchInput.trim()}
							{visibleInstalledItems.length} of {installedItems.length} items
						{:else}
							{installedItems.length} items
						{/if}
						{#if visibleInstalledItems.filter((i) => i.isDisabled).length > 0}
							&bull; {visibleInstalledItems.filter((i) => i.isDisabled).length} disabled
						{/if}
					</div>
				</div>

				<!-- Items List -->
				<div class="space-y-2">
					{#if visibleInstalledItems.length === 0 && installedSearchInput.trim() && !contentStore.isScanning}
						<div class="border-border bg-card/50 border-2 border-dashed p-8 text-center">
							<p class="text-muted-foreground text-sm">
								No matches for "{installedSearchInput.trim()}"
							</p>
							<Button
								variant="outline"
								size="sm"
								class="mt-3"
								onclick={() => (installedSearchInput = '')}
							>
								Clear search
							</Button>
						</div>
					{/if}

					{#each visibleInstalledItems as item (item.filename)}
						{@const isSelected = selectedItems.has(item.filename)}
						<button
							class="flex w-full gap-3 border-2 p-3 text-left transition-colors {isSelected
								? 'border-primary bg-primary/5'
								: 'border-border bg-background hover:border-primary/50'} {item.isDisabled
								? 'opacity-60'
								: ''}"
							onclick={() => toggleItemSelection(item.filename)}
						>
							<!-- Checkbox -->
							<div class="flex-shrink-0 pt-0.5">
								{#if isSelected}
									<SquareCheck class="text-primary h-5 w-5" />
								{:else}
									<Square class="text-muted-foreground h-5 w-5" />
								{/if}
							</div>

							<!-- Content -->
							<div class="min-w-0 flex-1">
								<div class="flex items-start justify-between gap-2">
									<div class="flex min-w-0 items-center gap-2">
										<h3 class="truncate font-medium">{getItemDisplayName(item)}</h3>
										{#if item.isDisabled}
											<span
												class="flex flex-shrink-0 items-center gap-1 rounded bg-amber-500/20 px-1.5 py-0.5 text-xs text-amber-500"
											>
												<PowerOff class="h-3 w-3" />
												Disabled
											</span>
										{/if}
									</div>
									<div class="flex flex-shrink-0 items-center gap-1">
										{#if item.modrinthProject}
											<span
												class="rounded border border-green-500/50 bg-green-500/20 px-1.5 py-0.5 text-xs text-green-500"
											>
												modrinth
											</span>
										{:else if item.curseforgeProject}
											<span
												class="rounded border border-orange-500/50 bg-orange-500/20 px-1.5 py-0.5 text-xs text-orange-500"
											>
												curseforge
											</span>
										{:else}
											<span
												class="bg-muted text-muted-foreground border-muted rounded border px-1.5 py-0.5 text-xs"
											>
												unknown
											</span>
										{/if}
									</div>
								</div>
								<p class="text-muted-foreground mt-0.5 truncate text-xs">{item.filename}</p>
								<div class="text-muted-foreground mt-1 flex items-center gap-3 text-xs">
									{#if getItemVersion(item)}
										<span>v{getItemVersion(item)}</span>
									{/if}
									<span>{formatBytes(item.size)}</span>
								</div>
							</div>
						</button>
					{/each}
				</div>
			{/if}
		</div>

		<!-- Action Bar (shown when items are selected) -->
		{#if selectedItems.size > 0}
			<div class="border-border bg-background border-t p-4">
				<div class="mb-2 flex items-center justify-between">
					<span class="text-muted-foreground text-sm">
						{selectedItems.size} item{selectedItems.size === 1 ? '' : 's'} selected
					</span>
					<button
						class="text-muted-foreground hover:text-foreground text-xs"
						onclick={() => selectedItems.clear()}
					>
						Clear selection
					</button>
				</div>
				<div class="flex gap-2">
					{#if selectedEnabledItems.length > 0}
						<Button
							variant="secondary"
							size="sm"
							class="flex-1"
							disabled={isBulkActioning}
							onclick={handleDisableSelected}
						>
							{#if isBulkActioning}
								<Loader2 class="mr-2 h-4 w-4 animate-spin" />
							{:else}
								<PowerOff class="mr-2 h-4 w-4" />
							{/if}
							Disable ({selectedEnabledItems.length})
						</Button>
					{/if}
					{#if selectedDisabledItems.length > 0}
						<Button
							variant="secondary"
							size="sm"
							class="flex-1"
							disabled={isBulkActioning}
							onclick={handleEnableSelected}
						>
							{#if isBulkActioning}
								<Loader2 class="mr-2 h-4 w-4 animate-spin" />
							{:else}
								<Power class="mr-2 h-4 w-4" />
							{/if}
							Enable ({selectedDisabledItems.length})
						</Button>
					{/if}
					<Button
						variant="destructive"
						size="sm"
						class="flex-1"
						disabled={isBulkActioning}
						onclick={() => (showRemoveConfirm = true)}
					>
						<Trash2 class="mr-2 h-4 w-4" />
						Remove ({selectedItems.size})
					</Button>
				</div>
			</div>
		{/if}

		{#if showQuickInstallProgress}
			<div class="pointer-events-none absolute right-4 bottom-4 left-4 z-50">
				<div class="bg-card border-border pointer-events-auto rounded-lg border-2 p-4 shadow-lg">
					<div class="flex items-start justify-between gap-3">
						<div class="min-w-0">
							<div class="truncate text-sm font-medium">
								{#if quickInstallName}
									Installing {quickInstallName}
								{:else}
									Installing...
								{/if}
							</div>
							{#if quickInstallError}
								<div class="text-destructive mt-1 text-xs">{quickInstallError}</div>
							{:else if contentStore.downloadProgress}
								<div class="text-muted-foreground mt-1 truncate text-xs">
									Downloading {contentStore.downloadProgress.filename}
								</div>
							{:else}
								<div class="text-muted-foreground mt-1 text-xs">Preparing...</div>
							{/if}
						</div>
						<Button
							variant="secondary"
							size="icon"
							onclick={dismissQuickInstallProgress}
							aria-label="Dismiss"
						>
							<X class="h-4 w-4" />
						</Button>
					</div>

					{#if contentStore.downloadProgress}
						<div class="bg-primary/5 border-primary/30 mt-3 rounded border-2 p-3">
							<div class="mb-2 flex items-center justify-between text-sm">
								<span class="text-muted-foreground mr-2 flex-1 truncate">
									{contentStore.downloadProgress.filename}
								</span>
								<span class="text-primary flex-shrink-0 font-medium">
									{contentStore.downloadProgress.progressPercent}%
								</span>
							</div>
							<div class="bg-muted h-2 overflow-hidden rounded-full">
								<div
									class="bg-primary h-full transition-all duration-150 ease-out"
									style="width: {contentStore.downloadProgress.progressPercent}%"
								></div>
							</div>
							<div class="text-muted-foreground mt-1 text-xs">
								{formatBytes(contentStore.downloadProgress.downloadedBytes)} / {formatBytes(
									contentStore.downloadProgress.totalBytes
								)}
							</div>
						</div>
					{/if}
				</div>
			</div>
		{/if}
	{/if}
</div>

<!-- Remove Confirmation Modal -->
{#if showRemoveConfirm}
	<div
		class="fixed inset-x-0 top-[var(--titlebar-height)] z-[60] flex h-[calc(100vh-var(--titlebar-height))] items-center justify-center bg-black/50 p-4"
	>
		<div class="bg-card border-border w-full max-w-sm border-2 p-4">
			<h3 class="mb-2 font-bold">
				Remove {selectedItems.size} item{selectedItems.size === 1 ? '' : 's'}?
			</h3>
			<p class="text-muted-foreground mb-4 text-sm">
				This will permanently delete the selected files. This action cannot be undone.
			</p>
			<div class="flex gap-2">
				<Button
					variant="outline"
					class="flex-1"
					onclick={() => (showRemoveConfirm = false)}
					disabled={isBulkActioning}
				>
					Cancel
				</Button>
				<Button
					variant="destructive"
					class="flex-1"
					onclick={handleRemoveSelected}
					disabled={isBulkActioning}
				>
					{#if isBulkActioning}
						<Loader2 class="mr-2 h-4 w-4 animate-spin" />
						Removing...
					{:else}
						Remove
					{/if}
				</Button>
			</div>
		</div>
	</div>
{/if}

<!-- Content Detail Modal -->
{#if selectedContentDetail}
	<div
		class="fixed inset-x-0 top-[var(--titlebar-height)] z-[60] flex h-[calc(100vh-var(--titlebar-height))] items-center justify-center bg-black/50 p-4"
	>
		<div
			class="bg-card border-border flex max-h-[90vh] w-full max-w-6xl flex-col overflow-hidden rounded-lg border-2 shadow-2xl"
		>
			<!-- Header -->
			<div class="border-border border-b p-5">
				<div class="flex items-start gap-4">
					{#if contentHistory.length > 0}
						<button
							class="text-muted-foreground hover:text-foreground mr-1 -ml-1 flex-shrink-0 self-center"
							onclick={handleBackClick}
							title="Back to {contentHistory[contentHistory.length - 1]?.name}"
						>
							<ChevronLeft class="h-6 w-6" />
						</button>
					{/if}
					{#if selectedContentDetail.iconUrl}
						<img
							src={selectedContentDetail.iconUrl}
							alt={selectedContentDetail.name}
							class="h-16 w-16 rounded object-cover"
						/>
					{:else}
						<div class="bg-muted flex h-16 w-16 items-center justify-center rounded">
							<Package class="text-muted-foreground/50 h-8 w-8" />
						</div>
					{/if}
					<div class="min-w-0 flex-1 space-y-1">
						<div class="flex items-start justify-between gap-2">
							<h2 class="truncate font-bold">{selectedContentDetail.name}</h2>
							<button
								class="text-muted-foreground hover:text-foreground flex-shrink-0"
								onclick={closeContentDetail}
							>
								<X class="h-5 w-5" />
							</button>
						</div>
						<p class="text-muted-foreground text-sm">{selectedContentDetail.author}</p>
						<div class="mt-1 flex flex-wrap items-center gap-2">
							<span
								class="rounded border px-1.5 py-0.5 text-xs {getPlatformColor(
									selectedContentDetail.platform
								)}"
							>
								{selectedContentDetail.platform}
							</span>
							<span class="text-muted-foreground flex items-center gap-1 text-xs">
								<Download class="h-3 w-3" />
								{formatDownloads(selectedContentDetail.downloads)}
							</span>
							{#if contentStore.isContentInstalled(selectedContentDetail)}
								<span
									class="flex items-center gap-1 rounded bg-green-500/20 px-1.5 py-0.5 text-xs text-green-500"
								>
									<CheckCircle class="h-3 w-3" />
									Already Installed
								</span>
							{:else if contentStore.isContentDownloading(selectedContentDetail.id)}
								<span
									class="flex items-center gap-1 rounded bg-yellow-500/20 px-1.5 py-0.5 text-xs text-yellow-500"
								>
									<Loader2 class="h-3 w-3 animate-spin" />
									Installing
								</span>
							{:else if contentStore.isContentQueued(selectedContentDetail.id)}
								<span
									class="flex items-center gap-1 rounded bg-yellow-500/20 px-1.5 py-0.5 text-xs text-yellow-500"
								>
									<Loader2 class="h-3 w-3 animate-spin" />
									Pending
								</span>
							{/if}
						</div>
						<p class="text-muted-foreground line-clamp-3 text-sm">
							{selectedContentDetail.description}
						</p>
					</div>
				</div>
			</div>

			<div
				class="grid min-h-0 flex-1 grid-cols-1 gap-4 overflow-hidden p-5 md:grid-cols-[2.5fr_1fr] xl:grid-cols-[3fr_1fr]"
			>
				<div class="min-h-0 space-y-4 overflow-y-auto pr-1">
					{#if detailError}
						<div
							class="bg-destructive/10 border-destructive text-destructive rounded border-2 p-3 text-sm"
						>
							{detailError}
						</div>
					{/if}

					<div class="flex items-center gap-2">
						<Button
							size="sm"
							variant={detailTab === 'about' ? 'default' : 'secondary'}
							onclick={() => (detailTab = 'about')}
							disabled={isLoadingDetail}
						>
							About
						</Button>
						<Button
							size="sm"
							variant={detailTab === 'gallery' ? 'default' : 'secondary'}
							disabled={isLoadingDetail || (selectedContentDetail.gallery?.length ?? 0) === 0}
							onclick={() => (detailTab = 'gallery')}
						>
							Gallery
						</Button>
					</div>

					{#if isLoadingDetail}
						<div class="border-border bg-background/70 space-y-3 rounded-lg border-2 p-4">
							<div class="flex items-center justify-between gap-2">
								<Skeleton class="h-4 w-20" />
								<Skeleton class="h-8 w-28" />
							</div>
							<div class="space-y-2">
								<Skeleton class="h-4 w-full" />
								<Skeleton class="h-4 w-[92%]" />
								<Skeleton class="h-4 w-[84%]" />
								<Skeleton class="h-4 w-[88%]" />
								<Skeleton class="h-4 w-[70%]" />
							</div>
						</div>
					{:else if detailTab === 'gallery'}
						{#if (selectedContentDetail.gallery?.length ?? 0) > 0}
							<div class="space-y-2">
								<div class="flex items-center justify-between">
									<h3 class="text-sm font-semibold">Gallery</h3>
									<span class="text-muted-foreground text-xs">
										{selectedContentDetail.gallery?.length ?? 0} images
									</span>
								</div>
								<div class="grid gap-3 sm:grid-cols-2">
									{#each selectedContentDetail.gallery ?? [] as image, idx (image.rawUrl ?? image.url)}
										<button
											type="button"
											class="border-border bg-muted/50 relative aspect-video cursor-pointer overflow-hidden rounded-lg border-2 text-left"
											onclick={() => openGalleryLightbox(idx)}
										>
											<img
												src={image.rawUrl ?? image.url}
												alt={image.title ?? selectedContentDetail.name}
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
					{:else}
						<div class="border-border bg-background/70 space-y-2 rounded-lg border-2 p-4">
							<div class="flex items-center justify-between gap-2">
								<h3 class="text-sm font-semibold">About</h3>
								<div class="flex items-center gap-2">
									<Button
										size="sm"
										variant="secondary"
										onclick={() => (descriptionExpanded = true)}
										disabled={!selectedContentDetail.body && !selectedContentDetail.description}
									>
										<Maximize2 class="mr-1 h-4 w-4" />
										Expand
									</Button>
									{#if selectedContentDetail.url}
										<a
											href={selectedContentDetail.url}
											target="_blank"
											rel="noopener noreferrer"
											class="bg-muted hover:bg-muted/80 inline-flex items-center gap-1.5 rounded px-2.5 py-1.5 text-xs transition-colors"
										>
											<ExternalLink class="h-3.5 w-3.5" />
											View on {selectedContentDetail.platform}
										</a>
									{/if}
								</div>
							</div>
							{#if selectedContentDetail.body || selectedContentDetail.description}
								<!-- svelte-ignore a11y_click_events_have_key_events -->
								<!-- svelte-ignore a11y_no_static_element_interactions -->
								<div
									class="[&_a]:text-primary text-sm leading-relaxed [&_a]:underline [&_h1]:text-lg [&_h1]:font-semibold [&_h2]:text-base [&_h2]:font-semibold [&_img]:my-2 [&_img]:max-w-full [&_img]:rounded-md [&_ol]:list-decimal [&_ol]:pl-5 [&_p]:mb-3 [&_ul]:list-disc [&_ul]:pl-5"
									onclick={handleDescriptionLinkClick}
								>
									<!-- eslint-disable-next-line svelte/no-at-html-tags -->
									{@html renderDescription(
										selectedContentDetail.body || selectedContentDetail.description
									)}
								</div>
							{:else}
								<p class="text-muted-foreground text-sm">No description available.</p>
							{/if}
						</div>
					{/if}
				</div>

				<div class="min-h-0 space-y-3 overflow-y-auto pr-1">
					<div class="border-border bg-background/70 max-w-sm rounded-lg border-2 p-3">
						<h3 class="mb-2 text-sm font-semibold">Select Version</h3>
						{#if contentStore.isLoadingVersions}
							<div class="space-y-2">
								<div class="text-muted-foreground flex items-center gap-2 text-sm">
									<Loader2 class="h-4 w-4 animate-spin" />
									Loading versions...
								</div>
								<Skeleton class="h-9 w-full" />
								<Skeleton class="h-9 w-full" />
								<Skeleton class="h-9 w-full" />
							</div>
						{:else if contentStore.selectedContentVersions.length === 0}
							<p class="text-muted-foreground text-sm">No compatible versions found</p>
						{:else}
							<div class="space-y-1">
								{#each contentStore.selectedContentVersions.slice(0, 15) as version (version.id)}
									{@const isSelected = contentStore.selectedVersion?.id === version.id}
									<button
										class="w-full rounded border-2 p-2 text-left transition-colors {isSelected
											? 'border-primary bg-primary/10'
											: 'border-border hover:border-primary/50'}"
										onclick={() => handleVersionSelect(version)}
									>
										<div class="flex items-center gap-2">
											{#if isSelected}
												<Check class="text-primary h-4 w-4" />
											{/if}
											<span class="text-sm font-medium">{version.versionNumber}</span>
										</div>
										<div class="text-muted-foreground mt-0.5 text-xs">
											{version.mcVersions.slice(0, 3).join(', ')}
											{#if version.mcVersions.length > 3}+{version.mcVersions.length - 3} more{/if}
											{#if version.releasedAt}
												&bull; {new Date(version.releasedAt * 1000).toLocaleDateString()}
											{/if}
										</div>
									</button>
								{/each}
							</div>
						{/if}

						{#if selectedContentDetail && contentStore.selectedVersion}
							{@const hasRequiredDeps = contentStore.selectedVersion.dependencies.some(
								(d) => d.dependencyType === 'required'
							)}

							{#if hasRequiredDeps}
								{#if contentStore.isResolvingDeps}
									<div class="bg-muted/50 border-border mt-4 rounded border-2 p-3">
										<div class="text-muted-foreground flex items-center gap-2 text-sm">
											<Loader2 class="h-4 w-4 animate-spin" />
											Loading dependencies...
										</div>
									</div>
								{:else if contentStore.resolvedDependencies.length > 0}
									{@const allDepsInstalled = contentStore.resolvedDependencies.every(
										(d) => d.alreadyInstalled
									)}
									<div
										class="mt-4 rounded border-2 p-3 {allDepsInstalled
											? 'border-green-500/50 bg-green-500/10'
											: 'border-amber-500/50 bg-amber-500/10'}"
									>
										<div
											class="flex items-center gap-2 text-sm font-medium {allDepsInstalled
												? 'text-green-500'
												: 'text-amber-500'}"
										>
											{#if allDepsInstalled}
												<CheckCircle class="h-4 w-4" />
												Dependencies Installed
											{:else}
												<AlertTriangle class="h-4 w-4" />
												Required Dependencies
											{/if}
										</div>
										<p class="text-muted-foreground mt-1 text-xs">
											{#if allDepsInstalled}
												All required dependencies are already installed.
											{:else}
												This {selectedContentDetail.contentType} requires the following (will be auto-installed):
											{/if}
										</p>
										<ul class="mt-2 space-y-1 text-xs">
											{#each contentStore.resolvedDependencies as resolved (resolved.content.id)}
												<li class="flex items-center gap-2">
													{#if resolved.alreadyInstalled}
														<CheckCircle class="h-3 w-3 text-green-500" />
													{:else}
														<span class="h-1.5 w-1.5 rounded-full bg-amber-500"></span>
													{/if}
													<button
														type="button"
														class="text-primary hover:text-primary/80 flex items-center gap-1 text-left underline transition-colors {resolved.alreadyInstalled
															? 'text-green-500/80 hover:text-green-500'
															: ''}"
														onclick={() => handleDependencyClick(resolved.content)}
													>
														{resolved.content.name}
														<ExternalLink class="h-2.5 w-2.5" />
													</button>
													{#if resolved.alreadyInstalled}
														<span class="text-[10px] text-green-500">installed</span>
													{/if}
												</li>
											{/each}
										</ul>
									</div>
								{:else}
									{@const requiredDeps = contentStore.selectedVersion.dependencies.filter(
										(d) => d.dependencyType === 'required'
									)}
									<div class="mt-4 rounded border-2 border-amber-500/50 bg-amber-500/10 p-3">
										<div class="flex items-center gap-2 text-sm font-medium text-amber-500">
											<AlertTriangle class="h-4 w-4" />
											Required Dependencies
										</div>
										<ul class="mt-2 space-y-1 text-xs">
											{#each requiredDeps as dep (dep.id)}
												<li class="flex items-center gap-2">
													<span class="h-1.5 w-1.5 rounded-full bg-amber-500"></span>
													<span class="text-muted-foreground italic">{dep.id}</span>
												</li>
											{/each}
										</ul>
									</div>
								{/if}
							{/if}
						{/if}
					</div>
				</div>
			</div>

			<div class="border-border space-y-3 border-t p-4">
				{#if installSuccess}
					<div
						class="flex items-center gap-2 rounded border-2 border-green-500/50 bg-green-500/10 p-3 text-sm text-green-500"
					>
						<CheckCircle class="h-4 w-4 flex-shrink-0" />
						{installSuccess}
					</div>
				{/if}

				{#if installError}
					<div
						class="bg-destructive/10 border-destructive text-destructive rounded border-2 p-3 text-sm"
					>
						<div class="flex items-center gap-2 font-medium">
							<AlertTriangle class="h-4 w-4 flex-shrink-0" />
							Installation Failed
						</div>
						<p class="mt-1 text-xs opacity-80">{installError}</p>
					</div>
				{/if}

				{#if uninstallError}
					<div
						class="bg-destructive/10 border-destructive text-destructive rounded border-2 p-3 text-sm"
					>
						<div class="flex items-center gap-2 font-medium">
							<AlertTriangle class="h-4 w-4 flex-shrink-0" />
							Uninstall Failed
						</div>
						<p class="mt-1 text-xs opacity-80">{uninstallError}</p>
					</div>
				{/if}

				{#if contentStore.isContentDownloading(selectedContentDetail.id)}
					{@const progress = contentStore.getContentProgress(selectedContentDetail.id)}
					{#if progress}
						<div class="bg-primary/5 border-primary/30 rounded border-2 p-3">
							<div class="mb-2 flex items-center justify-between text-sm">
								<span class="text-muted-foreground mr-2 flex-1 truncate">
									Downloading {progress.filename}
								</span>
								<span class="text-primary flex-shrink-0 font-medium">
									{progress.progressPercent}%
								</span>
							</div>
							<div class="bg-muted h-2 overflow-hidden rounded-full">
								<div
									class="bg-primary h-full transition-all duration-150 ease-out"
									style="width: {progress.progressPercent}%"
								></div>
							</div>
							<div class="text-muted-foreground mt-1 text-xs">
								{formatBytes(progress.downloadedBytes)} / {formatBytes(progress.totalBytes)}
							</div>
						</div>
					{/if}
				{/if}

				<div class="flex gap-2">
					<Button variant="outline" class="flex-1" onclick={closeContentDetail}>Cancel</Button>
					{#if contentStore.isContentInstalled(selectedContentDetail)}
						<Button
							variant="destructive"
							class="flex-1"
							disabled={isUninstalling || contentStore.isScanning}
							onclick={handleUninstall}
						>
							{#if isUninstalling}
								<Loader2 class="mr-2 h-4 w-4 animate-spin" />
								Uninstalling...
							{:else}
								<Trash2 class="mr-2 h-4 w-4" />
								Uninstall
							{/if}
						</Button>
					{:else if contentStore.isContentDownloading(selectedContentDetail.id)}
						<Button class="flex-1" variant="secondary" disabled>
							<Loader2 class="mr-2 h-4 w-4 animate-spin" />
							Installing...
						</Button>
					{:else if contentStore.isContentQueued(selectedContentDetail.id)}
						<Button class="flex-1" variant="secondary" disabled>
							<Loader2 class="mr-2 h-4 w-4 animate-spin" />
							Pending...
						</Button>
					{:else}
						<Button
							class="flex-1"
							disabled={!contentStore.selectedVersion ||
								isInstalling ||
								!!installSuccess ||
								contentStore.isScanning ||
								isBlockedForVanilla}
							onclick={handleInstall}
						>
							{#if isInstalling}
								<Loader2 class="mr-2 h-4 w-4 animate-spin" />
								Installing...
							{:else if contentStore.isScanning}
								<Loader2 class="mr-2 h-4 w-4 animate-spin" />
								Scanning...
							{:else if isBlockedForVanilla}
								<AlertTriangle class="mr-2 h-4 w-4" />
								Requires Mod Loader
							{:else}
								<Download class="mr-2 h-4 w-4" />
								Install
							{/if}
						</Button>
					{/if}
				</div>
			</div>
		</div>
	</div>
{/if}

<ScreenshotLightbox
	open={galleryLightboxIndex !== null}
	src={galleryLightboxIndex !== null
		? (currentGallery[galleryLightboxIndex]?.rawUrl ??
			currentGallery[galleryLightboxIndex]?.url ??
			null)
		: null}
	filename={galleryLightboxIndex !== null
		? (currentGallery[galleryLightboxIndex]?.title ?? `Image ${galleryLightboxIndex + 1}`)
		: undefined}
	canPrev={galleryLightboxIndex !== null && galleryLightboxIndex > 0}
	canNext={galleryLightboxIndex !== null && galleryLightboxIndex < currentGallery.length - 1}
	onClose={closeGalleryLightbox}
	onPrev={prevGalleryLightbox}
	onNext={nextGalleryLightbox}
/>

<DescriptionModal
	open={descriptionExpanded && !!selectedContentDetail}
	title={selectedContentDetail ? `${selectedContentDetail.name} — Description` : 'Description'}
	html={selectedContentDetail
		? renderDescription(selectedContentDetail.body || selectedContentDetail.description)
		: ''}
	onClose={() => (descriptionExpanded = false)}
/>
