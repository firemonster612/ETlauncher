<script lang="ts">
	import {
		AlertTriangle,
		ArrowLeft,
		Blocks,
		FileArchive,
		FolderOpen,
		Loader2,
		Package,
	} from '@lucide/svelte';
	import { open } from '@tauri-apps/plugin-dialog';
	import * as importService from '$lib/services/import';
	import * as modpackService from '$lib/services/modpack';
	import { alertDialogStore } from '$lib/stores/alertDialog.svelte';
	import { instancesStore } from '$lib/stores/instances.svelte';
	import type { ImportAnalysis, ImportSourceType } from '$lib/types';
	import { Button } from '$lib/ui/button';
	import { Input } from '$lib/ui/input';

	interface Props {
		open: boolean;
		onClose: () => void;
	}

	let { open: isOpen, onClose }: Props = $props();

	type Step = 'select' | 'analyze' | 'configure' | 'error';

	let step = $state<Step>('select');
	let selectedType = $state<'mrpack' | 'curseforge' | 'folder' | null>(null);
	let filePath = $state('');
	let analysis = $state<ImportAnalysis | null>(null);
	let instanceName = $state('');
	let errorMessage = $state<string | null>(null);
	let dialogError = $state<string | null>(null);

	function resetState() {
		step = 'select';
		selectedType = null;
		filePath = '';
		analysis = null;
		instanceName = '';
		errorMessage = null;
		dialogError = null;
	}

	function handleClose() {
		resetState();
		onClose();
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
			let msg = 'Analysis failed';
			if (e instanceof Error) {
				msg = e.message;
			} else if (typeof e === 'object' && e !== null && 'message' in e) {
				msg = String((e as { message: unknown }).message);
			} else if (typeof e === 'string') {
				msg = e;
			}
			errorMessage = msg;
			step = 'error';
		}
	}

	function handleImport() {
		// Capture values before closing the wizard
		const importType = selectedType;
		const importPath = filePath;
		const importName = instanceName.trim();
		const importAnalysis = analysis;

		// Close the wizard immediately — the task drawer shows progress
		handleClose();

		// Build the import promise based on type
		let importPromise: Promise<unknown>;

		switch (importType) {
			case 'mrpack':
				importPromise = modpackService.importModpackFile(importPath, importName || undefined);
				break;
			case 'curseforge':
				importPromise = importService.importCurseForgeZip(importPath, importName || undefined);
				break;
			case 'folder':
				if (!importAnalysis) return;
				importPromise = importService.importFromFolder(
					importPath,
					importName || 'Imported Instance',
					importAnalysis.sourceType
				);
				break;
			default:
				return;
		}

		// Fire and forget — refresh instances on success, show alert on failure
		importPromise
			.then(() => {
				instancesStore.load();
			})
			.catch((e: unknown) => {
				let msg = 'Import failed';
				if (e instanceof Error) {
					msg = e.message;
				} else if (typeof e === 'object' && e !== null && 'message' in e) {
					msg = String((e as { message: unknown }).message);
				} else if (typeof e === 'string') {
					msg = e;
				}
				alertDialogStore.alert({
					title: 'Import Failed',
					message: msg,
					type: 'error',
				});
			});
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
						disabled={!instanceName.trim()}
					>
						Import
					</Button>
				</div>

			{:else if step === 'error'}
				<!-- Analysis Error -->
				<div class="flex flex-col items-center gap-4 py-4">
					<AlertTriangle class="text-destructive h-12 w-12" />
					<div class="text-center">
						<p class="font-medium">Analysis Failed</p>
						<p class="text-muted-foreground text-sm">{errorMessage}</p>
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
