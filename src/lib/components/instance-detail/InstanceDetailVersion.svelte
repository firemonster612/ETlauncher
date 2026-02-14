<script lang="ts">
	import {
		AlertTriangle,
		ArrowRight,
		Calendar,
		Check,
		CheckCircle2,
		HelpCircle,
		Loader2,
		Package,
		RefreshCw,
		Trash2,
		Zap,
	} from '@lucide/svelte';
	import { updateInstance } from '$lib/services/instance';
	import { getLoaderVersions, installLoader } from '$lib/services/loader';
	import * as updateService from '$lib/services/update';
	import { versionsStore } from '$lib/stores/versions.svelte';
	import type { Instance, LoaderType } from '$lib/types/instance';
	import type { LoaderInstallProgress, LoaderVersion } from '$lib/types/loader';
	import type {
		InstanceUpdateCheck,
		InstanceUpdatePlan,
		ModpackInstanceUpdateCheck,
		ModpackUpdatePlan,
		UserContentDecision,
	} from '$lib/types/update';
	import { Button } from '$lib/ui/button';
	import { Select, SelectContent, SelectItem, SelectTrigger } from '$lib/ui/select';

	interface Props {
		instance: Instance;
		onUpdated: (instance: Instance) => void;
	}

	let { instance, onUpdated }: Props = $props();

	// Determine mode based on instance type
	let isModpackInstance = $derived(!!instance.modpackPlatform);

	// State
	let isChecking = $state(false);
	let isUpdating = $state(false);
	let updateStarted = $state(false);
	let error = $state<string | null>(null);

	// Modpack update state
	let modpackCheck = $state<ModpackInstanceUpdateCheck | null>(null);
	let selectedVersionId = $state<string | null>(null);
	let userContentDecisions = $state<Record<string, UserContentDecision>>({});

	// Non-modpack update state
	let instanceCheck = $state<InstanceUpdateCheck | null>(null);
	let selectedMcVersion = $state<string | null>(null);
	let incompatibleDecisions = $state<Record<string, UserContentDecision>>({});
	let isPreviewingMigration = $state(false);

	// Loader version change state
	let availableLoaderVersions = $state<LoaderVersion[]>([]);
	let isLoadingLoaderVersions = $state(false);
	let selectedLoaderVersion = $state<string | null>(null);
	let isChangingLoader = $state(false);
	let loaderChangeProgress = $state<LoaderInstallProgress | null>(null);
	let loaderChangeError = $state<string | null>(null);

	// Load versions for non-modpack instances
	$effect(() => {
		if (!isModpackInstance && versionsStore.versions.length === 0) {
			versionsStore.load();
		}
	});

	// Initial load
	$effect(() => {
		handleCheckUpdates();
	});

	// Load available loader versions for non-vanilla instances (both modpack and custom)
	$effect(() => {
		if (instance.loaderType !== 'vanilla') {
			loadLoaderVersions();
		}
	});

	async function loadLoaderVersions() {
		isLoadingLoaderVersions = true;
		try {
			availableLoaderVersions = await getLoaderVersions(
				instance.loaderType,
				instance.minecraftVersion
			);
			selectedLoaderVersion = instance.loaderVersion ?? null;
		} catch (e) {
			console.error('Failed to load loader versions:', e);
			availableLoaderVersions = [];
		} finally {
			isLoadingLoaderVersions = false;
		}
	}

	async function handleCheckUpdates() {
		isChecking = true;
		error = null;

		try {
			if (isModpackInstance) {
				modpackCheck = await updateService.checkModpackInstanceUpdates(instance.id);
				// Initialize decisions as pending for all user-added content
				const decisions: Record<string, UserContentDecision> = {};
				for (const item of modpackCheck.userAddedContent) {
					decisions[item.filename] = 'pending';
				}
				userContentDecisions = decisions;
				// Default to current version (user selects what they want)
				selectedVersionId = modpackCheck.currentVersion.versionId;
			} else {
				instanceCheck = await updateService.checkInstanceUpdates(instance.id);
				// Initialize decisions as pending for all incompatible content
				const decisions: Record<string, UserContentDecision> = {};
				for (const item of instanceCheck.incompatibleContent) {
					decisions[item.filename] = 'pending';
				}
				incompatibleDecisions = decisions;
				// Default to current version
				selectedMcVersion = instance.minecraftVersion;
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to check for updates';
		} finally {
			isChecking = false;
		}
	}

	// For non-modpack instances: preview migration when MC version changes
	async function handleMcVersionChange(version: string) {
		if (version === selectedMcVersion) return;
		selectedMcVersion = version;

		// If same as current, reset to the initial check state
		if (version === instance.minecraftVersion && instanceCheck) {
			const decisions: Record<string, UserContentDecision> = {};
			for (const item of instanceCheck.incompatibleContent) {
				decisions[item.filename] = 'pending';
			}
			incompatibleDecisions = decisions;
			return;
		}

		// Preview migration to selected version
		isPreviewingMigration = true;
		error = null;

		try {
			const preview = await updateService.previewVersionMigration(
				instance.id,
				version,
				instance.loaderType
			);

			// Update instanceCheck with new compatibility data
			if (instanceCheck) {
				instanceCheck = {
					...instanceCheck,
					latestMcVersion: version,
					hasMcUpdate: version !== instance.minecraftVersion,
					targetLoaderVersion: preview.targetLoaderVersion,
					compatibleContent: preview.updatable.concat(preview.upToDate),
					incompatibleContent: preview.incompatible,
					unidentifiedContent: preview.unidentified,
				};

				// Reset decisions for new incompatible content
				const decisions: Record<string, UserContentDecision> = {};
				for (const item of instanceCheck.incompatibleContent) {
					decisions[item.filename] = 'pending';
				}
				incompatibleDecisions = decisions;
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to preview version migration';
		} finally {
			isPreviewingMigration = false;
		}
	}

	async function handleExecuteUpdate() {
		isUpdating = true;
		error = null;
		updateStarted = false;

		try {
			if (isModpackInstance && modpackCheck && selectedVersionId) {
				const plan: ModpackUpdatePlan = {
					instanceId: instance.id,
					targetVersionId: selectedVersionId,
					userContentDecisions,
				};
				await updateService.executeModpackUpdate(instance.id, plan);
			} else if (!isModpackInstance && instanceCheck && selectedMcVersion) {
				const plan: InstanceUpdatePlan = {
					instanceId: instance.id,
					targetMcVersion: selectedMcVersion,
					targetLoaderType: instanceCheck.currentLoaderType,
					targetLoaderVersion: instanceCheck.targetLoaderVersion,
					incompatibleDecisions,
				};
				await updateService.executeInstanceUpdate(instance.id, plan);
			}
			updateStarted = true;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to start update';
		} finally {
			isUpdating = false;
		}
	}

	async function handleChangeLoaderVersion() {
		if (!selectedLoaderVersion || selectedLoaderVersion === instance.loaderVersion) return;

		isChangingLoader = true;
		loaderChangeError = null;
		loaderChangeProgress = null;

		try {
			await installLoader(instance.id, instance.loaderType, selectedLoaderVersion, (progress) => {
				loaderChangeProgress = progress;
			});

			const updatedInstance = await updateInstance(instance.id, {
				loaderVersion: selectedLoaderVersion,
			});

			onUpdated(updatedInstance);
			await loadLoaderVersions();
		} catch (e) {
			loaderChangeError = e instanceof Error ? e.message : 'Failed to change loader version';
		} finally {
			isChangingLoader = false;
			loaderChangeProgress = null;
		}
	}

	// Get selected version details (modpack)
	let selectedVersion = $derived(
		modpackCheck?.availableVersions.find((v) => v.versionId === selectedVersionId)
	);

	// Latest version is always first in the array (backend sorts newest-first)
	let latestModpackVersion = $derived(modpackCheck?.availableVersions[0]);

	// Can update if latest version exists and is NOT the current version
	let canUpdateToLatest = $derived(latestModpackVersion && !latestModpackVersion.isCurrent);

	function selectLatestVersion() {
		if (latestModpackVersion) {
			selectedVersionId = latestModpackVersion.versionId;
		}
	}

	// Check if all decisions are made
	let allUserContentDecisionsMade = $derived(
		modpackCheck?.userAddedContent.length === 0 ||
			Object.values(userContentDecisions).every((d) => d !== 'pending')
	);

	let allIncompatibleDecisionsMade = $derived(
		instanceCheck?.incompatibleContent.length === 0 ||
			Object.values(incompatibleDecisions).every((d) => d !== 'pending')
	);

	// Check if can update
	let canUpdateModpack = $derived(
		modpackCheck &&
			selectedVersionId &&
			selectedVersionId !== modpackCheck.currentVersion.versionId &&
			allUserContentDecisionsMade
	);

	// Check if loader version is available (required for non-vanilla instances)
	let hasValidLoaderVersion = $derived(
		instanceCheck &&
			(instanceCheck.currentLoaderType === 'vanilla' || !!instanceCheck.targetLoaderVersion)
	);

	let canUpdateInstance = $derived(
		instanceCheck &&
			selectedMcVersion &&
			selectedMcVersion !== instance.minecraftVersion &&
			allIncompatibleDecisionsMade &&
			hasValidLoaderVersion
	);

	// Check if version has changed
	let versionChanged = $derived(
		isModpackInstance
			? selectedVersionId !== modpackCheck?.currentVersion.versionId
			: selectedMcVersion !== instance.minecraftVersion
	);

	// Format date
	function formatDate(timestamp: number | undefined): string {
		if (!timestamp) return 'Unknown';
		return new Date(timestamp * 1000).toLocaleDateString();
	}

	// Get loader display color
	function getLoaderColor(loader: LoaderType): string {
		switch (loader) {
			case 'fabric':
				return 'text-amber-500';
			case 'forge':
				return 'text-orange-500';
			case 'neoforge':
				return 'text-red-500';
			case 'quilt':
				return 'text-purple-500';
			default:
				return 'text-green-500';
		}
	}

	// Get filtered MC versions for dropdown (only release versions)
	let availableMcVersions = $derived(
		versionsStore.versions.filter((v) => v.type === 'release').map((v) => v.id)
	);

	// Check if on latest MC version (for non-modpack instances)
	let isOnLatestMc = $derived(versionsStore.latestRelease === instance.minecraftVersion);

	// Bulk action helpers for user content (modpack)
	function keepAllUserContent() {
		if (!modpackCheck) return;
		const decisions: Record<string, UserContentDecision> = {};
		for (const item of modpackCheck.userAddedContent) {
			decisions[item.filename] = 'keep';
		}
		userContentDecisions = decisions;
	}

	function removeAllUserContent() {
		if (!modpackCheck) return;
		const decisions: Record<string, UserContentDecision> = {};
		for (const item of modpackCheck.userAddedContent) {
			decisions[item.filename] = 'remove';
		}
		userContentDecisions = decisions;
	}

	// Bulk action helpers for incompatible content (non-modpack)
	function keepAllIncompatible() {
		if (!instanceCheck) return;
		const decisions: Record<string, UserContentDecision> = {};
		for (const item of instanceCheck.incompatibleContent) {
			decisions[item.filename] = 'keep';
		}
		incompatibleDecisions = decisions;
	}

	function removeAllIncompatible() {
		if (!instanceCheck) return;
		const decisions: Record<string, UserContentDecision> = {};
		for (const item of instanceCheck.incompatibleContent) {
			decisions[item.filename] = 'remove';
		}
		incompatibleDecisions = decisions;
	}
</script>

<div class="space-y-6">
	{#if isChecking}
		<div class="flex items-center justify-center py-12">
			<div class="space-y-3 text-center">
				<Loader2 class="text-primary mx-auto h-8 w-8 animate-spin" />
				<p class="text-muted-foreground text-sm">Loading version information...</p>
			</div>
		</div>
	<!-- MODPACK VERSION MANAGEMENT -->
	{:else if isModpackInstance && modpackCheck}
		<!-- Current Version Card -->
		<section class="border-border bg-muted/10 rounded-lg border p-4">
			<h3 class="mb-3 font-semibold">Current Version</h3>
			<div class="flex items-center gap-3">
				<div class="bg-primary/10 rounded-lg p-2.5">
					<Package class="text-primary h-5 w-5" />
				</div>
				<div>
					<p class="font-medium">{modpackCheck.currentVersion.versionName}</p>
					<p class="text-muted-foreground text-sm">
						MC {modpackCheck.currentVersion.mcVersion}
						<span class="capitalize {getLoaderColor(modpackCheck.currentVersion.loaderType)}">
							• {modpackCheck.currentVersion.loaderType}
						</span>
						{#if modpackCheck.currentVersion.loaderVersion}
							<span class="text-muted-foreground">
								{modpackCheck.currentVersion.loaderVersion}
							</span>
						{/if}
					</p>
				</div>
			</div>
		</section>

		<!-- Loader Version Selector (only for non-vanilla modpacks) -->
		{#if modpackCheck.currentVersion.loaderType !== 'vanilla'}
			<section class="border-border bg-muted/10 rounded-lg border p-4">
				<h3 class="mb-3 font-semibold">Change Loader Version</h3>

				{#if isLoadingLoaderVersions}
					<div class="text-muted-foreground flex items-center gap-2 text-sm">
						<Loader2 class="h-4 w-4 animate-spin" />
						Loading available versions...
					</div>
				{:else if isChangingLoader && loaderChangeProgress}
					<div class="space-y-3">
						<div class="text-muted-foreground flex items-center gap-2 text-sm">
							<Loader2 class="h-4 w-4 animate-spin" />
							<span>{loaderChangeProgress.stage}</span>
						</div>
						<div class="bg-muted h-2 w-full rounded-full">
							<div
								class="bg-primary h-2 rounded-full transition-all"
								style="width: {loaderChangeProgress.progress}%"
							></div>
						</div>
					</div>
				{:else}
					<div class="flex items-end gap-3">
						<div class="flex-1">
							<Select
								type="single"
								value={selectedLoaderVersion || ''}
								onValueChange={(v) => {
									selectedLoaderVersion = v;
								}}
								disabled={isChangingLoader || isUpdating}
							>
								<SelectTrigger class="w-64">
									{#if selectedLoaderVersion}
										{selectedLoaderVersion}
										{#if selectedLoaderVersion === instance.loaderVersion}
											<span class="text-muted-foreground ml-1">(Current)</span>
										{/if}
									{:else}
										Select version...
									{/if}
								</SelectTrigger>
								<SelectContent class="max-h-64 overflow-y-auto">
									{#each availableLoaderVersions as version (version.version)}
										<SelectItem value={version.version}>
											<div class="flex items-center gap-2">
												<span>{version.version}</span>
												{#if version.version === instance.loaderVersion}
													<span class="text-xs text-green-500">(Current)</span>
												{/if}
												{#if version.stable}
													<span class="text-primary text-xs">(Stable)</span>
												{/if}
											</div>
										</SelectItem>
									{/each}
								</SelectContent>
							</Select>
						</div>

						<Button
							size="sm"
							disabled={!selectedLoaderVersion ||
								selectedLoaderVersion === instance.loaderVersion ||
								isChangingLoader ||
								isUpdating}
							onclick={handleChangeLoaderVersion}
						>
							Apply Change
						</Button>
					</div>

					{#if loaderChangeError}
						<div
							class="bg-destructive/10 border-destructive text-destructive mt-3 rounded border p-3 text-sm"
						>
							{loaderChangeError}
						</div>
					{/if}

					{#if selectedLoaderVersion && selectedLoaderVersion !== instance.loaderVersion}
						<div class="mt-3 flex items-center gap-2 text-sm text-yellow-600 dark:text-yellow-500">
							<AlertTriangle class="h-4 w-4" />
							<span>Changing loader versions may affect mod compatibility.</span>
						</div>
					{/if}
				{/if}
			</section>
		{/if}

		<!-- Version Selector -->
		<section class="border-border bg-muted/10 rounded-lg border p-4">
			<div class="mb-3 flex items-center justify-between">
				<h3 class="font-semibold">Switch Version</h3>
				<Button
					variant={canUpdateToLatest ? 'default' : 'outline'}
					size="sm"
					onclick={selectLatestVersion}
					disabled={!canUpdateToLatest || isChangingLoader}
				>
					<Zap class="mr-1.5 h-3.5 w-3.5" />
					{canUpdateToLatest ? 'Update to Latest' : 'On Latest'}
				</Button>
			</div>
			<Select
				type="single"
				value={selectedVersionId || ''}
				onValueChange={(v) => {
					selectedVersionId = v;
				}}
				disabled={isChangingLoader}
			>
				<SelectTrigger class="w-64">
					{#if selectedVersion}
						{selectedVersion.versionName}
						{#if selectedVersion.isCurrent}
							<span class="text-muted-foreground ml-1">(Current)</span>
						{/if}
					{:else}
						Select a version...
					{/if}
				</SelectTrigger>
				<SelectContent class="max-h-64 overflow-y-auto">
					{#each modpackCheck.availableVersions as version (version.versionId)}
						<SelectItem value={version.versionId}>
							<div class="flex items-center gap-2">
								<span>{version.versionName}</span>
								<span class="text-muted-foreground text-xs">MC {version.mcVersion}</span>
								{#if version.isCurrent}
									<span class="text-xs text-green-500">(Current)</span>
								{/if}
							</div>
						</SelectItem>
					{/each}
				</SelectContent>
			</Select>

			<!-- Selected Version Details -->
			{#if selectedVersion && !selectedVersion.isCurrent}
				<div class="bg-background/60 mt-4 space-y-2 rounded border p-4 text-sm">
					<div class="flex items-center gap-2">
						<span class="text-muted-foreground w-20">Version:</span>
						<span class="font-medium">{modpackCheck.currentVersion.versionName}</span>
						<ArrowRight class="text-primary h-4 w-4" />
						<span class="text-primary font-medium">{selectedVersion.versionName}</span>
					</div>
					<div class="flex items-center gap-2">
						<span class="text-muted-foreground w-20">Minecraft:</span>
						<span class="font-medium">{modpackCheck.currentVersion.mcVersion}</span>
						{#if modpackCheck.currentVersion.mcVersion !== selectedVersion.mcVersion}
							<ArrowRight class="text-primary h-4 w-4" />
							<span class="text-primary font-medium">{selectedVersion.mcVersion}</span>
						{/if}
					</div>
					<div class="flex items-center gap-2">
						<span class="text-muted-foreground w-20">Loader:</span>
						<span class="font-medium capitalize">{selectedVersion.loaderType}</span>
						{#if selectedVersion.loaderVersion}
							<span class="text-muted-foreground">({selectedVersion.loaderVersion})</span>
						{/if}
					</div>
					{#if selectedVersion.releasedAt}
						<div class="flex items-center gap-2">
							<span class="text-muted-foreground w-20">Released:</span>
							<Calendar class="h-3 w-3" />
							<span>{formatDate(selectedVersion.releasedAt)}</span>
						</div>
					{/if}

					{#if selectedVersion.changelog}
						<details class="mt-2">
							<summary class="text-primary cursor-pointer text-sm font-medium">
								View Changelog
							</summary>
							<div
								class="text-muted-foreground bg-muted/40 mt-2 max-h-48 overflow-y-auto rounded p-3 text-sm whitespace-pre-wrap"
							>
								{selectedVersion.changelog}
							</div>
						</details>
					{/if}
				</div>
			{/if}
		</section>

		<!-- User-Added Content Decisions -->
		{#if modpackCheck.userAddedContent.length > 0 && versionChanged}
			<section class="border-border bg-muted/10 rounded-lg border p-4">
				<div class="mb-3 flex items-center justify-between">
					<div class="flex items-center gap-2">
						<AlertTriangle class="h-4 w-4 text-yellow-500" />
						<h3 class="font-semibold">User-Added Content</h3>
					</div>
					<div class="flex gap-2">
						<Button variant="outline" size="sm" onclick={keepAllUserContent}>
							<Check class="mr-1.5 h-3.5 w-3.5" />
							Keep All
						</Button>
						<Button variant="outline" size="sm" onclick={removeAllUserContent}>
							<Trash2 class="mr-1.5 h-3.5 w-3.5" />
							Remove All
						</Button>
					</div>
				</div>
				<p class="text-muted-foreground mb-4 text-sm">
					These mods were added by you and aren't part of the modpack. "Keep" will try to find a
					compatible version, or keep the old file if none exists.
				</p>
				<div class="max-h-64 space-y-2 overflow-y-auto">
					{#each modpackCheck.userAddedContent as item (item.filename)}
						<div
							class="bg-background/60 flex items-center justify-between gap-3 rounded border p-3"
						>
							<div class="min-w-0 flex-1">
								<p class="truncate text-sm font-medium">{item.name || item.filename}</p>
								<p class="text-muted-foreground truncate text-xs">{item.filename}</p>
							</div>
							<div class="flex gap-1.5">
								<Button
									variant={userContentDecisions[item.filename] === 'keep' ? 'default' : 'outline'}
									size="sm"
									class="h-8 px-3"
									onclick={() => {
										userContentDecisions[item.filename] = 'keep';
									}}
								>
									<Check class="mr-1.5 h-3.5 w-3.5" />
									Keep
								</Button>
								<Button
									variant={userContentDecisions[item.filename] === 'remove'
										? 'destructive'
										: 'outline'}
									size="sm"
									class="h-8 px-3"
									onclick={() => {
										userContentDecisions[item.filename] = 'remove';
									}}
								>
									<Trash2 class="mr-1.5 h-3.5 w-3.5" />
									Remove
								</Button>
							</div>
						</div>
					{/each}
				</div>
			</section>
		{/if}

		<!-- NON-MODPACK VERSION MANAGEMENT -->
	{:else if !isModpackInstance && instanceCheck}
		<!-- Current Version Card -->
		<section class="border-border bg-muted/10 rounded-lg border p-4">
			<h3 class="mb-3 font-semibold">Current Version</h3>
			<div class="flex items-center gap-3">
				<div class="rounded-lg bg-green-500/10 p-2.5">
					<RefreshCw class="h-5 w-5 text-green-500" />
				</div>
				<div>
					<p class="font-medium">Minecraft {instanceCheck.currentMcVersion}</p>
					<p class="text-muted-foreground text-sm">
						<span class="capitalize {getLoaderColor(instanceCheck.currentLoaderType)}">
							{instanceCheck.currentLoaderType}
						</span>
						{#if instanceCheck.currentLoaderVersion}
							<span class="text-muted-foreground">
								{instanceCheck.currentLoaderVersion}
							</span>
						{/if}
					</p>
				</div>
			</div>
		</section>

		<!-- Loader Version Selector (only for non-vanilla instances) -->
		{#if instanceCheck.currentLoaderType !== 'vanilla'}
			<section class="border-border bg-muted/10 rounded-lg border p-4">
				<h3 class="mb-3 font-semibold">Change Loader Version</h3>

				{#if isLoadingLoaderVersions}
					<div class="text-muted-foreground flex items-center gap-2 text-sm">
						<Loader2 class="h-4 w-4 animate-spin" />
						Loading available versions...
					</div>
				{:else if isChangingLoader && loaderChangeProgress}
					<div class="space-y-3">
						<div class="text-muted-foreground flex items-center gap-2 text-sm">
							<Loader2 class="h-4 w-4 animate-spin" />
							<span>{loaderChangeProgress.stage}</span>
						</div>
						<div class="bg-muted h-2 w-full rounded-full">
							<div
								class="bg-primary h-2 rounded-full transition-all"
								style="width: {loaderChangeProgress.progress}%"
							></div>
						</div>
					</div>
				{:else}
					<div class="flex items-end gap-3">
						<div class="flex-1">
							<Select
								type="single"
								value={selectedLoaderVersion || ''}
								onValueChange={(v) => {
									selectedLoaderVersion = v;
								}}
								disabled={isChangingLoader || isUpdating}
							>
								<SelectTrigger class="w-64">
									{#if selectedLoaderVersion}
										{selectedLoaderVersion}
										{#if selectedLoaderVersion === instance.loaderVersion}
											<span class="text-muted-foreground ml-1">(Current)</span>
										{/if}
									{:else}
										Select version...
									{/if}
								</SelectTrigger>
								<SelectContent class="max-h-64 overflow-y-auto">
									{#each availableLoaderVersions as version (version.version)}
										<SelectItem value={version.version}>
											<div class="flex items-center gap-2">
												<span>{version.version}</span>
												{#if version.version === instance.loaderVersion}
													<span class="text-xs text-green-500">(Current)</span>
												{/if}
												{#if version.stable}
													<span class="text-primary text-xs">(Stable)</span>
												{/if}
											</div>
										</SelectItem>
									{/each}
								</SelectContent>
							</Select>
						</div>

						<Button
							size="sm"
							disabled={!selectedLoaderVersion ||
								selectedLoaderVersion === instance.loaderVersion ||
								isChangingLoader ||
								isUpdating}
							onclick={handleChangeLoaderVersion}
						>
							Apply Change
						</Button>
					</div>

					{#if loaderChangeError}
						<div
							class="bg-destructive/10 border-destructive text-destructive mt-3 rounded border p-3 text-sm"
						>
							{loaderChangeError}
						</div>
					{/if}

					{#if selectedLoaderVersion && selectedLoaderVersion !== instance.loaderVersion}
						<div class="mt-3 flex items-center gap-2 text-sm text-yellow-600 dark:text-yellow-500">
							<AlertTriangle class="h-4 w-4" />
							<span>Changing loader versions may affect mod compatibility.</span>
						</div>
					{/if}
				{/if}
			</section>
		{/if}

		<!-- Minecraft Version Selector -->
		<section class="border-border bg-muted/10 rounded-lg border p-4">
			<div class="mb-3 flex items-center justify-between">
				<h3 class="font-semibold">Change Minecraft Version</h3>
				<Button
					variant={isOnLatestMc ? 'outline' : 'default'}
					size="sm"
					onclick={() => handleMcVersionChange(versionsStore.latestRelease!)}
					disabled={isOnLatestMc || !versionsStore.latestRelease || isChangingLoader}
				>
					<Zap class="mr-1.5 h-3.5 w-3.5" />
					{isOnLatestMc ? 'On Latest' : 'Update to Latest'}
				</Button>
			</div>
			{#if versionsStore.isLoading}
				<div class="text-muted-foreground flex items-center gap-2 text-sm">
					<Loader2 class="h-4 w-4 animate-spin" />
					Loading versions...
				</div>
			{:else}
				<Select
					type="single"
					value={selectedMcVersion || ''}
					onValueChange={handleMcVersionChange}
					disabled={isChangingLoader}
				>
					<SelectTrigger class="w-64">
						{#if selectedMcVersion}
							Minecraft {selectedMcVersion}
							{#if selectedMcVersion === instance.minecraftVersion}
								<span class="text-muted-foreground ml-1">(Current)</span>
							{/if}
						{:else}
							Select a version...
						{/if}
					</SelectTrigger>
					<SelectContent class="max-h-64 overflow-y-auto">
						{#each availableMcVersions as version (version)}
							<SelectItem value={version}>
								<div class="flex items-center gap-2">
									<span>Minecraft {version}</span>
									{#if version === instance.minecraftVersion}
										<span class="text-xs text-green-500">(Current)</span>
									{/if}
									{#if version === versionsStore.latestRelease}
										<span class="text-primary text-xs">(Latest)</span>
									{/if}
								</div>
							</SelectItem>
						{/each}
					</SelectContent>
				</Select>

				{#if isPreviewingMigration}
					<div class="text-muted-foreground mt-3 flex items-center gap-2 text-sm">
						<Loader2 class="h-4 w-4 animate-spin" />
						Checking compatibility...
					</div>
				{/if}

				<!-- Target Version Details -->
				{#if selectedMcVersion && selectedMcVersion !== instance.minecraftVersion && !isPreviewingMigration}
					<div class="bg-background/60 mt-4 space-y-2 rounded border p-4 text-sm">
						<div class="flex items-center gap-2">
							<span class="text-muted-foreground w-20">Minecraft:</span>
							<span class="font-medium">{instance.minecraftVersion}</span>
							<ArrowRight class="text-primary h-4 w-4" />
							<span class="text-primary font-medium">{selectedMcVersion}</span>
						</div>
						<div class="flex items-center gap-2">
							<span class="text-muted-foreground w-20">Loader:</span>
							<span class="font-medium capitalize">{instanceCheck.currentLoaderType}</span>
							{#if instanceCheck.currentLoaderVersion}
								<span class="text-muted-foreground">({instanceCheck.currentLoaderVersion})</span>
							{/if}
							{#if instanceCheck.targetLoaderVersion && instanceCheck.targetLoaderVersion !== instanceCheck.currentLoaderVersion}
								<ArrowRight class="text-primary h-4 w-4" />
								<span class="text-primary font-medium">{instanceCheck.targetLoaderVersion}</span>
							{/if}
						</div>

						<!-- Warning if no loader version available -->
						{#if instanceCheck.currentLoaderType !== 'vanilla' && !instanceCheck.targetLoaderVersion}
							<div class="mt-2 flex items-center gap-2 text-yellow-500">
								<AlertTriangle class="h-4 w-4" />
								<span class="text-sm">
									No compatible {instanceCheck.currentLoaderType} version found for MC {selectedMcVersion}
								</span>
							</div>
						{/if}
					</div>
				{/if}
			{/if}
		</section>

		<!-- Content Compatibility (only show when version changed) -->
		{#if selectedMcVersion && selectedMcVersion !== instance.minecraftVersion && !isPreviewingMigration}
			<!-- Compatible Content -->
			{#if instanceCheck.compatibleContent.length > 0}
				<section class="border-border bg-muted/10 rounded-lg border p-4">
					<details class="group" open>
						<summary class="flex cursor-pointer items-center gap-2 font-semibold">
							<Check class="h-4 w-4 text-green-500" />
							Compatible ({instanceCheck.compatibleContent.length} mods)
						</summary>
						<p class="text-muted-foreground mt-2 text-sm">
							These mods will be updated to versions compatible with {selectedMcVersion}.
						</p>
						<div class="mt-3 max-h-40 space-y-1 overflow-y-auto">
							{#each instanceCheck.compatibleContent as item (item.filename)}
								<div class="bg-background/60 rounded p-2 text-sm">
									<span class="font-medium">{item.name}</span>
									{#if item.status.type === 'updateAvailable'}
										<span class="text-muted-foreground">
											→ v{item.status.availableVersion}
										</span>
									{/if}
								</div>
							{/each}
						</div>
					</details>
				</section>
			{/if}

			<!-- Incompatible Content Decisions -->
			{#if instanceCheck.incompatibleContent.length > 0}
				<section class="border-border bg-muted/10 rounded-lg border p-4">
					<div class="mb-3 flex items-center justify-between">
						<div class="flex items-center gap-2">
							<AlertTriangle class="h-4 w-4 text-yellow-500" />
							<h3 class="font-semibold">
								Incompatible ({instanceCheck.incompatibleContent.length} mods)
							</h3>
						</div>
						<div class="flex gap-2">
							<Button variant="outline" size="sm" onclick={keepAllIncompatible}>
								<Check class="mr-1.5 h-3.5 w-3.5" />
								Keep All
							</Button>
							<Button variant="outline" size="sm" onclick={removeAllIncompatible}>
								<Trash2 class="mr-1.5 h-3.5 w-3.5" />
								Remove All
							</Button>
						</div>
					</div>
					<p class="text-muted-foreground mb-4 text-sm">
						These mods don't have known compatible versions for {selectedMcVersion}. "Keep" will
						attempt to find an update anyway, or keep the old file if none exists.
					</p>
					<div class="max-h-64 space-y-2 overflow-y-auto">
						{#each instanceCheck.incompatibleContent as item (item.filename)}
							<div
								class="bg-background/60 flex items-center justify-between gap-3 rounded border p-3"
							>
								<div class="min-w-0 flex-1">
									<p class="truncate text-sm font-medium">{item.name}</p>
									<p class="text-muted-foreground truncate text-xs">{item.filename}</p>
								</div>
								<div class="flex gap-1.5">
									<Button
										variant={incompatibleDecisions[item.filename] === 'keep'
											? 'default'
											: 'outline'}
										size="sm"
										class="h-8 px-3"
										onclick={() => {
											incompatibleDecisions[item.filename] = 'keep';
										}}
									>
										<Check class="mr-1.5 h-3.5 w-3.5" />
										Keep
									</Button>
									<Button
										variant={incompatibleDecisions[item.filename] === 'remove'
											? 'destructive'
											: 'outline'}
										size="sm"
										class="h-8 px-3"
										onclick={() => {
											incompatibleDecisions[item.filename] = 'remove';
										}}
									>
										<Trash2 class="mr-1.5 h-3.5 w-3.5" />
										Remove
									</Button>
								</div>
							</div>
						{/each}
					</div>
				</section>
			{/if}

			<!-- Unidentified Content -->
			{#if instanceCheck.unidentifiedContent.length > 0}
				<section class="border-border bg-muted/10 rounded-lg border p-4">
					<details class="group">
						<summary class="flex cursor-pointer items-center gap-2 font-semibold">
							<HelpCircle class="text-muted-foreground h-4 w-4" />
							Unidentified ({instanceCheck.unidentifiedContent.length} mods)
						</summary>
						<p class="text-muted-foreground mt-2 text-sm">
							These mods couldn't be identified and will be kept as-is.
						</p>
						<div class="mt-3 max-h-32 space-y-1 overflow-y-auto">
							{#each instanceCheck.unidentifiedContent as item (item.filename)}
								<div class="bg-background/60 text-muted-foreground rounded p-2 text-sm">
									{item.filename}
								</div>
							{/each}
						</div>
					</details>
				</section>
			{/if}
		{/if}
	{/if}

	<!-- Error Display -->
	{#if error}
		<div class="bg-destructive/10 border-destructive text-destructive rounded border p-4 text-sm">
			{error}
		</div>
	{/if}

	<!-- Update Started Banner -->
	{#if updateStarted}
		<div class="border-border bg-primary/5 rounded-lg border p-4">
			<div class="flex items-center gap-3">
				<div class="bg-primary/10 rounded-full p-2">
					<CheckCircle2 class="text-primary h-5 w-5" />
				</div>
				<div>
					<p class="font-medium">Version change started</p>
					<p class="text-muted-foreground text-sm">
						The update is running in the background. You can track progress in the task bar at the
						bottom of the screen.
					</p>
				</div>
			</div>
		</div>
	{/if}

	<!-- Apply Button -->
	{#if !isChecking && !isUpdating && !updateStarted}
		<div class="flex justify-end gap-2">
			{#if isModpackInstance}
				{#if canUpdateModpack}
					<Button onclick={handleExecuteUpdate}>Apply Version Change</Button>
				{:else if versionChanged && !allUserContentDecisionsMade}
					<Button disabled>Make all selections to continue</Button>
				{:else if !versionChanged}
					<Button disabled>Select a different version</Button>
				{/if}
			{:else if !isModpackInstance}
				{#if canUpdateInstance}
					<Button onclick={handleExecuteUpdate}>Apply Version Change</Button>
				{:else if selectedMcVersion !== instance.minecraftVersion && !hasValidLoaderVersion}
					<Button disabled>No compatible loader version</Button>
				{:else if selectedMcVersion !== instance.minecraftVersion && !allIncompatibleDecisionsMade}
					<Button disabled>Make all selections to continue</Button>
				{:else if selectedMcVersion === instance.minecraftVersion}
					<Button disabled>Select a different version</Button>
				{/if}
			{/if}
		</div>
	{/if}
</div>
