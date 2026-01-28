<script lang="ts">
	import { Play, Square, X, Clock, Loader2, RotateCcw } from '@lucide/svelte';
	import { Button } from '$lib/ui/button';
	import { parseIconPath, getIconUrl } from '$lib/utils/icons';
	import type { Instance, LaunchStatus } from '$lib/types';

	interface Props {
		instance: Instance;
		launchStatus: LaunchStatus | undefined;
		onLaunch: (instanceId: string) => void;
		onKill: (instanceId: string) => void;
		onCardClick: (instance: Instance) => void;
	}

	let { instance, launchStatus, onLaunch, onKill, onCardClick }: Props = $props();

	let isPlayButtonHovered = $state(false);

	const status = $derived(launchStatus?.status ?? null);

	function formatTimeSince(timestamp: number): string {
		const now = Date.now() / 1000;
		const diff = now - timestamp;

		if (diff < 60) return 'Just now';
		if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
		if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
		if (diff < 604800) return `${Math.floor(diff / 86400)}d ago`;
		return new Date(timestamp * 1000).toLocaleDateString();
	}

	function formatPlayTime(seconds: number): string {
		if (seconds < 60) return '< 1 min';
		const hours = Math.floor(seconds / 3600);
		const minutes = Math.floor((seconds % 3600) / 60);
		if (hours === 0) return `${minutes}m`;
		return `${hours}h ${minutes}m`;
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

<div class="space-y-2">
	<!-- Header -->
	<div class="flex items-center gap-2">
		<RotateCcw class="text-primary h-4 w-4" />
		<h2 class="text-sm font-bold uppercase tracking-wider">Continue Playing</h2>
	</div>

	<div
		class="border-border bg-card hover:border-primary/60 group relative flex cursor-pointer gap-4 border-2 p-4 transition-all"
		onclick={handleCardClick}
		onkeydown={(e) => e.key === 'Enter' && onCardClick(instance)}
		role="button"
		tabindex="0"
	>
		<!-- Icon -->
		<div class="from-muted/50 to-muted relative h-20 w-20 flex-shrink-0 overflow-hidden bg-gradient-to-br">
			<div class="absolute inset-0 flex items-center justify-center">
				<img
					src={getIconSrc(instance.iconPath)}
					alt="{instance.name} icon"
					class="pixelated h-14 w-14 drop-shadow-lg"
				/>
			</div>
		</div>

		<!-- Info -->
		<div class="flex min-w-0 flex-1 flex-col justify-center gap-1">
			<h3 class="truncate text-lg font-bold" title={instance.name}>
				{instance.name}
			</h3>
			<div class="text-muted-foreground flex flex-wrap items-center gap-x-4 gap-y-1 text-xs">
				<span>{instance.minecraftVersion}</span>
				{#if instance.lastPlayedAt}
					<span class="flex items-center gap-1">
						<Clock class="h-3 w-3" />
						{formatTimeSince(instance.lastPlayedAt)}
					</span>
				{/if}
				<span>{formatPlayTime(instance.totalPlayTime)} played</span>
			</div>
		</div>

		<!-- Play Button -->
		<div class="flex items-center">
			<Button
				variant="default"
				size="lg"
				class="h-14 w-24"
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
					<Loader2 class="h-6 w-6 animate-spin" />
				{:else if status === 'windowReady'}
					{#if isPlayButtonHovered}
						<X class="h-6 w-6" />
					{:else}
						<Square class="h-6 w-6" />
					{/if}
				{:else}
					<Play class="h-6 w-6" />
				{/if}
			</Button>
		</div>
	</div>
</div>
