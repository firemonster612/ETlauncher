<script lang="ts">
	import { Calendar, Download, Package, Search, X } from '@lucide/svelte';
	import type { LoaderType, ModpackVersion } from '$lib/types';
	import { Button } from '$lib/ui/button';
	import { Input } from '$lib/ui/input';
	import * as Select from '$lib/ui/select';

	interface Props {
		open: boolean;
		versions: ModpackVersion[];
		selectedVersionId: string | null;
		onSelect: (versionId: string) => void;
		onClose: () => void;
		onInstall: (versionId: string) => void;
	}

	let { open, versions, selectedVersionId, onSelect, onClose, onInstall }: Props = $props();

	// Filter state
	let searchQuery = $state('');
	let filterMcVersion = $state<string | null>(null);
	let filterLoader = $state<LoaderType | null>(null);

	// Available filter options
	let availableMcVersions = $derived(
		[...new Set(versions.map((v) => v.mcVersion))].sort().reverse()
	);
	let availableLoaders = $derived([
		...new Set(versions.map((v) => v.loaderType).filter((l) => l && l !== 'unknown')),
	]);

	// Filtered versions
	let filteredVersions = $derived(() => {
		let result = versions;

		if (searchQuery) {
			const query = searchQuery.toLowerCase();
			result = result.filter(
				(v) => v.name.toLowerCase().includes(query) || v.mcVersion.toLowerCase().includes(query)
			);
		}

		if (filterMcVersion) {
			result = result.filter((v) => v.mcVersion === filterMcVersion);
		}

		if (filterLoader) {
			result = result.filter((v) => v.loaderType === filterLoader);
		}

		return result;
	});

	function formatDate(timestamp?: number): string {
		if (!timestamp) return '';
		return new Date(timestamp * 1000).toLocaleDateString();
	}

	function formatBytes(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
	}

	function clearFilters() {
		searchQuery = '';
		filterMcVersion = null;
		filterLoader = null;
	}

	let hasFilters = $derived(searchQuery || filterMcVersion || filterLoader);
</script>

{#if open}
	<div
		class="fixed inset-x-0 top-[var(--titlebar-height)] z-[80] flex h-[calc(100vh-var(--titlebar-height))] items-center justify-center bg-black/50 p-4"
		onclick={(e) => { if (e.target === e.currentTarget) onClose(); }}
		onkeydown={(e) => e.key === 'Escape' && onClose()}
		role="dialog"
		aria-modal="true"
		aria-label="Select version"
		tabindex="-1"
	>
		<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
		<div
			class="bg-card border-border flex max-h-[80vh] w-full max-w-2xl flex-col overflow-hidden rounded-lg border-2 shadow-2xl"
		>
			<!-- Header -->
			<div class="border-border flex items-center justify-between border-b p-4">
				<h2 class="text-lg font-bold">Select Version</h2>
				<button class="text-muted-foreground hover:text-foreground" onclick={onClose}>
					<X class="h-5 w-5" />
				</button>
			</div>

			<!-- Filters -->
			<div class="border-border space-y-3 border-b p-4">
				<div class="flex items-center gap-3">
					<div class="relative flex-1">
						<Search
							class="text-muted-foreground absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2"
						/>
						<Input
							type="text"
							placeholder="Search versions..."
							value={searchQuery}
							oninput={(e) => (searchQuery = e.currentTarget.value)}
							class="pl-9"
						/>
					</div>
					{#if hasFilters}
						<Button variant="ghost" size="sm" onclick={clearFilters}>
							<X class="mr-1 h-4 w-4" />
							Clear
						</Button>
					{/if}
				</div>
				<div class="flex items-center gap-3">
					<Select.Root
						type="single"
						value={filterMcVersion || ''}
						onValueChange={(v) => (filterMcVersion = v || null)}
					>
						<Select.Trigger class="border-border bg-background w-40 border-2 text-sm">
							{filterMcVersion || 'Any MC Version'}
						</Select.Trigger>
						<Select.Content class="border-border bg-card max-h-[200px] border-2">
							<Select.Item value="" label="Any MC Version">Any MC Version</Select.Item>
							{#each availableMcVersions as mcVersion (mcVersion)}
								<Select.Item value={mcVersion} label={mcVersion}>{mcVersion}</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>

					{#if availableLoaders.length > 1}
						<Select.Root
							type="single"
							value={filterLoader || ''}
							onValueChange={(v) => (filterLoader = (v as LoaderType) || null)}
						>
							<Select.Trigger class="border-border bg-background w-36 border-2 text-sm">
								{filterLoader
									? filterLoader.charAt(0).toUpperCase() + filterLoader.slice(1)
									: 'Any Loader'}
							</Select.Trigger>
							<Select.Content class="border-border bg-card border-2">
								<Select.Item value="" label="Any Loader">Any Loader</Select.Item>
								{#each availableLoaders as loader (loader)}
									<Select.Item value={loader} label={loader}>
										{loader.charAt(0).toUpperCase() + loader.slice(1)}
									</Select.Item>
								{/each}
							</Select.Content>
						</Select.Root>
					{/if}
				</div>
			</div>

			<!-- Version List -->
			<div class="flex-1 overflow-y-auto">
				{#if filteredVersions().length === 0}
					<div class="text-muted-foreground p-8 text-center text-sm">
						No versions match your filters
					</div>
				{:else}
					{#each filteredVersions() as version (version.id)}
						{@const totalSize = version.files.reduce((sum, f) => sum + f.size, 0)}
						<button
							type="button"
							class="hover:bg-muted/50 border-border flex w-full items-center gap-4 border-b p-4 text-left transition-colors last:border-b-0 {selectedVersionId ===
							version.id
								? 'bg-primary/10'
								: ''}"
							onclick={() => onSelect(version.id)}
						>
							<div class="flex-1">
								<div class="flex items-center gap-2">
									<span class="font-medium">{version.name}</span>
									{#if selectedVersionId === version.id}
										<span class="bg-primary/20 text-primary rounded px-1.5 py-0.5 text-xs"
											>Selected</span
										>
									{/if}
								</div>
								<div
									class="text-muted-foreground mt-1 flex flex-wrap items-center gap-x-3 gap-y-1 text-xs"
								>
									<span class="flex items-center gap-1">
										<Package class="h-3 w-3" />
										MC {version.mcVersion}
									</span>
									<span class="capitalize">{version.loaderType}</span>
									{#if version.releasedAt}
										<span class="flex items-center gap-1">
											<Calendar class="h-3 w-3" />
											{formatDate(version.releasedAt)}
										</span>
									{/if}
									{#if totalSize > 0}
										<span>{formatBytes(totalSize)}</span>
									{/if}
								</div>
								{#if version.changelog}
									<p class="text-muted-foreground mt-2 line-clamp-2 text-xs">
										{version.changelog.slice(0, 150)}{version.changelog.length > 150 ? '...' : ''}
									</p>
								{/if}
							</div>
							<Button
								size="sm"
								variant={selectedVersionId === version.id ? 'default' : 'secondary'}
								onclick={(e: Event) => {
									e.stopPropagation();
									onInstall(version.id);
								}}
							>
								<Download class="mr-1 h-4 w-4" />
								Install
							</Button>
						</button>
					{/each}
				{/if}
			</div>

			<!-- Footer -->
			<div class="border-border bg-card flex justify-between border-t p-4">
				<span class="text-muted-foreground text-sm">
					{filteredVersions().length} of {versions.length} versions
				</span>
				<Button variant="outline" onclick={onClose}>Close</Button>
			</div>
		</div>
	</div>
{/if}
