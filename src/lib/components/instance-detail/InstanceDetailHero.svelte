<script lang="ts">
	import {
		Play,
		Square,
		X,
		Settings,
		FolderOpen,
		Loader2,
		Clock,
		Calendar,
		ArrowUp,
		Package,
	} from '@lucide/svelte';
	import { Button } from '$lib/ui/button';
	import { parseIconPath, getIconUrl } from '$lib/utils/icons';
	import { getAvatarUrl } from '$lib/services/account';
	import type { Instance, LoaderType } from '$lib/types';

	interface Props {
		instance: Instance;
		status: string | null;
		activeAccountName: string | null;
		totalPlayTime: number;
		hasUpdate?: boolean;
		onLaunch: () => void;
		onKill: () => void;
		onOpenSettings: () => void;
		onOpenFolder: () => void;
		onCheckUpdate: () => void;
	}

	let {
		instance,
		status,
		activeAccountName,
		totalPlayTime,
		hasUpdate = false,
		onLaunch,
		onKill,
		onOpenSettings,
		onOpenFolder,
		onCheckUpdate,
	}: Props = $props();

	let isPlayButtonHovered = $state(false);
	let updateBannerDismissed = $state(false);

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
		if (icon) return getIconUrl(icon);
		return '/icons/entities/creeper/creeper.png';
	}

	function handlePlayButtonClick() {
		if (status === 'running' || status === 'windowReady') {
			onKill();
		} else {
			onLaunch();
		}
	}

	const isLaunching = $derived(
		status === 'checkingAccount' ||
			status === 'refreshingToken' ||
			status === 'loadingVersion' ||
			status === 'verifyingFiles' ||
			status === 'downloading' ||
			status === 'checkingJava' ||
			status === 'downloadingJava' ||
			status === 'buildingClasspath' ||
			status === 'launching' ||
			status === 'running'
	);
	const isRunning = $derived(status === 'running' || status === 'windowReady');
	const canLaunch = $derived(!status || status === 'stopped' || status === 'crashed');
</script>

<!-- Hero Header -->
<div class="relative overflow-hidden">
	<!-- Blurred background using instance icon -->
	<div class="from-muted/60 to-muted absolute inset-0 bg-gradient-to-br">
		<div class="absolute inset-0 flex items-center justify-center opacity-15 blur-3xl">
			<img
				src={getIconSrc(instance.iconPath)}
				alt=""
				class="pixelated h-64 w-64"
				aria-hidden="true"
			/>
		</div>
	</div>

	<!-- Content -->
	<div class="relative px-8 py-8">
		<div class="flex items-start gap-8">
			<!-- Large Icon with glow -->
			<div class="relative">
				<div
					class="from-primary/30 to-primary/0 absolute inset-0 rounded-lg bg-gradient-to-br blur-xl"
				></div>
				<img
					src={getIconSrc(instance.iconPath)}
					alt="{instance.name} icon"
					class="pixelated relative h-24 w-24 drop-shadow-2xl"
				/>
			</div>

			<!-- Info Section -->
			<div class="flex min-w-0 flex-1 flex-col gap-3">
				<!-- Instance Name -->
				<h1 class="text-3xl leading-tight font-bold tracking-tight">{instance.name}</h1>

				<!-- Version Badges -->
				<div class="flex flex-wrap items-center gap-2">
					<span class="bg-muted/60 rounded border px-2.5 py-1 text-sm font-medium">
						{instance.minecraftVersion}
					</span>
					<span
						class="rounded border px-2.5 py-1 text-sm font-medium capitalize {getLoaderColor(
							instance.loaderType
						)}"
					>
						{instance.loaderType}
						{#if instance.loaderVersion}
							<span class="ml-1 opacity-75">{instance.loaderVersion}</span>
						{/if}
					</span>
					{#if instance.modpackPlatform}
						<span
							class="flex items-center gap-1 rounded border border-blue-500/50 bg-blue-500/20 px-2.5 py-1 text-sm font-medium text-blue-400"
						>
							<Package class="h-3.5 w-3.5" />
							Modpack
						</span>
					{/if}
				</div>

				<!-- Stats Row -->
				<div class="text-muted-foreground flex flex-wrap items-center gap-4 text-sm">
					<span class="flex items-center gap-1.5">
						<Clock class="h-4 w-4" />
						{formatPlayTime(totalPlayTime)}
					</span>
					<span class="flex items-center gap-1.5">
						<Calendar class="h-4 w-4" />
						Created {formatDate(instance.createdAt)}
					</span>
					{#if instance.lastPlayedAt}
						<span class="flex items-center gap-1.5">
							<Play class="h-4 w-4" />
							Last played {formatDate(instance.lastPlayedAt)}
						</span>
					{/if}
					{#if activeAccountName}
						<span class="flex items-center gap-1.5">
							<img
								src={getAvatarUrl(activeAccountName, 16)}
								alt=""
								class="pixelated h-4 w-4 rounded"
							/>
							{activeAccountName}
						</span>
					{/if}
				</div>
			</div>

			<!-- Action Buttons -->
			<div class="mr-10 flex flex-col gap-2">
				<!-- Large Play Button -->
				<Button
					variant="default"
					size="lg"
					class="h-14 w-40 text-lg"
					onclick={handlePlayButtonClick}
					onmouseenter={() => (isPlayButtonHovered = true)}
					onmouseleave={() => (isPlayButtonHovered = false)}
					disabled={!canLaunch && !isRunning}
				>
					{#if isLaunching}
						<Loader2 class="mr-2 h-5 w-5 animate-spin" />
						Launching
					{:else if isRunning}
						{#if isPlayButtonHovered}
							<X class="mr-2 h-5 w-5" />
							Stop
						{:else}
							<Square class="mr-2 h-5 w-5" />
							Running
						{/if}
					{:else}
						<Play class="mr-2 h-5 w-5" />
						Play
					{/if}
				</Button>

				<!-- Secondary Actions -->
				<Button variant="outline" size="sm" class="w-full" onclick={onOpenSettings}>
					<Settings class="mr-1.5 h-4 w-4" />
					Settings
				</Button>
				<Button variant="outline" size="sm" class="w-full" onclick={onOpenFolder}>
					<FolderOpen class="mr-1.5 h-4 w-4" />
					Folder
				</Button>
			</div>
		</div>

		<!-- Update Banner (if update available) -->
		{#if hasUpdate && !updateBannerDismissed}
			<div
				class="bg-primary/10 border-primary/30 mt-6 flex w-full items-center justify-between rounded-lg border px-4 py-3"
			>
				<button
					class="flex flex-1 items-center gap-3 text-left transition-opacity hover:opacity-80"
					onclick={onCheckUpdate}
				>
					<div class="bg-primary/20 rounded-full p-2">
						<ArrowUp class="text-primary h-5 w-5" />
					</div>
					<div>
						<p class="text-primary font-medium">Update Available</p>
						<p class="text-muted-foreground text-sm">
							A newer version of this {instance.modpackPlatform ? 'modpack' : 'instance'} is available
						</p>
					</div>
				</button>
				<div class="flex items-center gap-3">
					<button class="text-primary text-sm font-medium hover:underline" onclick={onCheckUpdate}>
						Check Updates &rarr;
					</button>
					<button
						class="text-muted-foreground hover:text-foreground p-1 transition-colors"
						onclick={() => (updateBannerDismissed = true)}
						title="Dismiss"
					>
						<X class="h-4 w-4" />
					</button>
				</div>
			</div>
		{/if}
	</div>
</div>
