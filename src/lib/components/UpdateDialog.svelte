<script lang="ts">
	import {
		Loader2,
		Check,
		AlertTriangle,
		HelpCircle,
		ArrowRight,
		Package,
		Calendar,
		ChevronDown,
	} from '@lucide/svelte';
	import { Button } from '$lib/ui/button';
	import * as Sheet from '$lib/ui/sheet';
	import { Select, SelectContent, SelectItem, SelectTrigger } from '$lib/ui/select';
	import * as RadioGroup from '$lib/ui/radio-group';
	import * as updateService from '$lib/services/update';
	import type {
		ModpackInstanceUpdateCheck,
		ModpackUpdatePlan,
		InstanceUpdateCheck,
		InstanceUpdatePlan,
		UpdateProgress,
		UserContentDecision,
	} from '$lib/types/update';
	import type { Instance } from '$lib/types/instance';

	interface Props {
		instance: Instance;
		open: boolean;
		onClose: () => void;
		onUpdated?: (instance: Instance) => void;
	}

	let { instance, open: isOpen, onClose, onUpdated }: Props = $props();

	// Determine mode based on instance type
	let isModpackInstance = $derived(!!instance.modpackPlatform);

	// State
	let isChecking = $state(false);
	let isUpdating = $state(false);
	let error = $state<string | null>(null);
	let updateProgress = $state<UpdateProgress | null>(null);

	// Modpack update state
	let modpackCheck = $state<ModpackInstanceUpdateCheck | null>(null);
	let selectedVersionId = $state<string | null>(null);
	let userContentDecisions = $state<Record<string, UserContentDecision>>({});
	let showAdvancedVersionSelect = $state(false);

	// Non-modpack update state
	let instanceCheck = $state<InstanceUpdateCheck | null>(null);
	let incompatibleDecisions = $state<Record<string, UserContentDecision>>({});

	// Reset state when dialog opens
	$effect(() => {
		if (isOpen) {
			error = null;
			updateProgress = null;
			modpackCheck = null;
			instanceCheck = null;
			selectedVersionId = null;
			userContentDecisions = {};
			incompatibleDecisions = {};
			showAdvancedVersionSelect = false;
			// Auto-check for updates
			handleCheckUpdates();
		}
	});

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
				// Auto-select the LATEST version (first non-current version in the list)
				const latestVersion = modpackCheck.availableVersions.find((v) => !v.isCurrent);
				selectedVersionId = latestVersion?.versionId || modpackCheck.currentVersion.versionId;
			} else {
				instanceCheck = await updateService.checkInstanceUpdates(instance.id);
				// Initialize decisions as pending for all incompatible content
				const decisions: Record<string, UserContentDecision> = {};
				for (const item of instanceCheck.incompatibleContent) {
					decisions[item.filename] = 'pending';
				}
				incompatibleDecisions = decisions;
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to check for updates';
		} finally {
			isChecking = false;
		}
	}

	async function handleExecuteUpdate() {
		isUpdating = true;
		error = null;

		try {
			if (isModpackInstance && modpackCheck && selectedVersionId) {
				const plan: ModpackUpdatePlan = {
					instanceId: instance.id,
					targetVersionId: selectedVersionId,
					userContentDecisions,
				};

				const updatedInstance = await updateService.executeModpackUpdate(
					instance.id,
					plan,
					(progress) => {
						updateProgress = progress;
					}
				);

				onUpdated?.(updatedInstance);
				onClose();
			} else if (!isModpackInstance && instanceCheck) {
				const plan: InstanceUpdatePlan = {
					instanceId: instance.id,
					targetMcVersion: instanceCheck.latestMcVersion,
					targetLoaderType: instanceCheck.currentLoaderType,
					targetLoaderVersion: instanceCheck.targetLoaderVersion,
					incompatibleDecisions,
				};

				const updatedInstance = await updateService.executeInstanceUpdate(
					instance.id,
					plan,
					(progress) => {
						updateProgress = progress;
					}
				);

				onUpdated?.(updatedInstance);
				onClose();
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Update failed';
		} finally {
			isUpdating = false;
			updateProgress = null;
		}
	}

	// Get selected version details
	let selectedVersion = $derived(
		modpackCheck?.availableVersions.find((v) => v.versionId === selectedVersionId)
	);

	// Get the latest (non-current) version
	let latestVersion = $derived(modpackCheck?.availableVersions.find((v) => !v.isCurrent));

	// Check if all decisions are made (not pending)
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

	let canUpdateInstance = $derived(
		instanceCheck && instanceCheck.hasMcUpdate && allIncompatibleDecisionsMade
	);

	// Format date
	function formatDate(timestamp: number | undefined): string {
		if (!timestamp) return 'Unknown';
		return new Date(timestamp * 1000).toLocaleDateString();
	}
</script>

<Sheet.Root bind:open={isOpen} onOpenChange={(open) => !open && onClose()}>
	<Sheet.Content side="right" class="w-full overflow-y-auto sm:max-w-xl">
		<Sheet.Header class="border-border border-b px-6 pb-4">
			<Sheet.Title>Check for Updates</Sheet.Title>
			<Sheet.Description>
				{#if isModpackInstance}
					Check for new modpack versions for <span class="font-medium">{instance.name}</span>
				{:else}
					Check for Minecraft updates for <span class="font-medium">{instance.name}</span>
				{/if}
			</Sheet.Description>
		</Sheet.Header>

		<div class="space-y-6 px-6 py-6">
			{#if isChecking}
				<div class="flex items-center justify-center py-12">
					<div class="space-y-3 text-center">
						<Loader2 class="text-primary mx-auto h-8 w-8 animate-spin" />
						<p class="text-muted-foreground text-sm">Checking for updates...</p>
					</div>
				</div>
			{:else if isUpdating && updateProgress}
				<div class="space-y-4">
					<div class="space-y-2 text-center">
						<Loader2 class="text-primary mx-auto h-8 w-8 animate-spin" />
						<p class="font-medium">{updateProgress.stage}</p>
						{#if updateProgress.currentItem}
							<p class="text-muted-foreground text-sm">{updateProgress.currentItem}</p>
						{/if}
					</div>
					<div class="bg-muted h-2 w-full rounded-full">
						<div
							class="bg-primary h-2 rounded-full transition-all"
							style="width: {updateProgress.progress}%"
						></div>
					</div>
					{#if updateProgress.totalItems > 0}
						<p class="text-muted-foreground text-center text-xs">
							{updateProgress.completedItems} / {updateProgress.totalItems} items
						</p>
					{/if}
				</div>

				<!-- MODPACK INSTANCE UPDATE UI -->
			{:else if isModpackInstance && modpackCheck}
				{#if modpackCheck.hasUpdate && latestVersion}
					<!-- Update Available Banner -->
					<div class="bg-primary/10 border-primary/30 space-y-3 rounded border p-4">
						<div class="flex items-center gap-2">
							<Package class="text-primary h-5 w-5" />
							<span class="text-primary font-medium">Update Available!</span>
						</div>

						<!-- Version Change Info -->
						<div class="space-y-2 text-sm">
							<div class="flex items-center gap-2">
								<span class="text-muted-foreground w-20">Modpack:</span>
								<span class="font-medium">{modpackCheck.currentVersion.versionName}</span>
								<ArrowRight class="text-primary h-4 w-4" />
								<span class="text-primary font-medium">{latestVersion.versionName}</span>
							</div>
							<div class="flex items-center gap-2">
								<span class="text-muted-foreground w-20">Minecraft:</span>
								<span class="font-medium">{modpackCheck.currentVersion.mcVersion}</span>
								{#if modpackCheck.currentVersion.mcVersion !== latestVersion.mcVersion}
									<ArrowRight class="text-primary h-4 w-4" />
									<span class="text-primary font-medium">{latestVersion.mcVersion}</span>
								{/if}
							</div>
							<div class="flex items-center gap-2">
								<span class="text-muted-foreground w-20">Loader:</span>
								<span class="font-medium capitalize">{latestVersion.loaderType}</span>
								{#if latestVersion.loaderVersion}
									<span class="text-muted-foreground">({latestVersion.loaderVersion})</span>
								{/if}
							</div>
							{#if latestVersion.releasedAt}
								<div class="flex items-center gap-2">
									<span class="text-muted-foreground w-20">Released:</span>
									<Calendar class="h-3 w-3" />
									<span>{formatDate(latestVersion.releasedAt)}</span>
								</div>
							{/if}
						</div>

						{#if latestVersion.changelog}
							<details class="mt-2">
								<summary class="text-primary cursor-pointer text-sm font-medium"
									>View Changelog</summary
								>
								<div
									class="text-muted-foreground bg-background/50 mt-2 max-h-48 overflow-y-auto rounded p-3 text-sm whitespace-pre-wrap"
								>
									{latestVersion.changelog}
								</div>
							</details>
						{/if}
					</div>

					<!-- Advanced: Select Different Version -->
					<details class="group" bind:open={showAdvancedVersionSelect}>
						<summary
							class="text-muted-foreground hover:text-foreground flex cursor-pointer items-center gap-1 text-sm"
						>
							<ChevronDown class="h-4 w-4 transition-transform group-open:rotate-180" />
							Advanced: Select a different version
						</summary>
						<div class="mt-3 space-y-2">
							<Select
								type="single"
								value={selectedVersionId || ''}
								onValueChange={(v) => {
									selectedVersionId = v;
								}}
							>
								<SelectTrigger class="w-full">
									{#if selectedVersion}
										{selectedVersion.versionName} (MC {selectedVersion.mcVersion})
									{:else}
										Select version...
									{/if}
								</SelectTrigger>
								<SelectContent>
									{#each modpackCheck.availableVersions as version (version.versionId)}
										<SelectItem value={version.versionId} disabled={version.isCurrent}>
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

							{#if selectedVersion && selectedVersion !== latestVersion && !selectedVersion.isCurrent}
								<p class="text-muted-foreground text-xs">
									Selected: {selectedVersion.versionName} (MC {selectedVersion.mcVersion})
								</p>
							{/if}
						</div>
					</details>

					<!-- User-Added Content Decisions -->
					{#if modpackCheck.userAddedContent.length > 0}
						<div class="space-y-3">
							<div class="flex items-center gap-2">
								<AlertTriangle class="h-4 w-4 text-yellow-500" />
								<h4 class="text-sm font-medium">User-Added Content</h4>
							</div>
							<p class="text-muted-foreground text-xs">
								These mods were added by you and are not part of the modpack. Choose what to do with
								each:
							</p>
							<div class="max-h-64 space-y-3 overflow-y-auto">
								{#each modpackCheck.userAddedContent as item (item.filename)}
									<div class="bg-muted/30 space-y-2 rounded p-3">
										<div class="text-sm font-medium">{item.name || item.filename}</div>
										<RadioGroup.Root
											value={userContentDecisions[item.filename]}
											onValueChange={(v) => {
												userContentDecisions[item.filename] = v as UserContentDecision;
											}}
											class="flex flex-row gap-4"
										>
											<label class="flex cursor-pointer items-center gap-2 text-sm">
												<RadioGroup.Item value="keep" />
												Keep
											</label>
											<label class="flex cursor-pointer items-center gap-2 text-sm">
												<RadioGroup.Item value="remove" />
												Remove
											</label>
										</RadioGroup.Root>
										{#if userContentDecisions[item.filename] === 'pending'}
											<p class="text-xs text-yellow-500">Please select an option</p>
										{/if}
									</div>
								{/each}
							</div>
						</div>
					{/if}
				{:else}
					<!-- No Update Available -->
					<div class="py-6 text-center">
						<Check class="mx-auto mb-3 h-12 w-12 text-green-500" />
						<p class="font-medium">You're up to date!</p>
						<p class="text-muted-foreground mt-1 text-sm">
							{modpackCheck.modpackName} is on the latest version.
						</p>
						<div class="text-muted-foreground bg-muted/50 mt-4 rounded p-3 text-sm">
							<p>
								Current: <span class="text-foreground font-medium"
									>{modpackCheck.currentVersion.versionName}</span
								>
							</p>
							<p>
								Minecraft: <span class="text-foreground font-medium"
									>{modpackCheck.currentVersion.mcVersion}</span
								>
							</p>
						</div>
					</div>
				{/if}

				<!-- NON-MODPACK INSTANCE UPDATE UI -->
			{:else if !isModpackInstance && instanceCheck}
				{#if instanceCheck.hasMcUpdate}
					<!-- Update Available Banner -->
					<div class="bg-primary/10 border-primary/30 space-y-3 rounded border p-4">
						<div class="text-primary font-medium">Update Available!</div>

						<div class="space-y-2 text-sm">
							<div class="flex items-center gap-2">
								<span class="text-muted-foreground w-20">Minecraft:</span>
								<span class="font-medium">{instanceCheck.currentMcVersion}</span>
								<ArrowRight class="text-primary h-4 w-4" />
								<span class="text-primary font-medium">{instanceCheck.latestMcVersion}</span>
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
						</div>
					</div>

					<!-- Compatible Content -->
					{#if instanceCheck.compatibleContent.length > 0}
						<details class="group" open>
							<summary class="flex cursor-pointer items-center gap-2 text-sm font-medium">
								<Check class="h-4 w-4 text-green-500" />
								Compatible Content ({instanceCheck.compatibleContent.length})
							</summary>
							<p class="text-muted-foreground mt-1 text-xs">
								These will be updated to versions compatible with {instanceCheck.latestMcVersion}.
							</p>
							<div class="mt-2 max-h-32 space-y-1 overflow-y-auto">
								{#each instanceCheck.compatibleContent as item (item.filename)}
									<div class="bg-muted/30 rounded p-2 text-sm">
										{item.name}
									</div>
								{/each}
							</div>
						</details>
					{/if}

					<!-- Incompatible Content Decisions -->
					{#if instanceCheck.incompatibleContent.length > 0}
						<div class="space-y-3">
							<div class="flex items-center gap-2">
								<AlertTriangle class="h-4 w-4 text-yellow-500" />
								<h4 class="text-sm font-medium">
									Incompatible Content ({instanceCheck.incompatibleContent.length})
								</h4>
							</div>
							<p class="text-muted-foreground text-xs">
								These mods don't have compatible versions for {instanceCheck.latestMcVersion}.
								Choose what to do with each:
							</p>
							<div class="max-h-64 space-y-3 overflow-y-auto">
								{#each instanceCheck.incompatibleContent as item (item.filename)}
									<div class="bg-muted/30 space-y-2 rounded p-3">
										<div class="text-sm font-medium">{item.name}</div>
										<RadioGroup.Root
											value={incompatibleDecisions[item.filename]}
											onValueChange={(v) => {
												incompatibleDecisions[item.filename] = v as UserContentDecision;
											}}
											class="flex flex-row gap-4"
										>
											<label class="flex cursor-pointer items-center gap-2 text-sm">
												<RadioGroup.Item value="keep" />
												Keep (may cause issues)
											</label>
											<label class="flex cursor-pointer items-center gap-2 text-sm">
												<RadioGroup.Item value="remove" />
												Remove
											</label>
										</RadioGroup.Root>
										{#if incompatibleDecisions[item.filename] === 'pending'}
											<p class="text-xs text-yellow-500">Please select an option</p>
										{/if}
									</div>
								{/each}
							</div>
						</div>
					{/if}

					<!-- Unidentified Content -->
					{#if instanceCheck.unidentifiedContent.length > 0}
						<details class="group">
							<summary class="flex cursor-pointer items-center gap-2 text-sm font-medium">
								<HelpCircle class="text-muted-foreground h-4 w-4" />
								Unidentified Content ({instanceCheck.unidentifiedContent.length})
							</summary>
							<p class="text-muted-foreground mt-1 text-xs">
								These mods couldn't be identified and will be kept as-is.
							</p>
							<div class="mt-2 max-h-32 space-y-1 overflow-y-auto">
								{#each instanceCheck.unidentifiedContent as item (item.filename)}
									<div class="bg-muted/30 text-muted-foreground rounded p-2 text-sm">
										{item.filename}
									</div>
								{/each}
							</div>
						</details>
					{/if}
				{:else}
					<!-- No Update Available -->
					<div class="py-6 text-center">
						<Check class="mx-auto mb-3 h-12 w-12 text-green-500" />
						<p class="font-medium">You're up to date!</p>
						<p class="text-muted-foreground mt-1 text-sm">
							Already running the latest Minecraft version.
						</p>
						<div class="text-muted-foreground bg-muted/50 mt-4 rounded p-3 text-sm">
							<p>
								Minecraft: <span class="text-foreground font-medium"
									>{instanceCheck.currentMcVersion}</span
								>
							</p>
							<p class="capitalize">
								Loader: <span class="text-foreground font-medium"
									>{instanceCheck.currentLoaderType}</span
								>
								{#if instanceCheck.currentLoaderVersion}
									({instanceCheck.currentLoaderVersion})
								{/if}
							</p>
						</div>
					</div>
				{/if}
			{/if}

			{#if error}
				<div
					class="bg-destructive/10 border-destructive text-destructive rounded border p-4 text-sm"
				>
					{error}
				</div>
			{/if}
		</div>

		<Sheet.Footer class="border-border border-t px-6 pt-4">
			<Button variant="outline" onclick={onClose} disabled={isUpdating}>
				{#if (isModpackInstance && modpackCheck && !modpackCheck.hasUpdate) || (!isModpackInstance && instanceCheck && !instanceCheck.hasMcUpdate)}
					Close
				{:else}
					Cancel
				{/if}
			</Button>
			{#if isModpackInstance}
				{#if canUpdateModpack}
					<Button onclick={handleExecuteUpdate} disabled={isUpdating}>
						{#if isUpdating}
							<Loader2 class="mr-2 h-4 w-4 animate-spin" />
							Updating...
						{:else}
							Update to {selectedVersion?.versionName}
						{/if}
					</Button>
				{:else if modpackCheck?.hasUpdate && !allUserContentDecisionsMade}
					<Button disabled>Make all selections to continue</Button>
				{/if}
			{:else if canUpdateInstance}
				<Button onclick={handleExecuteUpdate} disabled={isUpdating}>
					{#if isUpdating}
						<Loader2 class="mr-2 h-4 w-4 animate-spin" />
						Updating...
					{:else}
						Update to {instanceCheck?.latestMcVersion}
					{/if}
				</Button>
			{:else if instanceCheck?.hasMcUpdate && !allIncompatibleDecisionsMade}
				<Button disabled>Make all selections to continue</Button>
			{/if}
		</Sheet.Footer>
	</Sheet.Content>
</Sheet.Root>
