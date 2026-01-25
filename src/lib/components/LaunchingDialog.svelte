<script lang="ts">
	import { Loader2, Download } from '@lucide/svelte';
	import { launchStore } from '$lib/stores/launch.svelte';
	import { instancesStore } from '$lib/stores/instances.svelte';
	import { parseIconPath, getIconUrl } from '$lib/utils/icons';
	import { formatBytes, formatSpeed, formatEta } from '$lib/utils/format';

	// Find the first instance that is currently launching (before window is ready)
	const launchingState = $derived.by(() => {
		for (const [instanceId, state] of launchStore.launchStates.entries()) {
			const status = state.status.status;
			// Show dialog for all pre-windowReady states
			if (
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
			) {
				return { instanceId, status: state.status };
			}
			// windowReady means the window is open - don't show dialog
		}
		return null;
	});

	// Computed: should we show the dialog?
	const showDialog = $derived(launchingState !== null);

	const instance = $derived(
		launchingState ? instancesStore.instances.find((i) => i.id === launchingState.instanceId) : null
	);

	// Get status message
	function getStatusMessage(
		status:
			| { status: 'checkingAccount' }
			| { status: 'refreshingToken' }
			| { status: 'loadingVersion' }
			| { status: 'verifyingFiles'; checked: number; total: number }
			| { status: 'downloading'; progress: { completedFiles: number; totalFiles: number } }
			| { status: 'checkingJava'; version: number }
			| { status: 'downloadingJava'; version: number; progress: number }
			| { status: 'buildingClasspath' }
			| { status: 'launching' }
			| { status: 'running'; pid: number }
	): string {
		switch (status.status) {
			case 'checkingAccount':
				return 'Checking account...';
			case 'refreshingToken':
				return 'Refreshing token...';
			case 'loadingVersion':
				return 'Loading version info...';
			case 'verifyingFiles':
				return `Verifying files (${status.checked}/${status.total})...`;
			case 'downloading':
				return `Downloading files (${status.progress.completedFiles}/${status.progress.totalFiles})...`;
			case 'checkingJava':
				return `Checking Java ${status.version}...`;
			case 'downloadingJava':
				return `Downloading Java ${status.version} (${status.progress}%)...`;
			case 'buildingClasspath':
				return 'Building classpath...';
			case 'launching':
				return 'Starting Minecraft...';
			case 'running':
				return 'Opening Minecraft window...';
			default:
				return 'Launching...';
		}
	}

	function getIconSrc(iconPath: string | undefined): string {
		if (!iconPath) return '/icons/entity/creeper.webp';
		const parsed = parseIconPath(iconPath);
		if (!parsed) return '/icons/entity/creeper.webp';
		return getIconUrl(parsed);
	}

	// Calculate overall progress percentage based on current stage
	function getOverallProgress(
		status:
			| { status: 'checkingAccount' }
			| { status: 'refreshingToken' }
			| { status: 'loadingVersion' }
			| { status: 'verifyingFiles'; checked: number; total: number }
			| {
					status: 'downloading';
					progress: {
						completedFiles: number;
						totalFiles: number;
						downloadedBytes: number;
						totalBytes: number;
					};
			  }
			| { status: 'checkingJava'; version: number }
			| { status: 'downloadingJava'; version: number; progress: number }
			| { status: 'buildingClasspath' }
			| { status: 'launching' }
			| { status: 'running'; pid: number }
	): number {
		switch (status.status) {
			case 'checkingAccount':
				return 5;
			case 'refreshingToken':
				return 10;
			case 'loadingVersion':
				return 15;
			case 'verifyingFiles': {
				const base = 20;
				const range = 10; // 20-30%
				const progress = status.total > 0 ? status.checked / status.total : 0;
				return base + progress * range;
			}
			case 'downloading': {
				const base = 30;
				const range = 40; // 30-70%
				const progress =
					status.progress.totalBytes > 0
						? status.progress.downloadedBytes / status.progress.totalBytes
						: status.progress.totalFiles > 0
							? status.progress.completedFiles / status.progress.totalFiles
							: 0;
				return base + progress * range;
			}
			case 'checkingJava':
				return 75;
			case 'downloadingJava': {
				const base = 75;
				const range = 10; // 75-85%
				return base + (status.progress / 100) * range;
			}
			case 'buildingClasspath':
				return 90;
			case 'launching':
				return 95;
			case 'running':
				return 100;
			default:
				return 0;
		}
	}

	const progress = $derived(launchingState ? getOverallProgress(launchingState.status) : 0);
</script>

{#if showDialog && instance}
	<div
		class="fixed inset-0 z-[100] flex items-center justify-center bg-black/80"
		role="dialog"
		aria-label="Launching Minecraft"
	>
		<div
			class="bg-card border-border flex w-full max-w-md flex-col items-center gap-6 rounded-xl border-2 p-8 shadow-2xl"
		>
			<!-- Instance icon -->
			<div class="relative">
				<img
					src={getIconSrc(instance.iconPath)}
					alt={instance.name}
					class="h-24 w-24 rounded-lg object-cover"
				/>
				<div
					class="bg-primary absolute -right-2 -bottom-2 flex h-8 w-8 items-center justify-center rounded-full"
				>
					<Loader2 class="text-primary-foreground h-5 w-5 animate-spin" />
				</div>
			</div>

			<!-- Instance name -->
			<div class="text-center">
				<h2 class="text-xl font-semibold">{instance.name}</h2>
				<p class="text-muted-foreground text-sm">
					{instance.minecraftVersion}
					{#if instance.loaderType !== 'vanilla'}
						<span class="capitalize"> - {instance.loaderType}</span>
					{/if}
				</p>
			</div>

			<!-- Progress section -->
			<div class="w-full space-y-3">
				<!-- Main progress bar -->
				<div class="space-y-2">
					<div class="bg-muted h-2 w-full overflow-hidden rounded-full">
						<div
							class="bg-primary h-full transition-all duration-300 ease-out"
							style="width: {progress}%"
						></div>
					</div>
					<!-- Status message -->
					{#if launchingState}
						<p class="text-muted-foreground text-center text-sm">
							{getStatusMessage(launchingState.status)}
						</p>
					{/if}
				</div>

				<!-- Detailed download progress when downloading -->
				{#if launchingState?.status.status === 'downloading'}
					{@const dlProgress = launchingState.status.progress}
					{@const hasBytes = dlProgress.totalBytes > 0}
					{@const speed = dlProgress.speedBytesPerSec || 0}
					{@const eta = speed > 0 && dlProgress.totalBytes > dlProgress.downloadedBytes
						? formatEta((dlProgress.totalBytes - dlProgress.downloadedBytes) / speed)
						: ''}
					<div class="bg-muted/30 rounded-lg p-3 space-y-2">
						<!-- Download header with speed and ETA -->
						<div class="flex items-center justify-between text-xs">
							<div class="flex items-center gap-1.5 text-muted-foreground">
								<Download class="h-3.5 w-3.5" />
								<span class="truncate max-w-[180px]" title={dlProgress.currentFile}>
									{dlProgress.currentFile || 'Downloading...'}
								</span>
							</div>
							{#if speed > 0}
								<div class="flex items-center gap-2 font-mono text-muted-foreground">
									<span>{formatSpeed(speed)}</span>
									{#if eta}
										<span class="text-muted-foreground/60">•</span>
										<span>{eta}</span>
									{/if}
								</div>
							{/if}
						</div>

						<!-- File progress bar -->
						{#if hasBytes}
							{@const fileProgress = (dlProgress.downloadedBytes / dlProgress.totalBytes) * 100}
							<div class="bg-muted h-1.5 w-full overflow-hidden rounded-full">
								<div
									class="bg-primary/70 h-full transition-all duration-150 ease-out"
									style="width: {fileProgress}%"
								></div>
							</div>
							<div class="flex justify-between text-xs text-muted-foreground">
								<span>{formatBytes(dlProgress.downloadedBytes)} / {formatBytes(dlProgress.totalBytes)}</span>
								<span>{dlProgress.completedFiles} / {dlProgress.totalFiles} files</span>
							</div>
						{:else}
							<div class="text-xs text-muted-foreground">
								{dlProgress.completedFiles} / {dlProgress.totalFiles} files
							</div>
						{/if}
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}
