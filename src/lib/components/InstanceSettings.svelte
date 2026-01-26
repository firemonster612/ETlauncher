<script lang="ts">
	import { Download, Loader2, FolderOpen, Copy, Upload } from '@lucide/svelte';
	import { save } from '@tauri-apps/plugin-dialog';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { Button } from '$lib/ui/button';
	import { Input } from '$lib/ui/input';
	import { RangeSlider } from '$lib/ui/slider';
	import { Textarea } from '$lib/ui/textarea';
	import * as Sheet from '$lib/ui/sheet';
	import * as RadioGroup from '$lib/ui/radio-group';
	import { instancesStore } from '$lib/stores/instances.svelte';
	import * as instanceService from '$lib/services/instance';
	import * as importService from '$lib/services/import';
	import IconPicker from './IconPicker.svelte';
	import { makeIconPath, type EntityIcon } from '$lib/utils/icons';
	import type { Instance, UpdateInstanceRequest } from '$lib/types';

	interface Props {
		instance: Instance;
		open: boolean;
		onClose: () => void;
	}

	let { instance, open: isOpen, onClose }: Props = $props();

	// Form state - initialize from instance
	let name = $state('');
	let iconPath = $state<string | undefined>(undefined);
	let javaPath = $state('');
	let memoryRange = $state<[number, number]>([512, 4096]);
	let jvmArgs = $state('');
	let gameArgs = $state('');
	let resolutionWidth = $state(0);
	let resolutionHeight = $state(0);
	let description = $state('');
	let author = $state('');

	let isSaving = $state(false);
	let saveError = $state<string | null>(null);
	let autoSaveTimeout: ReturnType<typeof setTimeout> | null = null;

	// Export state
	let exportFormat = $state<'mrpack' | 'curseforge'>('mrpack');

	const globalMemoryMinMb = $derived(settingsStore.settings?.memoryMinMb ?? 512);
	const globalMemoryMaxMb = $derived(settingsStore.settings?.memoryMaxMb ?? 4096);

	// Debounced auto-save function
	function scheduleAutoSave() {
		if (autoSaveTimeout) {
			clearTimeout(autoSaveTimeout);
		}
		autoSaveTimeout = setTimeout(() => {
			handleAutoSave();
		}, 1000);
	}

	// Reset form when instance changes
	$effect(() => {
		name = instance.name;
		iconPath = instance.iconPath;
		javaPath = instance.javaPath || '';
		memoryRange = [
			instance.memoryMinMb ?? globalMemoryMinMb,
			instance.memoryMaxMb ?? globalMemoryMaxMb,
		];
		jvmArgs = instance.jvmArgs || '';
		gameArgs = instance.gameArgs || '';
		resolutionWidth = instance.resolutionWidth || 0;
		resolutionHeight = instance.resolutionHeight || 0;
		description = instance.description || '';
		author = instance.author || '';
	});

	// Cleanup timeout on close
	$effect(() => {
		if (!isOpen && autoSaveTimeout) {
			clearTimeout(autoSaveTimeout);
			autoSaveTimeout = null;
		}
	});

	function handleIconSelect(icon: EntityIcon) {
		iconPath = makeIconPath(icon);
		scheduleAutoSave();
	}

	function buildUpdates(): UpdateInstanceRequest {
		return {
			name: name !== instance.name ? name : undefined,
			iconPath: iconPath !== instance.iconPath ? iconPath : undefined,
			javaPath: javaPath || undefined,
			memoryMinMb: memoryRange[0] !== globalMemoryMinMb ? memoryRange[0] : undefined,
			memoryMaxMb: memoryRange[1] !== globalMemoryMaxMb ? memoryRange[1] : undefined,
			jvmArgs: jvmArgs || undefined,
			gameArgs: gameArgs || undefined,
			resolutionWidth: resolutionWidth || undefined,
			resolutionHeight: resolutionHeight || undefined,
			description: description || undefined,
			author: author || undefined,
		};
	}

	async function handleAutoSave() {
		if (isSaving) return;

		isSaving = true;
		saveError = null;

		const updates = buildUpdates();

		// Remove undefined values
		Object.keys(updates).forEach((key) => {
			if (updates[key as keyof UpdateInstanceRequest] === undefined) {
				delete updates[key as keyof UpdateInstanceRequest];
			}
		});

		// Only save if there are actual changes
		if (Object.keys(updates).length === 0) {
			isSaving = false;
			return;
		}

		const result = await instancesStore.update(instance.id, updates);
		isSaving = false;

		if (!result) {
			saveError = instancesStore.error || 'Failed to save settings';
		}
	}

	function formatMemory(mb: number): string {
		if (mb >= 1024) {
			return `${(mb / 1024).toFixed(1)} GB`;
		}
		return `${mb} MB`;
	}

	async function handleReinstallLoader() {
		if (instance.loaderType === 'vanilla' || !instance.loaderVersion) return;
		await instancesStore.installLoader(instance.id, instance.loaderType, instance.loaderVersion);
	}

	async function handleOpenFolder() {
		await instanceService.openInstanceFolder(instance.id);
	}

	async function handleDuplicate() {
		await instancesStore.duplicate(instance.id, `${instance.name} (Copy)`);
		onClose();
	}

	let isExporting = $state(false);
	let exportError = $state<string | null>(null);
	let exportSuccess = $state(false);

	async function handleExport() {
		const extension = exportFormat === 'mrpack' ? 'mrpack' : 'zip';
		const filterName = exportFormat === 'mrpack' ? 'Modrinth Pack' : 'CurseForge Modpack';

		try {
			const filePath = await save({
				title: 'Export Instance',
				defaultPath: `${instance.name}.${extension}`,
				filters: [{ name: filterName, extensions: [extension] }],
			});

			if (!filePath) return;

			isExporting = true;
			exportError = null;
			exportSuccess = false;

			if (exportFormat === 'mrpack') {
				await instanceService.exportInstance(instance.id, filePath);
			} else {
				await importService.exportCurseForgeModpack(instance.id, filePath);
			}

			isExporting = false;
			exportSuccess = true;
			// Reset success message after 3 seconds
			setTimeout(() => {
				exportSuccess = false;
			}, 3000);
		} catch (e: unknown) {
			isExporting = false;
			exportError = e instanceof Error ? e.message : 'Export failed';
		}
	}
</script>

<Sheet.Root bind:open={isOpen} onOpenChange={(open) => !open && onClose()}>
	<Sheet.Content side="right" class="w-full overflow-y-auto sm:max-w-lg">
		<Sheet.Header class="border-border border-b px-6 pb-4">
			<Sheet.Title>Instance Settings</Sheet.Title>
			<Sheet.Description>
				Configure settings for <span class="font-medium">{instance.name}</span>
			</Sheet.Description>
		</Sheet.Header>

		<div class="space-y-8 px-6 py-6">
			<!-- Instance Name -->
			<div class="space-y-3">
				<label for="instance-name" class="text-sm font-medium">Instance Name</label>
				<Input
					id="instance-name"
					type="text"
					bind:value={name}
					placeholder="My Instance"
					oninput={scheduleAutoSave}
				/>
			</div>

			<!-- Instance Icon -->
			<div class="space-y-3">
				<p class="text-sm font-medium">Instance Icon</p>
				<p class="text-muted-foreground text-xs">Choose an icon for this instance</p>
				<IconPicker selected={iconPath} onSelect={handleIconSelect} />
			</div>

			<!-- Java Path -->
			<div class="space-y-3">
				<label for="java-path" class="text-sm font-medium">Java Path</label>
				<p class="text-muted-foreground text-xs">
					Leave empty to use the default Java installation
				</p>
				<Input
					id="java-path"
					type="text"
					bind:value={javaPath}
					placeholder="Auto-detect"
					oninput={scheduleAutoSave}
				/>
			</div>

			<!-- Memory Allocation -->
			<div class="space-y-4">
				<div>
					<p class="text-sm font-medium">Memory Allocation</p>
					<p class="text-muted-foreground mt-1 text-xs">Adjust the RAM allocated to Minecraft</p>
				</div>

				<div class="space-y-2">
					<div class="flex justify-between text-sm">
						<span>Min: <span class="text-primary">{formatMemory(memoryRange[0])}</span></span>
						<span>Max: <span class="text-primary">{formatMemory(memoryRange[1])}</span></span>
					</div>
					<RangeSlider
						min={512}
						max={16384}
						step={512}
						bind:value={memoryRange}
						onValueCommit={scheduleAutoSave}
					/>
					<div class="text-muted-foreground flex justify-between text-xs">
						<span>512 MB</span>
						<span>16 GB</span>
					</div>
				</div>
			</div>

			<!-- JVM Arguments -->
			<div class="space-y-3">
				<label for="jvm-args" class="text-sm font-medium">JVM Arguments</label>
				<p class="text-muted-foreground text-xs">
					Additional arguments passed to the Java Virtual Machine
				</p>
				<Textarea
					id="jvm-args"
					bind:value={jvmArgs}
					placeholder="-XX:+UseG1GC"
					rows={3}
					oninput={scheduleAutoSave}
				/>
			</div>

			<!-- Game Arguments -->
			<div class="space-y-3">
				<label for="game-args" class="text-sm font-medium">Game Arguments</label>
				<p class="text-muted-foreground text-xs">Additional arguments passed to Minecraft</p>
				<Textarea
					id="game-args"
					bind:value={gameArgs}
					placeholder="--width 1920 --height 1080"
					rows={2}
					oninput={scheduleAutoSave}
				/>
			</div>

			<!-- Resolution -->
			<div class="space-y-3">
				<p class="text-sm font-medium">Window Resolution</p>
				<p class="text-muted-foreground text-xs">Set to 0 for default resolution</p>
				<div class="flex items-center gap-3">
					<Input
						type="number"
						bind:value={resolutionWidth}
						placeholder="Width"
						min={0}
						class="w-24"
						oninput={scheduleAutoSave}
					/>
					<span class="text-muted-foreground">×</span>
					<Input
						type="number"
						bind:value={resolutionHeight}
						placeholder="Height"
						min={0}
						class="w-24"
						oninput={scheduleAutoSave}
					/>
				</div>
			</div>

			<!-- Instance Info (Read-only) -->
			<div class="border-border space-y-4 border-t pt-6">
				<h4 class="text-muted-foreground text-sm font-medium">Instance Info</h4>
				<div class="bg-muted/50 space-y-2 rounded p-4 text-sm">
					<p>
						<span class="text-muted-foreground">Minecraft:</span>
						<span class="font-medium">{instance.minecraftVersion}</span>
					</p>
					<p>
						<span class="text-muted-foreground">Loader:</span>
						<span class="font-medium capitalize">{instance.loaderType}</span>
						{#if instance.loaderVersion}
							<span class="text-muted-foreground">({instance.loaderVersion})</span>
						{/if}
					</p>
					{#if instance.modpackPlatform}
						<p>
							<span class="text-muted-foreground">Modpack:</span>
							<span class="font-medium">{instance.modpackPlatform}</span>
						</p>
					{/if}
				</div>
			</div>

			<!-- Quick Actions -->
			<div class="border-border space-y-4 border-t pt-6">
				<h4 class="text-muted-foreground text-sm font-medium">Quick Actions</h4>

				<Button variant="outline" class="w-full" onclick={handleOpenFolder}>
					<FolderOpen class="mr-2 h-4 w-4" />
					Open Game Folder
				</Button>

				<!-- Reinstall Loader Button -->
				{#if instance.loaderType !== 'vanilla' && instance.loaderVersion}
					<Button
						variant="outline"
						class="w-full"
						onclick={handleReinstallLoader}
						disabled={instancesStore.isInstallingLoader}
					>
						{#if instancesStore.isInstallingLoader}
							<Loader2 class="mr-2 h-4 w-4 animate-spin" />
							Reinstalling...
						{:else}
							<Download class="mr-2 h-4 w-4" />
							Reinstall {instance.loaderType}
							{instance.loaderVersion}
						{/if}
					</Button>
				{/if}

				<Button variant="outline" class="w-full" onclick={handleDuplicate}>
					<Copy class="mr-2 h-4 w-4" />
					Duplicate Instance
				</Button>

				</div>

			<!-- Export Section -->
			<div class="border-border space-y-4 border-t pt-6">
				<h4 class="text-muted-foreground text-sm font-medium">Export Instance</h4>

				<div class="space-y-3">
					<label for="export-description" class="text-sm font-medium">Description</label>
					<p class="text-muted-foreground text-xs">Optional description for the exported modpack</p>
					<Textarea
						id="export-description"
						bind:value={description}
						placeholder="A modpack for..."
						rows={2}
						oninput={scheduleAutoSave}
					/>
				</div>

				<div class="space-y-3">
					<label for="export-author" class="text-sm font-medium">Author</label>
					<p class="text-muted-foreground text-xs">Your name or username for the exported modpack</p>
					<Input
						id="export-author"
						type="text"
						bind:value={author}
						placeholder="Author name"
						oninput={scheduleAutoSave}
					/>
				</div>

				<div class="space-y-3">
					<p class="text-sm font-medium">Export Format</p>
					<p class="text-muted-foreground text-xs">Choose the format for sharing your modpack</p>
					<RadioGroup.Root bind:value={exportFormat} class="grid grid-cols-2 gap-3">
						<label
							for="format-mrpack"
							class="border-border hover:bg-muted/50 flex cursor-pointer items-start gap-3 rounded-lg border p-3 transition-colors has-[[data-state=checked]]:border-primary has-[[data-state=checked]]:bg-primary/5"
						>
							<RadioGroup.Item value="mrpack" id="format-mrpack" class="mt-0.5" />
							<div>
								<p class="text-sm font-medium">Modrinth (.mrpack)</p>
								<p class="text-muted-foreground text-xs">Best for Modrinth, Prism Launcher</p>
							</div>
						</label>
						<label
							for="format-curseforge"
							class="border-border hover:bg-muted/50 flex cursor-pointer items-start gap-3 rounded-lg border p-3 transition-colors has-[[data-state=checked]]:border-primary has-[[data-state=checked]]:bg-primary/5"
						>
							<RadioGroup.Item value="curseforge" id="format-curseforge" class="mt-0.5" />
							<div>
								<p class="text-sm font-medium">CurseForge (.zip)</p>
								<p class="text-muted-foreground text-xs">Best for CurseForge, ATLauncher</p>
							</div>
						</label>
					</RadioGroup.Root>
				</div>

				<Button variant="outline" class="w-full" onclick={handleExport} disabled={isExporting}>
					{#if isExporting}
						<Loader2 class="mr-2 h-4 w-4 animate-spin" />
						Exporting...
					{:else}
						<Upload class="mr-2 h-4 w-4" />
						Export as {exportFormat === 'mrpack' ? '.mrpack' : '.zip'}
					{/if}
				</Button>

				{#if exportError}
					<p class="text-destructive text-sm">{exportError}</p>
				{/if}

				{#if exportSuccess}
					<p class="text-sm text-green-500">Export completed successfully!</p>
				{/if}
			</div>
		</div>

		{#if saveError}
			<div
				class="bg-destructive/10 border-destructive text-destructive mx-6 mb-4 rounded border p-4 text-sm"
			>
				{saveError}
			</div>
		{/if}

		{#if instancesStore.loaderInstallError}
			<div
				class="bg-destructive/10 border-destructive text-destructive mx-6 mb-4 rounded border p-4 text-sm"
			>
				{instancesStore.loaderInstallError}
			</div>
		{/if}

		<Sheet.Footer class="border-border flex items-center justify-between border-t px-6 pt-4">
			<span class="text-muted-foreground text-xs">
				{#if isSaving}
					Saving...
				{:else}
					Auto-saves on change
				{/if}
			</span>
			<Button variant="outline" onclick={onClose} disabled={isSaving}>
				Close
			</Button>
		</Sheet.Footer>
	</Sheet.Content>
</Sheet.Root>
