<script lang="ts">
	import {
		AlertTriangle,
		CheckCircle,
		FileDown,
		Layers,
		Loader2,
		Plus,
		Search,
		Trash2,
	} from '@lucide/svelte';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import ContentBrowser from '$lib/components/ContentBrowser.svelte';
	import ImportWizard from '$lib/components/ImportWizard.svelte';
	import InstanceCard from '$lib/components/InstanceCard.svelte';
	import InstanceDetailModal from '$lib/components/InstanceDetailModal.svelte';
	import InstanceSettings from '$lib/components/InstanceSettings.svelte';
	import { accountsStore } from '$lib/stores/accounts.svelte';
	import { alertDialogStore } from '$lib/stores/alertDialog.svelte';
	import { instancesStore } from '$lib/stores/instances.svelte';
	import { launchStore } from '$lib/stores/launch.svelte';
	import { settingsStore } from '$lib/stores/settings.svelte';
	import { versionsStore } from '$lib/stores/versions.svelte';
	import type { ContentType, Instance, LoaderType } from '$lib/types';
	import { Button } from '$lib/ui/button';
	import { Input } from '$lib/ui/input';
	import { LoaderSelect } from '$lib/ui/loader-select';
	import * as Select from '$lib/ui/select';

	let search = $state('');
	let showCreateModal = $state(false);
	let showDeleteModal = $state(false);
	let instanceToDelete = $state<string | null>(null);

	// Create form state
	let createName = $state('');
	let createVersion = $state('');
	let createLoader = $state<LoaderType>('vanilla');
	let createLoaderVersion = $state('');
	let isCreating = $state(false);

	// Content browser state
	let showContentBrowser = $state(false);
	let contentBrowserInstance = $state<Instance | null>(null);
	let contentBrowserContentType = $state<ContentType | undefined>(undefined);

	// Settings modal state
	let showSettings = $state(false);
	let settingsInstance = $state<Instance | null>(null);

	// Detail modal state
	let showDetailModal = $state(false);
	let detailInstance = $state<Instance | null>(null);
	let isClosingDetailModal = $state(false);

	// Export state
	let showExportModal = $state(false);
	let exportInstance = $state<Instance | null>(null);
	let isExporting = $state(false);
	let exportResult = $state<{ success: boolean; path?: string; error?: string } | null>(null);

	// Import wizard state
	let showImportWizard = $state(false);

	function openSettings(instance: Instance) {
		settingsInstance = instance;
		showSettings = true;
	}

	function closeSettings() {
		showSettings = false;
		settingsInstance = null;
	}

	function openContentBrowser(instance: Instance, contentType?: ContentType) {
		contentBrowserInstance = instance;
		contentBrowserContentType = contentType;
		showContentBrowser = true;
	}

	function closeContentBrowser(contentWasInstalled: boolean) {
		showContentBrowser = false;
		contentBrowserInstance = null;
		contentBrowserContentType = undefined;

		// Refresh instance list if content was installed
		// This ensures the instance detail view shows updated content counts
		if (contentWasInstalled) {
			instancesStore.load();
		}
	}

	function openDetailModal(instance: Instance) {
		detailInstance = instance;
		showDetailModal = true;
	}

	function closeDetailModal() {
		// Set flag to prevent effect from reopening modal during URL clear
		isClosingDetailModal = true;
		showDetailModal = false;
		detailInstance = null;
		// Clear query param without adding to history
		if (page.url.searchParams.has('id')) {
			goto(resolve('/instances'), { replaceState: true }).then(() => {
				isClosingDetailModal = false;
			});
		} else {
			isClosingDetailModal = false;
		}
	}

	function handleInstanceUpdated(updatedInstance: Instance) {
		// Update the detail instance if it's the same one
		if (detailInstance && detailInstance.id === updatedInstance.id) {
			detailInstance = updatedInstance;
		}
		// Refresh instance list after update
		instancesStore.load();
	}

	onMount(() => {
		instancesStore.load();
		settingsStore.load();
		versionsStore.load();
		accountsStore.load();
		// launchStore is initialized at app layout level
	});

	// Open detail modal if id query param is present
	$effect(() => {
		const instanceId = page.url.searchParams.get('id');
		if (instanceId && instancesStore.instances.length > 0 && !detailInstance && !isClosingDetailModal) {
			const instance = instancesStore.instances.find((i) => i.id === instanceId);
			if (instance) {
				detailInstance = instance;
				showDetailModal = true;
			}
		}
	});

	// Handle action query param (create/import from homepage)
	$effect(() => {
		const action = page.url.searchParams.get('action');
		if (action === 'create' && !showCreateModal) {
			showCreateModal = true;
			goto(resolve('/instances'), { replaceState: true });
		} else if (action === 'import' && !showImportWizard) {
			showImportWizard = true;
			goto(resolve('/instances'), { replaceState: true });
		}
	});

	// Set default version when versions load
	$effect(() => {
		if (versionsStore.latestRelease && !createVersion) {
			createVersion = versionsStore.latestRelease;
		}
	});

	const filteredInstances = $derived(
		instancesStore.instances.filter((instance) =>
			instance.name.toLowerCase().includes(search.toLowerCase())
		)
	);

	async function handleCreate() {
		if (!createName.trim()) return;

		isCreating = true;

		// Capture loader values before resetting form
		const loader = createLoader;
		const loaderVersion = createLoaderVersion;

		// Just create the instance record (fast), then close the modal
		// Loader install + game file setup runs in the background via the task drawer
		try {
			const instance = await instancesStore.createOnly({
				name: createName.trim(),
				minecraftVersion: createVersion,
				loaderType: loader === 'vanilla' ? undefined : loader,
				loaderVersion: loader !== 'vanilla' ? loaderVersion : undefined,
			});

			if (instance) {
				// Close modal immediately
				createName = '';
				createVersion = versionsStore.latestRelease ?? '';
				createLoader = 'vanilla';
				createLoaderVersion = '';
				closeCreateModal();

				// Kick off loader install + setup in background (task drawer shows progress)
				instancesStore.setupInBackground(instance.id, {
					loaderType: loader === 'vanilla' ? undefined : loader,
					loaderVersion: loader !== 'vanilla' ? loaderVersion : undefined,
				});
			}
		} finally {
			isCreating = false;
		}
	}

	function closeCreateModal() {
		showCreateModal = false;
	}

	function confirmDelete(instanceId: string) {
		instanceToDelete = instanceId;
		showDeleteModal = true;
	}

	async function handleDelete(deleteFiles: boolean) {
		if (instanceToDelete) {
			await instancesStore.delete(instanceToDelete, deleteFiles);
			showDeleteModal = false;
			instanceToDelete = null;
		}
	}

	async function handleLaunch(instanceId: string) {
		// Check if user is logged in
		if (!accountsStore.activeAccount) {
			alertDialogStore.alert({
				title: 'Account Required',
				message:
					'Please log in with a Microsoft account first. Go to Accounts and set one as active.',
				type: 'warning',
			});
			return;
		}

		console.log('Launching instance:', instanceId, 'with account:', accountsStore.activeAccount.id);
		const result = await launchStore.launch(instanceId, accountsStore.activeAccount.id);
		console.log('Launch result:', result);
	}

	async function handleKill(instanceId: string) {
		console.log('Killing instance:', instanceId);
		await launchStore.kill(instanceId);
	}

	function getInstanceStatus(instanceId: string): string | null {
		const state = launchStore.launchStates.get(instanceId);
		if (!state) return null;
		return state.status.status;
	}

	function closeExportModal() {
		showExportModal = false;
		exportInstance = null;
		exportResult = null;
	}

	function handleImportWizardClose() {
		showImportWizard = false;
	}
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between gap-4">
		<h1 class="text-2xl">Instances</h1>
		<div class="flex max-w-md flex-1 items-center gap-4">
			<div class="relative flex-1">
				<Search
					class="text-muted-foreground absolute top-1/2 left-3 z-10 h-4 w-4 -translate-y-1/2"
				/>
				<Input type="text" placeholder="Search..." bind:value={search} class="pl-9" />
			</div>
			<Button variant="outline" onclick={() => (showImportWizard = true)} title="Import instance">
				<FileDown class="mr-2 h-4 w-4" />
				Import
			</Button>
			<Button onclick={() => (showCreateModal = true)}>
				<Plus class="mr-2 h-4 w-4" />
				New
			</Button>
		</div>
	</div>

	<!-- Error Display -->
	{#if instancesStore.error}
		<div class="bg-destructive/10 border-destructive text-destructive border-2 p-4 text-sm">
			{instancesStore.error}
			<button class="ml-2 underline" onclick={() => instancesStore.clearError()}>Dismiss</button>
		</div>
	{/if}

	{#if launchStore.error}
		<div class="bg-destructive/10 border-destructive text-destructive border-2 p-4 text-sm">
			Launch error: {launchStore.error}
			<button class="ml-2 underline" onclick={() => launchStore.clearError()}>Dismiss</button>
		</div>
	{/if}

	{#if instancesStore.loaderInstallError}
		<div class="bg-destructive/10 border-destructive text-destructive border-2 p-4 text-sm">
			Loader installation error: {instancesStore.loaderInstallError}
			<button class="ml-2 underline" onclick={() => instancesStore.clearLoaderError()}
				>Dismiss</button
			>
		</div>
	{/if}

	{#if instancesStore.isLoading}
		<div class="text-muted-foreground">Loading instances...</div>
	{:else if filteredInstances.length === 0}
		<div class="border-border bg-card/50 border-2 border-dashed p-12 text-center">
			<Layers class="text-muted-foreground/50 mx-auto h-12 w-12" />
			<p class="text-muted-foreground mt-4 text-sm">
				{search ? 'No instances match your search' : 'No instances yet'}
			</p>
			{#if !search}
				<Button class="mt-4" onclick={() => (showCreateModal = true)}>
					<Plus class="mr-2 h-4 w-4" />
					Create Instance
				</Button>
			{/if}
		</div>
	{:else}
		<!-- Instance Grid -->
		<div
			class="grid gap-4"
			style="grid-template-columns: repeat(auto-fill, 320px); justify-content: start;"
		>
		{#each filteredInstances as instance (instance.id)}
			{@const status = getInstanceStatus(instance.id)}
			<InstanceCard
				{instance}
				{status}
				onLaunch={handleLaunch}
				onKill={handleKill}
				onOpenSettings={openSettings}
				onOpenContentBrowser={openContentBrowser}
				onDelete={confirmDelete}
				onCardClick={openDetailModal}
			/>
			{/each}
		</div>
	{/if}
</div>

<!-- Create Instance Modal -->
{#if showCreateModal}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<div class="bg-card border-border mx-4 w-full max-w-md space-y-4 border-2 p-6">
			<h2 class="text-lg font-bold">Create New Instance</h2>

				<div class="space-y-4">
					<div>
						<label for="name" class="text-muted-foreground mb-1 block text-sm">Instance Name</label>
						<Input id="name" type="text" bind:value={createName} placeholder="My Instance" />
					</div>

					<div class="space-y-4">
						<div>
							<span class="text-muted-foreground mb-1 block text-sm">Minecraft Version</span>
							<Select.Root
								type="single"
								bind:value={createVersion}
								disabled={versionsStore.isLoading}
							>
								<Select.Trigger class="border-border bg-background w-full border-2">
									{#if versionsStore.isLoading}
										Loading versions...
									{:else if createVersion}
										{createVersion}
									{:else}
										Select version...
									{/if}
								</Select.Trigger>
								<Select.Content class="border-border bg-card max-h-[300px] border-2">
									{#each versionsStore.versions as version (version.id)}
										<Select.Item value={version.id} label={version.id}>
											{version.id}
											{#if version.type === 'snapshot'}
												<span class="text-muted-foreground ml-1">(snapshot)</span>
											{:else if version.type === 'old_beta'}
												<span class="text-muted-foreground ml-1">(beta)</span>
											{:else if version.type === 'old_alpha'}
												<span class="text-muted-foreground ml-1">(alpha)</span>
											{/if}
										</Select.Item>
									{/each}
								</Select.Content>
							</Select.Root>
						</div>

						<div>
							<LoaderSelect
								loaderType={createLoader}
								loaderVersion={createLoaderVersion}
								minecraftVersion={createVersion}
								onLoaderTypeChange={(loader) => (createLoader = loader)}
								onLoaderVersionChange={(version) => (createLoaderVersion = version)}
							/>
						</div>
					</div>
				</div>

				<div class="flex gap-2 pt-2">
					<Button
						variant="outline"
						class="flex-1"
						onclick={closeCreateModal}
						disabled={isCreating}
					>
						Cancel
					</Button>
					<Button
						class="flex-1"
						onclick={handleCreate}
						disabled={!createName.trim() || isCreating}
					>
						{#if isCreating}
							Creating...
						{:else}
							Create
						{/if}
					</Button>
				</div>
		</div>
	</div>
{/if}

<!-- Delete Confirmation Modal -->
{#if showDeleteModal}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<div class="bg-card border-border mx-4 w-full max-w-md space-y-4 border-2 p-6">
			<h2 class="text-lg font-bold">Delete Instance</h2>
			<p class="text-muted-foreground text-sm">How would you like to delete this instance?</p>

			<div class="flex flex-col gap-2 pt-2">
				<Button variant="outline" onclick={() => handleDelete(false)} class="justify-start">
					Remove from launcher only
					<span class="text-muted-foreground ml-2 text-xs">(keeps files)</span>
				</Button>
				<Button variant="destructive" onclick={() => handleDelete(true)} class="justify-start">
					<Trash2 class="mr-2 h-4 w-4" />
					Delete everything
					<span class="ml-2 text-xs">(permanent)</span>
				</Button>
				<Button variant="ghost" onclick={() => (showDeleteModal = false)}>Cancel</Button>
			</div>
		</div>
	</div>
{/if}

<!-- Content Browser -->
{#if showContentBrowser && contentBrowserInstance}
	<ContentBrowser
		instanceId={contentBrowserInstance.id}
		instanceName={contentBrowserInstance.name}
		mcVersion={contentBrowserInstance.minecraftVersion}
		loaderType={contentBrowserInstance.loaderType}
		initialContentType={contentBrowserContentType}
		onClose={closeContentBrowser}
	/>
{/if}

<!-- Instance Settings -->
{#if settingsInstance}
	<InstanceSettings instance={settingsInstance} open={showSettings} onClose={closeSettings} />
{/if}

<!-- Instance Detail Modal -->
{#if detailInstance}
	{@const status = getInstanceStatus(detailInstance.id)}
	<InstanceDetailModal
		instance={detailInstance}
		open={showDetailModal}
		{status}
		onClose={closeDetailModal}
		onLaunch={handleLaunch}
		onKill={handleKill}
		onOpenSettings={openSettings}
		onOpenContentBrowser={openContentBrowser}
		onInstanceUpdated={handleInstanceUpdated}
	/>
{/if}

<!-- Export Modal -->
{#if showExportModal}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<div class="bg-card border-border mx-4 w-full max-w-md space-y-4 border-2 p-6">
			<h2 class="text-lg font-bold">Export Instance</h2>

			{#if isExporting}
				<div class="flex items-center gap-3 py-4">
					<Loader2 class="text-primary h-6 w-6 animate-spin" />
					<div>
						<p class="font-medium">Exporting {exportInstance?.name}...</p>
						<p class="text-muted-foreground text-sm">Creating .mrpack file</p>
					</div>
				</div>
			{:else if exportResult?.success}
				<div class="py-4">
					<div class="mb-3 flex items-center gap-3 text-green-500">
						<CheckCircle class="h-6 w-6" />
						<p class="font-medium">Export Complete!</p>
					</div>
					<p class="text-muted-foreground text-sm">Saved to:</p>
					<p class="bg-muted mt-1 p-2 font-mono text-sm break-all">
						{exportResult.path}
					</p>
				</div>
			{:else if exportResult?.error}
				<div class="py-4">
					<div class="text-destructive mb-3 flex items-center gap-3">
						<AlertTriangle class="h-6 w-6" />
						<p class="font-medium">Export Failed</p>
					</div>
					<p class="text-muted-foreground text-sm">
						{exportResult.error}
					</p>
				</div>
			{/if}

			<div class="flex justify-end pt-2">
				<Button
					variant={exportResult?.success ? 'default' : 'outline'}
					onclick={closeExportModal}
					disabled={isExporting}
				>
					{exportResult?.success ? 'Done' : 'Close'}
				</Button>
			</div>
		</div>
	</div>
{/if}

<!-- Import Wizard -->
<ImportWizard open={showImportWizard} onClose={handleImportWizardClose} />
