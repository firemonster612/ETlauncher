<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/ui/button';
	import { Checkbox } from '$lib/ui/checkbox';
	import { Input } from '$lib/ui/input';
	import { Slider } from '$lib/ui/slider';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { RotateCcw } from '@lucide/svelte';

	let saveStatus = $state<'idle' | 'saving' | 'saved' | 'error'>('idle');

	// Convenience getter for settings with defaults
	const settings = $derived(settingsStore.settings);

	// Local state for sliders (for instant visual feedback)
	let memoryMin = $state(0);
	let memoryMax = $state(0);
	let concurrentDownloads = $state(1);

	// Sync local state when settings load
	$effect(() => {
		if (settings) {
			memoryMin = settings.memoryMinMb;
			memoryMax = settings.memoryMaxMb;
			concurrentDownloads = settings.concurrentDownloads;
		}
	});

	onMount(() => {
		settingsStore.load();
	});

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
		if (confirm('Reset all settings to defaults?')) {
			await settingsStore.reset();
		}
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
					<span>Minimum RAM</span>
					<span class="text-primary">{formatMemory(memoryMin)}</span>
				</div>
				<Slider
					min={256}
					max={8192}
					step={256}
					value={memoryMin}
					onValueChange={(value) => {
						if (value <= memoryMax) {
							memoryMin = value;
						}
					}}
					onValueCommit={(value) => {
						if (value <= memoryMax) {
							saveSettings({ memoryMinMb: value });
						}
					}}
				/>
			</div>

			<div class="space-y-2">
				<div class="flex justify-between text-sm">
					<span>Maximum RAM</span>
					<span class="text-primary">{formatMemory(memoryMax)}</span>
				</div>
				<Slider
					min={512}
					max={16384}
					step={512}
					value={memoryMax}
					onValueChange={(value) => {
						if (value >= memoryMin) {
							memoryMax = value;
						}
					}}
					onValueCommit={(value) => {
						if (value >= memoryMin) {
							saveSettings({ memoryMaxMb: value });
						}
					}}
				/>
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
