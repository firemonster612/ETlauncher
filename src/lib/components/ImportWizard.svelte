<script lang="ts">
	import {
		Loader2,
		CheckCircle,
		AlertTriangle,
		FolderOpen,
		FileArchive,
		ArrowLeft,
		Package,
		Blocks,
	} from '@lucide/svelte';
	import { open } from '@tauri-apps/plugin-dialog';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import * as importService from '$lib/services/import';
	import * as modpackService from '$lib/services/modpack';
	import { Button } from '$lib/ui/button';
	import { Input } from '$lib/ui/input';
	import DownloadProgress from './DownloadProgress.svelte';
	import type {
		Instance,
		ImportAnalysis,
		ImportSourceType,
		ImportProgress,
		ModpackInstallProgress,
	} from '$lib/types';

	interface Props {
		open: boolean;
		onClose: (instanceCreated: boolean) => void;
	}

	let { open: isOpen, onClose }: Props = $props();

	type Step = 'select' | 'analyze' | 'configure' | 'importing' | 'complete' | 'error';

	let step = $state<Step>('select');
	let selectedType = $state<'mrpack' | 'curseforge' | 'folder' | null>(null);
	let filePath = $state('');
	let analysis = $state<ImportAnalysis | null>(null);
	let instanceName = $state('');
	let isImporting = $state(false);
	let importProgress = $state<ModpackInstallProgress | ImportProgress | null>(null);
	let result = $state<{ success: boolean; instance?: Instance; error?: string } | null>(null);
	let dialogError = $state<string | null>(null);

	function resetState() {
		step = 'select';
		selectedType = null;
		filePath = '';
		analysis = null;
		instanceName = '';
		isImporting = false;
		importProgress = null;
		result = null;
		dialogError = null;
	}

	function handleClose(instanceCreated: boolean = false) {
		resetState();
		onClose(instanceCreated);
	}

	async function selectMrpack() {
		try {
			const path = await open({
				title: 'Import Modrinth .mrpack File',
				filters: [{ name: 'Modrinth Pack', extensions: ['mrpack'] }],
				multiple: false,
			});

			if (!path) return;

			selectedType = 'mrpack';
			filePath = path as string;
			step = 'configure';
			// Extract suggested name from filename
			const filename = filePath.split(/[/\\]/).pop() || '';
			instanceName = filename.replace('.mrpack', '');
		} catch {
			dialogError = 'Failed to open file dialog. Please try again.';
		}
	}

	async function selectCurseForge() {
		try {
			const path = await open({
				title: 'Import CurseForge .zip Modpack',
				filters: [{ name: 'CurseForge Modpack', extensions: ['zip'] }],
				multiple: false,
			});

			if (!path) return;

			selectedType = 'curseforge';
			filePath = path as string;
			step = 'analyze';
			await analyzeSource();
		} catch {
			dialogError = 'Failed to open file dialog. Please try again.';
		}
	}

	async function selectFolder() {
		try {
			const path = await open({
				title: 'Select Minecraft or Instance Folder',
				directory: true,
				multiple: false,
			});

			if (!path) return;

			selectedType = 'folder';
			filePath = path as string;
			step = 'analyze';
			await analyzeSource();
		} catch {
			dialogError = 'Failed to open folder dialog. Please try again.';
		}
	}

	async function analyzeSource() {
		try {
			analysis = await importService.analyzeImportSource(filePath);
			instanceName = analysis.suggestedName || 'Imported Instance';
			step = 'configure';
		} catch (e: unknown) {
			let errorMessage = 'Analysis failed';
			if (e instanceof Error) {
				errorMessage = e.message;
			} else if (typeof e === 'object' && e !== null && 'message' in e) {
				errorMessage = String((e as { message: unknown }).message);
			} else if (typeof e === 'string') {
				errorMessage = e;
			}
			result = { success: false, error: errorMessage };
			step = 'error';
		}
	}

	async function handleImport() {
		isImporting = true;
		importProgress = null;
		step = 'importing';

		let unlisten: UnlistenFn | undefined;

		try {
			// Listen for progress events
			if (selectedType === 'mrpack') {
				unlisten = await listen<ModpackInstallProgress>('modpack_install_progress', (event) => {
					importProgress = event.payload;
				});
			} else {
				unlisten = await listen<ImportProgress>('import_progress', (event) => {
					importProgress = event.payload;
				});
			}

			let instance: Instance;

			switch (selectedType) {
				case 'mrpack':
					instance = await modpackService.importModpackFile(
						filePath,
						instanceName.trim() || undefined
					);
					break;
				case 'curseforge':
					instance = await importService.importCurseForgeZip(
						filePath,
						instanceName.trim() || undefined
					);
					break;
				case 'folder':
					if (!analysis) throw new Error('No analysis available');
					instance = await importService.importFromFolder(
						filePath,
						instanceName.trim() || 'Imported Instance',
						analysis.sourceType
					);
					break;
				default:
					throw new Error('Unknown import type');
			}

			result = { success: true, instance };
			step = 'complete';
		} catch (e: unknown) {
			let errorMessage = 'Import failed';
			if (e instanceof Error) {
				errorMessage = e.message;
			} else if (typeof e === 'object' && e !== null && 'message' in e) {
				errorMessage = String((e as { message: unknown }).message);
			} else if (typeof e === 'string') {
				errorMessage = e;
			}
			result = { success: false, error: errorMessage };
			step = 'error';
		} finally {
			unlisten?.();
			isImporting = false;
			importProgress = null;
		}
	}

	function getSourceTypeLabel(sourceType: ImportSourceType): string {
		switch (sourceType) {
			case 'vanillaMinecraft':
				return 'Vanilla .minecraft';
			case 'multiMC':
				return 'MultiMC Instance';
			case 'prismLauncher':
				return 'Prism Launcher Instance';
			case 'curseForgeZip':
				return 'CurseForge Modpack';
			default:
				return 'Unknown';
		}
	}

	function formatLoaderType(loaderType: string): string {
		if (loaderType === 'vanilla') return 'Vanilla';
		return loaderType.charAt(0).toUpperCase() + loaderType.slice(1);
	}
</script>

{#if isOpen}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<div class="bg-card border-border mx-4 w-full max-w-lg space-y-4 border-2 p-6">
			{#if step === 'select'}
				<!-- Step 1: Select Import Type -->
				<h2 class="text-lg font-bold">Import Instance</h2>
				<p class="text-muted-foreground text-sm">
					Choose what type of content you want to import:
				</p>

				<div class="space-y-2 pt-2">
					<button
						type="button"
						class="hover:bg-muted/50 border-border flex w-full items-center gap-4 border-2 p-4 text-left transition-colors"
						onclick={() => { dialogError = null; selectMrpack(); }}
					>
						<Package class="text-primary h-8 w-8 shrink-0" />
						<div>
							<p class="font-medium">Modrinth .mrpack</p>
							<p class="text-muted-foreground text-xs">Import a Modrinth modpack file</p>
						</div>
					</button>

					<button
						type="button"
						class="hover:bg-muted/50 border-border flex w-full items-center gap-4 border-2 p-4 text-left transition-colors"
						onclick={() => { dialogError = null; selectCurseForge(); }}
					>
						<FileArchive class="text-orange-500 h-8 w-8 shrink-0" />
						<div>
							<p class="font-medium">CurseForge .zip</p>
							<p class="text-muted-foreground text-xs">Import a CurseForge modpack file</p>
						</div>
					</button>

					<button
						type="button"
						class="hover:bg-muted/50 border-border flex w-full items-center gap-4 border-2 p-4 text-left transition-colors"
						onclick={() => { dialogError = null; selectFolder(); }}
					>
						<FolderOpen class="text-blue-500 h-8 w-8 shrink-0" />
						<div>
							<p class="font-medium">Minecraft / Instance Folder</p>
							<p class="text-muted-foreground text-xs">
								Import from .minecraft, MultiMC, or Prism Launcher
							</p>
						</div>
					</button>
				</div>

				{#if dialogError}
					<div class="bg-destructive/10 text-destructive flex items-center gap-2 rounded p-3 text-sm">
						<AlertTriangle class="h-4 w-4 shrink-0" />
						<span>{dialogError}</span>
					</div>
				{/if}

				<div class="flex justify-end pt-2">
					<Button variant="outline" onclick={() => handleClose()}>Cancel</Button>
				</div>

			{:else if step === 'analyze'}
				<!-- Step 2: Analyzing -->
				<div class="flex flex-col items-center gap-4 py-8">
					<Loader2 class="text-primary h-12 w-12 animate-spin" />
					<div class="text-center">
						<p class="font-medium">Analyzing source...</p>
						<p class="text-muted-foreground text-sm">Detecting version and content</p>
					</div>
				</div>

			{:else if step === 'configure'}
				<!-- Step 3: Configure Import -->
				<div class="flex items-center gap-2">
					<button type="button" onclick={() => (step = 'select')} class="hover:bg-muted rounded p-1">
						<ArrowLeft class="h-5 w-5" />
					</button>
					<h2 class="text-lg font-bold">Configure Import</h2>
				</div>

				{#if analysis}
					<div class="bg-muted/50 space-y-2 rounded p-4 text-sm">
						<div class="flex items-center gap-2">
							<Blocks class="text-muted-foreground h-4 w-4" />
							<span class="text-muted-foreground">Source:</span>
							<span class="font-medium">{getSourceTypeLabel(analysis.sourceType)}</span>
						</div>
						{#if analysis.minecraftVersion}
							<p>
								<span class="text-muted-foreground">Minecraft:</span>
								<span class="font-medium">{analysis.minecraftVersion}</span>
							</p>
						{/if}
						{#if analysis.loaderType !== 'vanilla'}
							<p>
								<span class="text-muted-foreground">Loader:</span>
								<span class="font-medium">{formatLoaderType(analysis.loaderType)}</span>
								{#if analysis.loaderVersion}
									<span class="text-muted-foreground">({analysis.loaderVersion})</span>
								{/if}
							</p>
						{/if}
						{#if analysis.modCount > 0}
							<p>
								<span class="text-muted-foreground">Mods:</span>
								<span class="font-medium">{analysis.modCount}</span>
							</p>
						{/if}
						<div class="flex gap-4 text-xs">
							{#if analysis.hasConfig}
								<span class="text-muted-foreground">Config</span>
							{/if}
							{#if analysis.hasResourcepacks}
								<span class="text-muted-foreground">Resource Packs</span>
							{/if}
							{#if analysis.hasShaderpacks}
								<span class="text-muted-foreground">Shaders</span>
							{/if}
						</div>
					</div>
				{/if}

				<div>
					<label for="instance-name" class="text-muted-foreground mb-1 block text-sm">
						Instance Name
					</label>
					<Input id="instance-name" type="text" bind:value={instanceName} placeholder="My Instance" />
				</div>

				<div class="flex gap-2 pt-2">
					<Button variant="outline" class="flex-1" onclick={() => (step = 'select')}>Back</Button>
					<Button
						class="flex-1"
						onclick={handleImport}
						disabled={!instanceName.trim() || isImporting}
					>
						Import
					</Button>
				</div>

			{:else if step === 'importing'}
				<!-- Step 4: Importing -->
				<div class="flex flex-col items-center gap-4 py-4">
					<Loader2 class="text-primary h-10 w-10 animate-spin" />
					<div class="text-center">
						<p class="font-medium">Importing {instanceName}...</p>
						<p class="text-muted-foreground text-sm">This may take a while</p>
					</div>

					{#if importProgress && 'totalItems' in importProgress}
						<div class="w-full">
							<DownloadProgress
								stage={importProgress.stage}
								progress={importProgress.progress}
								currentItem={importProgress.currentItem}
								totalItems={importProgress.totalItems}
								completedItems={importProgress.completedItems}
							/>
						</div>
					{:else if importProgress && 'stage' in importProgress}
						<div class="w-full space-y-2">
							<div class="flex justify-between text-sm">
								<span>{importProgress.stage}</span>
								<span>{importProgress.progress}%</span>
							</div>
							<div class="bg-muted h-2 overflow-hidden rounded-full">
								<div
									class="bg-primary h-full transition-all duration-300"
									style="width: {importProgress.progress}%"
								></div>
							</div>
							{#if importProgress.currentItem}
								<p class="text-muted-foreground truncate text-xs">{importProgress.currentItem}</p>
							{/if}
						</div>
					{/if}
				</div>

			{:else if step === 'complete'}
				<!-- Step 5: Complete -->
				<div class="flex flex-col items-center gap-4 py-4">
					<CheckCircle class="h-12 w-12 text-green-500" />
					<div class="text-center">
						<p class="font-medium">Import Complete!</p>
						<p class="text-muted-foreground text-sm">
							Instance "{result?.instance?.name}" has been created.
						</p>
					</div>
				</div>

				<div class="flex justify-end">
					<Button onclick={() => handleClose(true)}>Done</Button>
				</div>

			{:else if step === 'error'}
				<!-- Step 6: Error -->
				<div class="flex flex-col items-center gap-4 py-4">
					<AlertTriangle class="text-destructive h-12 w-12" />
					<div class="text-center">
						<p class="font-medium">Import Failed</p>
						<p class="text-muted-foreground text-sm">{result?.error}</p>
					</div>
				</div>

				<div class="flex gap-2">
					<Button variant="outline" class="flex-1" onclick={() => handleClose()}>Cancel</Button>
					<Button class="flex-1" onclick={() => (step = 'select')}>Try Again</Button>
				</div>
			{/if}
		</div>
	</div>
{/if}
