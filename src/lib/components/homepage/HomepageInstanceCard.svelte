<script lang="ts">
	import { Play, Square, X, Clock, Calendar, Loader2, ChevronRight } from '@lucide/svelte';
	import { Button } from '$lib/ui/button';
	import { parseIconPath, getIconUrl } from '$lib/utils/icons';
	import type { Instance, LoaderType, LaunchStatus } from '$lib/types';

	interface Props {
		instance: Instance;
		launchStatus: LaunchStatus | undefined;
		onLaunch: (instanceId: string) => void;
		onKill: (instanceId: string) => void;
		onCardClick: (instance: Instance) => void;
	}

	let { instance, launchStatus, onLaunch, onKill, onCardClick }: Props = $props();

	let isPlayButtonHovered = $state(false);

	// Computed status
	const status = $derived(launchStatus?.status ?? null);

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
		return '/icons/entities/creeper/creeper.png';
	}

	function handleCardClick(e: MouseEvent) {
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
</script>

<div
	class="border-border bg-card instance-card-glow group hover:border-primary/60 relative cursor-pointer border-2 transition-all duration-200"
	onclick={handleCardClick}
	onkeydown={(e) => e.key === 'Enter' && onCardClick(instance)}
	role="button"
	tabindex="0"
>
	<!-- Icon Banner Section -->
	<div class="from-muted/50 to-muted relative h-24 overflow-hidden bg-gradient-to-br" style="isolation: isolate;">
		<!-- Blurred background icon -->
		<div class="absolute inset-0 flex items-center justify-center opacity-20 blur-xl" style="will-change: transform; backface-visibility: hidden; transform: translateZ(0);">
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
		<!-- Title - 2 lines max -->
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

	<!-- Bottom bar with play button only -->
	<div class="border-border border-t px-4 py-3">
		<Button
			variant="default"
			size="sm"
			class="w-full"
			onclick={handlePlayButtonClick}
			onmouseenter={() => (isPlayButtonHovered = true)}
			onmouseleave={() => (isPlayButtonHovered = false)}
			disabled={status !== null &&
				status !== 'stopped' &&
				status !== 'crashed' &&
				status !== 'running' &&
				status !== 'windowReady'}
		>
			{#if isLaunching}
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
	</div>
</div>
