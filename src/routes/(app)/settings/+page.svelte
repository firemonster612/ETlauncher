<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/ui/button';
	import { Checkbox } from '$lib/ui/checkbox';
	import { Input } from '$lib/ui/input';
	import * as Select from '$lib/ui/select';
	import { Slider, RangeSlider } from '$lib/ui/slider';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { themeStore } from '$lib/stores/theme.svelte';
	import {
		RotateCcw,
		Eye,
		EyeOff,
		HardDrive,
		Trash2,
		RefreshCw,
		Check,
		X,
		Monitor,
		Sun,
		Moon,
		Palette,
	} from '@lucide/svelte';
	import * as settingsService from '$lib/services/settings';
	import ColorPicker from '$lib/components/ColorPicker.svelte';
	import type {
		ResourcePoolStats,
		LinkStrategy,
		Theme,
		ColorPreset,
		ThemeColors,
	} from '$lib/types';

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
	let gcRunning = $state(false);
	let poolActionResult = $state<{ type: 'success' | 'error'; message: string } | null>(null);

	// Theme state
	let customHue = $state(172);
	let customChroma = $state(0.18);

	// Sidebar state
	let sidebarHue = $state(220);
	let sidebarChroma = $state(0.05);

	// Color presets configuration
	const colorPresets: { id: ColorPreset; label: string; hue: number; chroma: number }[] = [
		{ id: 'default', label: 'Cyan', hue: 172, chroma: 0.18 },
		{ id: 'purple', label: 'Purple', hue: 280, chroma: 0.2 },
		{ id: 'green', label: 'Green', hue: 145, chroma: 0.18 },
		{ id: 'orange', label: 'Orange', hue: 45, chroma: 0.2 },
		{ id: 'pink', label: 'Pink', hue: 330, chroma: 0.22 },
		{ id: 'blue', label: 'Blue', hue: 220, chroma: 0.18 },
	];

	// Sync local state when settings load
	$effect(() => {
		if (settings) {
			memoryRange = [settings.memoryMinMb, settings.memoryMaxMb];
			concurrentDownloads = settings.concurrentDownloads;
			// Sync custom color state
			if (settings.customColors) {
				customHue = settings.customColors.primaryHue ?? 172;
				customChroma = settings.customColors.primaryChroma ?? 0.18;
			}
			// Sync sidebar color state
			if (settings.customSidebarColor) {
				sidebarHue = settings.customSidebarColor.hue ?? 220;
				sidebarChroma = settings.customSidebarColor.chroma ?? 0.05;
			}
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

	/** Save theme preference to localStorage for flash prevention */
	function saveThemeToLocalStorage(theme: Theme, preset: ColorPreset, colors?: ThemeColors) {
		try {
			localStorage.setItem('etlauncher-theme', JSON.stringify(theme));

			// Calculate the actual hue/chroma to store
			let hue = 172;
			let chroma = 0.18;
			if (preset === 'custom' && colors) {
				hue = colors.primaryHue ?? 172;
				chroma = colors.primaryChroma ?? 0.18;
			} else {
				const presetValues = themeStore.getPresetValues(preset);
				hue = presetValues.hue;
				chroma = presetValues.chroma;
			}
			localStorage.setItem('etlauncher-accent', JSON.stringify({ hue, chroma }));
		} catch {
			// localStorage may not be available
		}
	}

	async function handleThemeChange(theme: Theme) {
		await themeStore.applyThemeMode(theme);
		// Re-apply accent color and sidebar since primary color values differ between light/dark
		themeStore.applyAccentColor(settings?.colorPreset ?? 'default', settings?.customColors);
		themeStore.applySidebarStyle(settings?.sidebarStyle ?? 'default', settings?.customSidebarColor);
		saveThemeToLocalStorage(theme, settings?.colorPreset ?? 'default', settings?.customColors);
		await saveSettings({ theme });
	}

	async function handleColorPresetChange(preset: ColorPreset) {
		themeStore.applyAccentColor(preset);
		saveThemeToLocalStorage(settings?.theme ?? 'dark', preset);
		await saveSettings({ colorPreset: preset, customColors: undefined });
	}

	async function handleCustomColorChange() {
		const colors: ThemeColors = {
			primaryHue: customHue,
			primaryChroma: customChroma,
		};
		themeStore.applyAccentColor('custom', colors);
		saveThemeToLocalStorage(settings?.theme ?? 'dark', 'custom', colors);
		await saveSettings({ colorPreset: 'custom', customColors: colors });
	}

	function applyCustomColorPreview() {
		// Apply preview without saving
		themeStore.applyAccentColor('custom', {
			primaryHue: customHue,
			primaryChroma: customChroma,
		});
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
		<!-- Appearance Settings -->
		<section class="border-border bg-card space-y-4 border-2 p-4">
			<h3 class="text-muted-foreground text-sm tracking-wider uppercase">Appearance</h3>

			<!-- Theme Mode -->
			<div class="space-y-2">
				<span class="text-sm">Theme</span>
				<div class="flex gap-2">
					<button
						class="flex flex-1 flex-col items-center gap-1 border-2 p-3 transition-colors {settings.theme ===
						'system'
							? 'border-primary bg-primary/10'
							: 'border-border hover:border-primary/50'}"
						onclick={() => handleThemeChange('system')}
					>
						<Monitor class="h-5 w-5" />
						<span class="text-xs">System</span>
					</button>
					<button
						class="flex flex-1 flex-col items-center gap-1 border-2 p-3 transition-colors {settings.theme ===
						'light'
							? 'border-primary bg-primary/10'
							: 'border-border hover:border-primary/50'}"
						onclick={() => handleThemeChange('light')}
					>
						<Sun class="h-5 w-5" />
						<span class="text-xs">Light</span>
					</button>
					<button
						class="flex flex-1 flex-col items-center gap-1 border-2 p-3 transition-colors {settings.theme ===
						'dark'
							? 'border-primary bg-primary/10'
							: 'border-border hover:border-primary/50'}"
						onclick={() => handleThemeChange('dark')}
					>
						<Moon class="h-5 w-5" />
						<span class="text-xs">Dark</span>
					</button>
				</div>
				<p class="text-muted-foreground text-xs">Choose your preferred color scheme</p>
			</div>

			<!-- Accent Color -->
			<div class="space-y-2">
				<span class="text-sm">Accent Color</span>
				<div class="grid grid-cols-4 gap-2 sm:grid-cols-7">
					{#each colorPresets as preset (preset.id)}
						{@const isSelected = (settings.colorPreset ?? 'default') === preset.id}
						<button
							class="relative flex flex-col items-center gap-1 rounded-lg border-2 p-2 transition-all {isSelected
								? 'border-primary bg-primary/10'
								: 'border-border bg-muted/30 hover:border-primary/50 hover:bg-muted/50'}"
							onclick={() => handleColorPresetChange(preset.id)}
							title={preset.label}
						>
							<div
								class="ring-offset-background h-6 w-6 rounded-full ring-2 ring-offset-2 {isSelected
									? 'ring-primary'
									: 'ring-transparent'}"
								style="background: oklch(0.65 {preset.chroma} {preset.hue})"
							></div>
							<span class="text-xs">{preset.label}</span>
						</button>
					{/each}
					<button
						class="relative flex flex-col items-center gap-1 rounded-lg border-2 p-2 transition-all {settings.colorPreset ===
						'custom'
							? 'border-primary bg-primary/10'
							: 'border-border bg-muted/30 hover:border-primary/50 hover:bg-muted/50'}"
						onclick={() => handleColorPresetChange('custom')}
						title="Custom"
					>
						<Palette class="h-6 w-6 {settings.colorPreset === 'custom' ? 'text-primary' : ''}" />
						<span class="text-xs">Custom</span>
					</button>
				</div>
			</div>

			<!-- Custom Color Controls -->
			{#if settings.colorPreset === 'custom'}
				<div class="bg-muted/50 flex items-center gap-3 p-3">
					<span class="text-sm">Custom color:</span>
					<ColorPicker
						hue={customHue}
						saturation={customChroma / 0.35}
						oninput={(h, s) => {
							customHue = h;
							customChroma = Math.max(0.05, s * 0.35);
							applyCustomColorPreview();
						}}
						onchange={() => handleCustomColorChange()}
					/>
				</div>
			{/if}

			<!-- Font Family -->
			<div class="space-y-2">
				<span class="text-sm">Font</span>
				<div class="grid grid-cols-3 gap-2">
					<button
						class="flex flex-col items-center gap-1 border-2 p-3 transition-colors {(settings.fontFamily ??
							'pixel') === 'pixel'
							? 'border-primary bg-primary/10'
							: 'border-border hover:border-primary/50'}"
						onclick={() => {
							themeStore.applyFontFamily('pixel');
							saveSettings({ fontFamily: 'pixel', customFont: undefined });
						}}
					>
						<span class="text-sm" style="font-family: 'Silkscreen', monospace">Aa</span>
						<span class="text-xs">Pixel</span>
					</button>
					<button
						class="flex flex-col items-center gap-1 border-2 p-3 transition-colors {settings.fontFamily ===
						'system'
							? 'border-primary bg-primary/10'
							: 'border-border hover:border-primary/50'}"
						onclick={() => {
							themeStore.applyFontFamily('system');
							saveSettings({ fontFamily: 'system', customFont: undefined });
						}}
					>
						<span class="text-sm" style="font-family: system-ui, sans-serif">Aa</span>
						<span class="text-xs">System</span>
					</button>
					<button
						class="flex flex-col items-center gap-1 border-2 p-3 transition-colors {settings.fontFamily ===
						'custom'
							? 'border-primary bg-primary/10'
							: 'border-border hover:border-primary/50'}"
						onclick={() => {
							const fontFamily = settings.customFont?.family || 'Arial';
							themeStore.applyFontFamily('custom', { family: fontFamily });
							saveSettings({ fontFamily: 'custom', customFont: { family: fontFamily } });
						}}
					>
						<span
							class="text-sm"
							style="font-family: {settings.customFont?.family || 'Arial'}, sans-serif"
						>
							Aa
						</span>
						<span class="text-xs">Custom</span>
					</button>
				</div>

				{#if settings.fontFamily === 'custom'}
					<div class="mt-2 flex gap-2">
						<input
							type="text"
							class="border-border bg-background text-foreground focus:border-primary flex-1 border-2 px-3 py-2 text-sm focus:outline-none"
							placeholder="Font name (e.g., Arial, Roboto)"
							value={settings.customFont?.family || ''}
							oninput={(e) => {
								const fontFamily = e.currentTarget.value;
								themeStore.applyFontFamily('custom', { family: fontFamily });
							}}
							onchange={(e) => {
								const fontFamily = e.currentTarget.value;
								if (fontFamily) {
									saveSettings({ customFont: { family: fontFamily } });
								}
							}}
						/>
					</div>
					<p class="text-muted-foreground text-xs">Enter any font name installed on your system</p>
				{:else}
					<p class="text-muted-foreground text-xs">Choose the font style for the launcher</p>
				{/if}
			</div>

			<!-- Sidebar Style -->
			<div class="space-y-2">
				<span class="text-sm">Sidebar & Titlebar</span>
				<div class="grid grid-cols-3 gap-2">
					<button
						class="flex flex-col items-center gap-1 border-2 p-3 transition-colors {(settings.sidebarStyle ??
							'default') === 'default'
							? 'border-primary bg-primary/10'
							: 'border-border hover:border-primary/50'}"
						onclick={() => {
							themeStore.applySidebarStyle('default');
							saveSettings({ sidebarStyle: 'default', customSidebarColor: undefined });
						}}
					>
						<div class="bg-muted border-border h-6 w-8 border"></div>
						<span class="text-xs">Default</span>
					</button>
					<button
						class="flex flex-col items-center gap-1 border-2 p-3 transition-colors {settings.sidebarStyle ===
						'accent'
							? 'border-primary bg-primary/10'
							: 'border-border hover:border-primary/50'}"
						onclick={() => {
							themeStore.applySidebarStyle('accent');
							saveSettings({ sidebarStyle: 'accent', customSidebarColor: undefined });
						}}
					>
						<div class="bg-primary/20 border-primary/40 h-6 w-8 border"></div>
						<span class="text-xs">Accent</span>
					</button>
					<button
						class="flex flex-col items-center gap-1 border-2 p-3 transition-colors {settings.sidebarStyle ===
						'custom'
							? 'border-primary bg-primary/10'
							: 'border-border hover:border-primary/50'}"
						onclick={() => {
							themeStore.applySidebarStyle('custom', { hue: sidebarHue, chroma: sidebarChroma });
							saveSettings({
								sidebarStyle: 'custom',
								customSidebarColor: { hue: sidebarHue, chroma: sidebarChroma },
							});
						}}
					>
						<div
							class="h-6 w-8 border"
							style="background: oklch(0.2 {sidebarChroma} {sidebarHue}); border-color: oklch(0.35 {sidebarChroma} {sidebarHue})"
						></div>
						<span class="text-xs">Custom</span>
					</button>
				</div>

				{#if settings.sidebarStyle === 'custom'}
					<div class="bg-muted/50 flex items-center gap-3 p-3">
						<span class="text-sm">Custom color:</span>
						<ColorPicker
							hue={sidebarHue}
							saturation={sidebarChroma / 0.35}
							oninput={(h, s) => {
								sidebarHue = h;
								sidebarChroma = s * 0.35;
								themeStore.applySidebarStyle('custom', { hue: sidebarHue, chroma: sidebarChroma });
							}}
							onchange={() =>
								saveSettings({ customSidebarColor: { hue: sidebarHue, chroma: sidebarChroma } })}
						/>
					</div>
				{/if}
				<p class="text-muted-foreground text-xs">Customize the sidebar and titlebar appearance</p>
			</div>

			<!-- Hover Lift Toggle -->
			<div class="flex items-center justify-between">
				<div>
					<span class="text-sm">Disable hover lift</span>
					<p class="text-muted-foreground text-xs">
						Turn off the slight raise effect on buttons when hovering
					</p>
				</div>
				<Checkbox
					checked={settings.disableHoverLift ?? false}
					onCheckedChange={(checked) => {
						themeStore.applyHoverLift(!!checked);
						try {
							localStorage.setItem('etlauncher-disable-hover-lift', String(!!checked));
						} catch {
							// localStorage may be unavailable
						}
						saveSettings({ disableHoverLift: !!checked });
					}}
				/>
			</div>
		</section>

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
