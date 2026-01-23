<script lang="ts">
	import {
		Play,
		Square,
		X,
		Settings,
		Trash2,
		Clock,
		Calendar,
		Loader2,
		PackagePlus,
		ChevronRight,
	} from '@lucide/svelte';
	import { Button } from '$lib/ui/button';
	import DownloadProgress from '$lib/components/DownloadProgress.svelte';
	import { parseIconPath, getIconUrl } from '$lib/utils/icons';
	import type { Instance, LoaderType, LaunchStatus, InstanceSetupStatus } from '$lib/types';

	interface Props {
		instance: Instance;
		status: string | null;
		launchStatus: LaunchStatus | undefined;
		setupStatus: InstanceSetupStatus | undefined;
		onLaunch: (instanceId: string) => void;
		onKill: (instanceId: string) => void;
		onOpenSettings: (instance: Instance) => void;
		onOpenContentBrowser: (instance: Instance) => void;
		onDelete: (instanceId: string) => void;
		onCardClick: (instance: Instance) => void;
	}

	let {
		instance,
		status,
		launchStatus,
		setupStatus,
		onLaunch,
		onKill,
		onOpenSettings,
		onOpenContentBrowser,
		onDelete,
		onCardClick,
	}: Props = $props();

	// Computed: check if instance is being set up
	const isSettingUp = $derived(
		setupStatus &&
			(setupStatus.status === 'pending' ||
				setupStatus.status === 'preparing' ||
				setupStatus.status === 'downloadingGameFiles' ||
				setupStatus.status === 'installingLoader')
	);

	let isPlayButtonHovered = $state(false);

	function formatPlayTime(seconds: number): string {
		if (seconds < 60) return '< 1 min';
		const hours = Math.floor(seconds / 3600);
		const minutes = Math.floor((seconds % 3600) / 60);
		if (hours === 0) return `${minutes}m`;
		return `${hours}h ${minutes}m`;
	}

	function formatDate(timestamp: number): string {
		return new Date(timestamp * 1000).toLocaleDateString();
	}

	function getLoaderColor(loader: LoaderType): string {
		switch (loader) {
			case 'fabric':
				return 'bg-amber-500/20 text-amber-500 border-amber-500/50';
			case 'forge':
				return 'bg-orange-500/20 text-orange-500 border-orange-500/50';
			case 'neoforge':
				return 'bg-red-500/20 text-red-500 border-red-500/50';
			case 'quilt':
				return 'bg-purple-500/20 text-purple-500 border-purple-500/50';
			default:
				return 'bg-green-500/20 text-green-500 border-green-500/50';
		}
	}

	function getIconSrc(iconPath: string | undefined): string {
		const icon = parseIconPath(iconPath);
		if (icon) {
			return getIconUrl(icon);
		}
		// Fallback to creeper
		return '/icons/entities/creeper/creeper.png';
	}

	function handleCardClick(e: MouseEvent) {
		// Don't trigger card click if clicking on a button
		if ((e.target as HTMLElement).closest('button')) return;
		onCardClick(instance);
	}

	function handlePlayButtonClick(e: MouseEvent) {
		e.stopPropagation();
		if (status === 'running' || status === 'windowReady') {
			onKill(instance.id);
		} else {
			onLaunch(instance.id);
		}
	}
</script>

<div
	class="border-border bg-card instance-card-glow group hover:border-primary/60 relative w-[320px] cursor-pointer border-2 transition-all duration-200"
	onclick={handleCardClick}
	onkeydown={(e) => e.key === 'Enter' && onCardClick(instance)}
	role="button"
	tabindex="0"
>
	<!-- Icon Banner Section -->
	<div class="from-muted/50 to-muted relative h-24 overflow-hidden bg-gradient-to-br">
		<!-- Blurred background icon -->
		<div class="absolute inset-0 flex items-center justify-center opacity-20 blur-xl">
			<img
				src={getIconSrc(instance.iconPath)}
				alt=""
				class="pixelated h-32 w-32"
				aria-hidden="true"
			/>
		</div>
		<!-- Main icon -->
		<div class="absolute inset-0 flex items-center justify-center">
			<img
				src={getIconSrc(instance.iconPath)}
				alt="{instance.name} icon"
				class="pixelated h-16 w-16 drop-shadow-lg"
			/>
		</div>
		<!-- Expand indicator - top right -->
		<div
			class="group-hover:bg-primary/30 absolute top-2 right-2 rounded bg-black/30 p-1 transition-all duration-200"
			title="View details"
		>
			<ChevronRight
				class="h-4 w-4 text-white/70 transition-all duration-200 group-hover:translate-x-0.5 group-hover:text-white"
			/>
		</div>
	</div>

	<!-- Info Section -->
	<div class="p-4">
		<!-- Title - 2 lines max, min-height reserves space for 2 lines to keep cards aligned -->
		<h3 class="line-clamp-2 min-h-[2.8rem] text-lg leading-tight font-bold" title={instance.name}>
			{instance.name}
		</h3>

		<!-- Version and Loader -->
		<div class="mt-2 flex flex-wrap items-center gap-x-2 gap-y-1">
			<span class="text-muted-foreground text-sm">{instance.minecraftVersion}</span>
			<span
				class="max-w-[140px] truncate rounded border px-1.5 py-0.5 text-xs whitespace-nowrap capitalize {getLoaderColor(
					instance.loaderType
				)}"
				title="{instance.loaderType}{instance.loaderVersion ? ` ${instance.loaderVersion}` : ''}"
			>
				{instance.loaderType}
				{#if instance.loaderVersion}
					<span class="ml-1 opacity-75">{instance.loaderVersion}</span>
				{/if}
			</span>
		</div>

		<!-- Play time and date -->
		<div class="text-muted-foreground mt-2 flex flex-wrap items-center gap-x-4 gap-y-1 text-xs">
			<span class="flex items-center gap-1 whitespace-nowrap">
				<Clock class="h-3 w-3 flex-shrink-0" />
				{formatPlayTime(instance.totalPlayTime)}
			</span>
			<span class="flex items-center gap-1 whitespace-nowrap">
				<Calendar class="h-3 w-3 flex-shrink-0" />
				{formatDate(instance.createdAt)}
			</span>
		</div>
	</div>

	<!-- Bottom bar with play button and actions -->
	<div class="border-border flex items-center justify-between gap-2 border-t px-4 py-3">
		<!-- Play button -->
		<Button
			variant="default"
			size="sm"
			onclick={handlePlayButtonClick}
			onmouseenter={() => (isPlayButtonHovered = true)}
			onmouseleave={() => (isPlayButtonHovered = false)}
			disabled={status !== null &&
				status !== 'stopped' &&
				status !== 'crashed' &&
				status !== 'running' &&
				status !== 'windowReady'}
		>
			{#if status === 'checkingAccount' || status === 'refreshingToken' || status === 'loadingVersion' || status === 'verifyingFiles' || status === 'downloading' || status === 'checkingJava' || status === 'downloadingJava' || status === 'buildingClasspath' || status === 'launching' || status === 'running'}
				<Loader2 class="mr-2 h-4 w-4 animate-spin" />
				Launching
			{:else if status === 'windowReady'}
				{#if isPlayButtonHovered}
					<X class="mr-2 h-4 w-4" />
				{:else}
					<Square class="mr-2 h-4 w-4" />
				{/if}
				Running
			{:else}
				<Play class="mr-2 h-4 w-4" />
				Play
			{/if}
		</Button>

		<!-- Action buttons -->
		<div class="flex items-center gap-1">
			<Button
				variant="ghost"
				size="sm"
				onclick={(e: MouseEvent) => {
					e.stopPropagation();
					onOpenSettings(instance);
				}}
				title="Instance settings"
				data-tutorial="instance-settings-btn"
			>
				<Settings class="h-4 w-4" />
			</Button>
			<Button
				variant="ghost"
				size="sm"
				onclick={(e: MouseEvent) => {
					e.stopPropagation();
					onOpenContentBrowser(instance);
				}}
				title="Add mods, shaders, resource packs"
				data-tutorial="content-browser-btn"
			>
				<PackagePlus class="h-4 w-4" />
			</Button>
			<Button
				variant="ghost"
				size="sm"
				class="text-destructive hover:text-destructive hover:bg-destructive/10"
				onclick={(e: MouseEvent) => {
					e.stopPropagation();
					onDelete(instance.id);
				}}
				title="Delete instance"
			>
				<Trash2 class="h-4 w-4" />
			</Button>
		</div>
	</div>

	<!-- Download Progress (during launch) -->
	{#if launchStatus?.status === 'downloading'}
		{@const progress = launchStatus.progress}
		<div class="border-border border-t px-4 py-3">
			<DownloadProgress
				stage="Downloading game files"
				progress={progress.totalBytes > 0
					? (progress.downloadedBytes / progress.totalBytes) * 100
					: 0}
				currentItem={progress.currentFile}
				totalBytes={progress.totalBytes}
				downloadedBytes={progress.downloadedBytes}
				compact
			/>
		</div>
	{/if}

	<!-- Setup Progress (during instance creation) -->
	{#if isSettingUp}
		<div class="border-border border-t px-4 py-3">
			{#if setupStatus?.status === 'pending' || setupStatus?.status === 'preparing'}
				<div class="text-muted-foreground flex items-center gap-2 text-sm">
					<Loader2 class="h-4 w-4 animate-spin" />
					<span>
						{setupStatus?.status === 'preparing' && 'message' in setupStatus
							? setupStatus.message
							: 'Setting up instance...'}
					</span>
				</div>
			{:else if setupStatus?.status === 'downloadingGameFiles'}
				{@const progress = setupStatus.progress}
				<DownloadProgress
					stage="Setting up game files"
					progress={progress.totalBytes > 0
						? (progress.downloadedBytes / progress.totalBytes) * 100
						: 0}
					currentItem={progress.currentFile}
					totalBytes={progress.totalBytes}
					downloadedBytes={progress.downloadedBytes}
					compact
				/>
			{:else if setupStatus?.status === 'installingLoader'}
				<div class="text-muted-foreground flex items-center gap-2 text-sm">
					<Loader2 class="h-4 w-4 animate-spin" />
					<span>Installing {setupStatus.stage}... ({setupStatus.progress}%)</span>
				</div>
			{/if}
		</div>
	{/if}
</div>
