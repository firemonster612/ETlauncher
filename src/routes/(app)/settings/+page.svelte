<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/ui/button';
	import { Checkbox } from '$lib/ui/checkbox';
	import { Input } from '$lib/ui/input';
	import * as Select from '$lib/ui/select';
	import { Slider, RangeSlider } from '$lib/ui/slider';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { RotateCcw, Eye, EyeOff, HardDrive, Trash2, RefreshCw, Check, X } from '@lucide/svelte';
	import * as settingsService from '$lib/services/settings';
	import type { ResourcePoolStats, LinkStrategy } from '$lib/types';

	let saveStatus = $state<'idle' | 'saving' | 'saved' | 'error'>('idle');

	// Convenience getter for settings with defaults
	const settings = $derived(settingsStore.settings);

	// Local state for sliders (for instant visual feedback)
	let memoryRange = $state<[number, number]>([512, 4096]);
	let concurrentDownloads = $state(1);
	let showApiKey = $state(false);

	// Resource pool state
	let poolStats = $state<ResourcePoolStats | null>(null);
	let poolLoading = $state(false);
	let poolMigrating = $state(false);
	let gcRunning = $state(false);
	let poolActionResult = $state<{ type: 'success' | 'error'; message: string } | null>(null);

	// Sync local state when settings load
	$effect(() => {
		if (settings) {
			memoryRange = [settings.memoryMinMb, settings.memoryMaxMb];
			concurrentDownloads = settings.concurrentDownloads;
		}
	});

	onMount(() => {
		settingsStore.load();
		loadPoolStats();
	});

	async function loadPoolStats() {
		poolLoading = true;
		try {
			poolStats = await settingsService.getPoolStats();
		} catch (e) {
			console.error('Failed to load pool stats:', e);
		} finally {
			poolLoading = false;
		}
	}

	async function runGarbageCollection() {
		gcRunning = true;
		poolActionResult = null;
		try {
			const result = await settingsService.garbageCollectPool();
			await loadPoolStats();
			poolActionResult = {
				type: 'success',
				message: `Cleaned ${result.resourcesRemoved} unused resources, freed ${formatBytes(result.bytesFreed)}`,
			};
		} catch (e) {
			console.error('GC failed:', e);
			poolActionResult = {
				type: 'error',
				message: 'Cleanup failed. Check console for details.',
			};
		} finally {
			gcRunning = false;
		}
	}

	async function migrateAllInstances() {
		poolMigrating = true;
		poolActionResult = null;
		try {
			const result = await settingsService.migrateAllInstancesToPool();
			await loadPoolStats();
			poolActionResult = {
				type: 'success',
				message: `Migrated ${result.instancesMigrated} instances (${result.totalFilesMigrated} files), saved ${formatBytes(result.totalSpaceSavedBytes)}`,
			};
		} catch (e) {
			console.error('Migration failed:', e);
			poolActionResult = {
				type: 'error',
				message: 'Migration failed. Check console for details.',
			};
		} finally {
			poolMigrating = false;
		}
	}

	function formatBytes(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
	}

	async function saveSettings(updates: Record<string, unknown>) {
		saveStatus = 'saving';
		try {
			await settingsStore.update(updates as Parameters<typeof settingsStore.update>[0]);
			saveStatus = 'saved';
			setTimeout(() => {
				saveStatus = 'idle';
			}, 2000);
		} catch {
			saveStatus = 'error';
		}
	}

	async function resetToDefaults() {
		await settingsStore.reset();
	}

	function formatMemory(mb: number | undefined): string {
		if (mb === undefined) return '...';
		if (mb >= 1024) {
			return `${(mb / 1024).toFixed(1)} GB`;
		}
		return `${mb} MB`;
	}
</script>

<div class="max-w-2xl space-y-6">
	<div class="flex items-center justify-between">
		<h1 class="text-2xl">Settings</h1>
		{#if saveStatus === 'saving'}
			<span class="text-muted-foreground text-sm">Saving...</span>
		{:else if saveStatus === 'saved'}
			<span class="text-primary text-sm">Saved</span>
		{:else if saveStatus === 'error'}
			<span class="text-destructive text-sm">Error saving</span>
		{/if}
	</div>

	{#if settingsStore.isLoading || !settings}
		<div class="text-muted-foreground">Loading settings...</div>
	{:else if settingsStore.error}
		<div class="text-destructive">Error: {settingsStore.error}</div>
	{:else}
		<!-- General Settings -->
		<section class="border-border bg-card space-y-4 border-2 p-4">
			<h3 class="text-muted-foreground text-sm tracking-wider uppercase">General</h3>

			<div class="space-y-2">
				<label for="instancesPath" class="text-sm">Instances Path</label>
				<div class="flex gap-2">
					<Input
						id="instancesPath"
						type="text"
						value={settings.instancesPath}
						onchange={(e) => saveSettings({ instancesPath: e.currentTarget.value })}
						class="flex-1"
					/>
				</div>
				<p class="text-muted-foreground text-xs">Where game instances are stored</p>
			</div>

			<div class="flex items-center justify-between">
				<div>
					<span class="text-sm">Close launcher when game starts</span>
					<p class="text-muted-foreground text-xs">Hide the launcher while playing</p>
				</div>
				<Checkbox
					checked={settings.closeLauncherOnGameStart}
					onCheckedChange={(checked) => saveSettings({ closeLauncherOnGameStart: !!checked })}
				/>
			</div>

			<div class="flex items-center justify-between">
				<div>
					<span class="text-sm">Reopen launcher when game closes</span>
					<p class="text-muted-foreground text-xs">Show the launcher after exiting the game</p>
				</div>
				<Checkbox
					checked={settings.reopenLauncherOnGameClose}
					onCheckedChange={(checked) => saveSettings({ reopenLauncherOnGameClose: !!checked })}
				/>
			</div>
		</section>

		<!-- Memory Settings -->
		<section class="border-border bg-card space-y-4 border-2 p-4">
			<h3 class="text-muted-foreground text-sm tracking-wider uppercase">Memory</h3>

			<div class="space-y-2">
				<div class="flex justify-between text-sm">
					<span>Min: <span class="text-primary">{formatMemory(memoryRange[0])}</span></span>
					<span>Max: <span class="text-primary">{formatMemory(memoryRange[1])}</span></span>
				</div>
				<RangeSlider
					min={512}
					max={16384}
					step={512}
					value={memoryRange}
					onValueChange={(value) => {
						memoryRange = value;
					}}
					onValueCommit={(value) => {
						saveSettings({ memoryMinMb: value[0], memoryMaxMb: value[1] });
					}}
				/>
				<div class="text-muted-foreground flex justify-between text-xs">
					<span>512 MB</span>
					<span>16 GB</span>
				</div>
			</div>

			<p class="text-muted-foreground text-xs">
				Default memory allocation for new instances. Can be overridden per-instance.
			</p>
		</section>

		<!-- Downloads Settings -->
		<section class="border-border bg-card space-y-4 border-2 p-4">
			<h3 class="text-muted-foreground text-sm tracking-wider uppercase">Downloads</h3>

			<div class="space-y-2">
				<div class="flex justify-between text-sm">
					<span>Concurrent Downloads</span>
					<span class="text-primary">{concurrentDownloads}</span>
				</div>
				<Slider
					min={1}
					max={16}
					step={1}
					value={concurrentDownloads}
					onValueChange={(value) => {
						concurrentDownloads = value;
					}}
					onValueCommit={(value) => saveSettings({ concurrentDownloads: value })}
				/>
				<p class="text-muted-foreground text-xs">Number of files to download simultaneously</p>
			</div>
		</section>

		<!-- CurseForge API -->
		<section class="border-border bg-card space-y-4 border-2 p-4">
			<h3 class="text-muted-foreground text-sm tracking-wider uppercase">CurseForge</h3>

			<div class="space-y-2">
				<label for="curseforgeApiKey" class="text-sm">API Key</label>
				<div class="flex gap-2">
					<Input
						id="curseforgeApiKey"
						type={showApiKey ? 'text' : 'password'}
						placeholder="Enter your CurseForge API key"
						value={settings.curseforgeApiKey ?? ''}
						onchange={(e) => saveSettings({ curseforgeApiKey: e.currentTarget.value || undefined })}
						class="flex-1"
					/>
					<Button
						variant="outline"
						size="icon"
						onclick={() => (showApiKey = !showApiKey)}
						aria-label={showApiKey ? 'Hide API key' : 'Show API key'}
					>
						{#if showApiKey}
							<EyeOff class="h-4 w-4" />
						{:else}
							<Eye class="h-4 w-4" />
						{/if}
					</Button>
				</div>
				<p class="text-muted-foreground text-xs">
					Required to browse and download CurseForge content. Get your free API key at
					<a
						href="https://console.curseforge.com"
						target="_blank"
						rel="noopener noreferrer"
						class="text-primary underline hover:no-underline"
					>
						console.curseforge.com
					</a>
				</p>
				<p class="text-muted-foreground text-xs">
					A built-in API key is planned for a future release. For now, each user needs their own key
					due to CurseForge's API terms of service.
				</p>
			</div>
		</section>

		<!-- Resource Pool Settings -->
		<section class="border-border bg-card space-y-4 border-2 p-4">
			<h3 class="text-muted-foreground text-sm tracking-wider uppercase">Resource Pool</h3>

			<div class="flex items-center justify-between">
				<div>
					<span class="text-sm">Enable Resource Pool</span>
					<p class="text-muted-foreground text-xs">
						Store mods/shaders/resourcepacks once and link to instances
					</p>
				</div>
				<Checkbox
					checked={settings.resourcePool?.enabled ?? true}
					onCheckedChange={(checked) =>
						saveSettings({
							resourcePool: { ...settings.resourcePool, enabled: !!checked },
						})}
				/>
			</div>

			{#if settings.resourcePool?.enabled}
				<div class="space-y-2">
					<label for="linkStrategy" class="text-sm">Link Strategy</label>
					<Select.Root
						type="single"
						value={settings.resourcePool?.linkStrategy ?? 'auto'}
						onValueChange={(value: string) =>
							saveSettings({
								resourcePool: {
									...settings.resourcePool,
									linkStrategy: value as LinkStrategy,
								},
							})}
					>
						<Select.Trigger class="border-input bg-background w-full border-2 p-2 text-sm">
							{#if settings.resourcePool?.linkStrategy === 'hardLink'}
								Always use hard links
							{:else if settings.resourcePool?.linkStrategy === 'symlink'}
								Always use symlinks
							{:else if settings.resourcePool?.linkStrategy === 'copy'}
								Always copy files
							{:else}
								Auto (hard link when possible)
							{/if}
						</Select.Trigger>
						<Select.Content>
							<Select.Item value="auto" label="Auto (hard link when possible)"
								>Auto (hard link when possible)</Select.Item
							>
							<Select.Item value="hardLink" label="Always use hard links"
								>Always use hard links</Select.Item
							>
							<Select.Item value="symlink" label="Always use symlinks"
								>Always use symlinks</Select.Item
							>
							<Select.Item value="copy" label="Always copy files">Always copy files</Select.Item>
						</Select.Content>
					</Select.Root>
					<p class="text-muted-foreground text-xs">How to link files from the pool to instances</p>
				</div>

				<!-- Pool Statistics -->
				<div class="bg-muted/50 space-y-2 p-3">
					<div class="flex items-center gap-2 text-sm">
						<HardDrive class="h-4 w-4" />
						<span>Pool Statistics</span>
						<Button variant="ghost" size="icon" class="ml-auto h-6 w-6" onclick={loadPoolStats}>
							<RefreshCw class="h-3 w-3" />
						</Button>
					</div>
					{#if poolLoading}
						<p class="text-muted-foreground text-xs">Loading...</p>
					{:else if poolStats}
						<div class="text-muted-foreground grid grid-cols-2 gap-x-4 gap-y-1 text-xs">
							<span>Total resources:</span>
							<span class="text-foreground">{poolStats.totalResources}</span>
							<span>Mods:</span>
							<span class="text-foreground">{poolStats.modCount}</span>
							<span>Shaders:</span>
							<span class="text-foreground">{poolStats.shaderCount}</span>
							<span>Resource packs:</span>
							<span class="text-foreground">{poolStats.resourcePackCount}</span>
							<span>Pool size:</span>
							<span class="text-foreground">{formatBytes(poolStats.totalSizeBytes)}</span>
							<span>Space saved:</span>
							<span class="text-primary">{formatBytes(poolStats.spaceSavedBytes)}</span>
							<span>Unused:</span>
							<span class="text-foreground">{poolStats.unusedCount}</span>
						</div>
					{:else}
						<p class="text-muted-foreground text-xs">No pool data available</p>
					{/if}
				</div>

				<!-- Pool Actions -->
				<div class="flex flex-wrap gap-2">
					<Button
						variant="outline"
						size="sm"
						onclick={migrateAllInstances}
						disabled={poolMigrating}
					>
						{#if poolMigrating}
							Migrating...
						{:else}
							Migrate Existing Instances
						{/if}
					</Button>
					<Button variant="outline" size="sm" onclick={runGarbageCollection} disabled={gcRunning}>
						<Trash2 class="mr-2 h-4 w-4" />
						{#if gcRunning}
							Running...
						{:else}
							Clean Unused
						{/if}
					</Button>
				</div>

				{#if poolActionResult}
					<div
						class="flex items-center gap-2 text-sm {poolActionResult.type === 'success'
							? 'text-primary'
							: 'text-destructive'}"
					>
						{#if poolActionResult.type === 'success'}
							<Check class="h-4 w-4" />
						{:else}
							<X class="h-4 w-4" />
						{/if}
						<span>{poolActionResult.message}</span>
						<button
							class="text-muted-foreground hover:text-foreground ml-auto"
							onclick={() => (poolActionResult = null)}
						>
							<X class="h-3 w-3" />
						</button>
					</div>
				{/if}
			{/if}
		</section>

		<!-- Version Settings -->
		<section class="border-border bg-card space-y-4 border-2 p-4">
			<h3 class="text-muted-foreground text-sm tracking-wider uppercase">Versions</h3>

			<div class="flex items-center justify-between">
				<div>
					<span class="text-sm">Show snapshots</span>
					<p class="text-muted-foreground text-xs">Include snapshot versions in version list</p>
				</div>
				<Checkbox
					checked={settings.showSnapshots}
					onCheckedChange={(checked) => saveSettings({ showSnapshots: !!checked })}
				/>
			</div>

			<div class="flex items-center justify-between">
				<div>
					<span class="text-sm">Show old versions</span>
					<p class="text-muted-foreground text-xs">Include alpha and beta versions</p>
				</div>
				<Checkbox
					checked={settings.showOldVersions}
					onCheckedChange={(checked) => saveSettings({ showOldVersions: !!checked })}
				/>
			</div>
		</section>

		<!-- Reset -->
		<section class="border-border bg-card border-2 p-4">
			<div class="flex items-center justify-between">
				<div>
					<span class="text-sm">Reset to Defaults</span>
					<p class="text-muted-foreground text-xs">Restore all settings to their default values</p>
				</div>
				<Button variant="outline" size="sm" onclick={resetToDefaults}>
					<RotateCcw class="mr-2 h-4 w-4" />
					Reset
				</Button>
			</div>
		</section>
	{/if}
</div>
