<script lang="ts">
	import type { LoaderType } from '$lib/types/instance';
	import type { LoaderVersion } from '$lib/types/loader';
	import { getLoaderVersions } from '$lib/services/loader';
	import { Select, SelectContent, SelectItem, SelectTrigger } from '$lib/ui/select';

	interface Props {
		loaderType?: LoaderType;
		loaderVersion?: string;
		minecraftVersion?: string;
		onLoaderTypeChange?: (value: LoaderType) => void;
		onLoaderVersionChange?: (value: string) => void;
		disabled?: boolean;
	}

	let {
		loaderType = undefined,
		loaderVersion = undefined,
		minecraftVersion,
		onLoaderTypeChange,
		onLoaderVersionChange,
		disabled = false,
	}: Props = $props();

	const loaders: { value: LoaderType; label: string }[] = [
		{ value: 'vanilla', label: 'Vanilla (No mods)' },
		{ value: 'forge', label: 'Forge' },
		{ value: 'neoforge', label: 'NeoForge' },
		{ value: 'fabric', label: 'Fabric' },
		{ value: 'quilt', label: 'Quilt' },
		{ value: 'liteloader', label: 'LiteLoader (Legacy)' },
	];

	let loaderVersions = $state<LoaderVersion[]>([]);
	let loadingVersions = $state(false);
	let showVersionSelector = $derived(() => {
		return loaderType && loaderType !== 'vanilla';
	});

	// Reactive values to track for refetching
	let currentLoader = $derived(loaderType);
	let currentMcVersion = $derived(minecraftVersion);

	// Refetch loader versions when loader type or Minecraft version changes
	$effect(() => {
		// Access derived values to establish dependencies
		const loader = currentLoader;
		const mcVersion = currentMcVersion;

		if (loader && loader !== 'vanilla' && mcVersion) {
			fetchLoaderVersions(loader, mcVersion);
		} else {
			loaderVersions = [];
		}
	});

	async function fetchLoaderVersions(loader: LoaderType, mcVersion: string) {
		loadingVersions = true;
		try {
			loaderVersions = await getLoaderVersions(loader, mcVersion);

			// Auto-select the latest stable version
			const latestStable = loaderVersions.find((v) => v.stable);
			if (latestStable) {
				onLoaderVersionChange?.(latestStable.version);
			} else if (loaderVersions.length > 0) {
				// If no stable version, select the first one
				onLoaderVersionChange?.(loaderVersions[0].version);
			} else {
				onLoaderVersionChange?.('');
			}
		} catch (error) {
			console.error('Failed to load loader versions:', error);
			loaderVersions = [];
			onLoaderVersionChange?.('');
		} finally {
			loadingVersions = false;
		}
	}

	function handleLoaderTypeChange(newValue: string) {
		const newLoaderType = newValue as LoaderType;
		onLoaderTypeChange?.(newLoaderType);

		// Clear current version - the $effect will fetch new versions
		onLoaderVersionChange?.('');

		if (newLoaderType === 'vanilla') {
			loaderVersions = [];
		}
		// The $effect will automatically fetch versions for the new loader type
	}

	function handleLoaderVersionChange(newValue: string) {
		onLoaderVersionChange?.(newValue);
	}

	const loaderId = crypto.randomUUID();
	const loaderVersionId = crypto.randomUUID();
</script>

<div class="space-y-3">
	<div>
		<label class="text-foreground text-sm font-medium" for={loaderId}> Mod Loader </label>
		<Select
			type="single"
			value={loaderType ? String(loaderType) : undefined}
			onValueChange={handleLoaderTypeChange}
			{disabled}
		>
			<SelectTrigger id={loaderId}>
				{#if loaderType}
					{loaders.find((l) => l.value === loaderType)?.label || 'Select loader...'}
				{:else}
					Select loader...
				{/if}
			</SelectTrigger>
			<SelectContent>
				{#each loaders as loader (loader.value)}
					<SelectItem value={String(loader.value)}>
						{loader.label}
					</SelectItem>
				{/each}
			</SelectContent>
		</Select>
	</div>

	{#if showVersionSelector()}
		<div>
			<label class="text-foreground text-sm font-medium" for={loaderVersionId}>
				Loader Version
			</label>
			<Select
				type="single"
				value={loaderVersion ? String(loaderVersion) : undefined}
				onValueChange={handleLoaderVersionChange}
				{disabled}
			>
				<SelectTrigger id={loaderVersionId}>
					{#if loadingVersions}
						Loading versions...
					{:else if loaderVersion}
						{loaderVersion}
					{:else}
						Select version...
					{/if}
				</SelectTrigger>
				<SelectContent>
					{#each loaderVersions as version (version.version)}
						<SelectItem value={String(version.version)}>
							{version.version}
							{#if version.stable}
								<span class="text-muted-foreground ml-2 text-xs">(Stable)</span>
							{/if}
						</SelectItem>
					{/each}
					{#if loaderVersions.length === 0 && !loadingVersions}
						<div class="text-muted-foreground p-2 text-center text-sm">
							No versions available for this loader
						</div>
					{/if}
				</SelectContent>
			</Select>
		</div>
	{/if}
</div>
