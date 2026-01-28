<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { Loader2, Camera } from '@lucide/svelte';
	import NewsCarousel from '$lib/components/homepage/NewsCarousel.svelte';
	import StatsBar from '$lib/components/homepage/StatsBar.svelte';
	import ContinuePlayingSection from '$lib/components/homepage/ContinuePlayingSection.svelte';
	import RecentScreenshotsSection from '$lib/components/homepage/RecentScreenshotsSection.svelte';
	import MostPlayedInstancesSection from '$lib/components/homepage/MostPlayedInstancesSection.svelte';
	import MostPlayedWorldsSection from '$lib/components/homepage/MostPlayedWorldsSection.svelte';
	import FavoriteServersSection from '$lib/components/homepage/FavoriteServersSection.svelte';
	import QuickActionsSection from '$lib/components/homepage/QuickActionsSection.svelte';
	import { homepageStore } from '$lib/stores/homepage.svelte';
	import { launchStore } from '$lib/stores/launch.svelte';
	import { accountsStore } from '$lib/stores/accounts.svelte';
	import { alertDialogStore } from '$lib/stores/alertDialog.svelte';
	import * as instanceDetailService from '$lib/services/instance-detail';
	import type { Instance, HomepageWorld } from '$lib/types';

	onMount(() => {
		homepageStore.loadAll();
	});

	async function handleLaunch(instanceId: string) {
		if (!accountsStore.activeAccount) {
			alertDialogStore.alert({
				title: 'Account Required',
				message:
					'Please log in with a Microsoft account first. Go to Accounts and set one as active.',
				type: 'warning',
			});
			return;
		}

		await launchStore.launch(instanceId, accountsStore.activeAccount.id);
	}

	async function handleKill(instanceId: string) {
		await launchStore.kill(instanceId);
	}

	function handleInstanceClick(instance: Instance) {
		goto(resolve(`/instances?id=${instance.id}`), { replaceState: true });
	}

	async function handleWorldLaunch(world: HomepageWorld) {
		if (!accountsStore.activeAccount) {
			alertDialogStore.alert({
				title: 'Account Required',
				message:
					'Please log in with a Microsoft account first. Go to Accounts and set one as active.',
				type: 'warning',
			});
			return;
		}

		if (!world.supportsQuickPlay) {
			alertDialogStore.alert({
				title: 'Quick Play Not Supported',
				message: `Quick play requires Minecraft 1.20 or later. This world is on ${world.minecraftVersion}.`,
				type: 'warning',
			});
			return;
		}

		try {
			await instanceDetailService.launchIntoWorld(
				world.instanceId,
				accountsStore.activeAccount.id,
				world.folderName
			);
		} catch (e) {
			console.error('Failed to launch into world:', e);
			alertDialogStore.alert({
				title: 'Launch Failed',
				message: e instanceof Error ? e.message : 'Failed to launch into world',
				type: 'error',
			});
		}
	}

	// Get launch statuses map
	const launchStatuses = $derived(
		new Map(
			[...launchStore.launchStates.entries()].map(([id, state]) => [id, state.status])
		)
	);

	// Get launch status for continue instance
	const continueInstanceStatus = $derived(
		homepageStore.continueInstance
			? launchStatuses.get(homepageStore.continueInstance.id)
			: undefined
	);
</script>

<div class="homepage-container space-y-6">
	<!-- News Carousel -->
	{#if homepageStore.isLoadingNews}
		<div class="border-border bg-muted/30 flex aspect-[21/9] items-center justify-center border-2">
			<Loader2 class="text-muted-foreground h-8 w-8 animate-spin" />
		</div>
	{:else if homepageStore.newsArticles.length > 0}
		<NewsCarousel articles={homepageStore.newsArticles} />
	{/if}

	<!-- Stats Bar -->
	{#if homepageStore.isLoadingData}
		<div class="bg-muted/30 h-12 animate-pulse rounded"></div>
	{:else}
		<StatsBar stats={homepageStore.stats} />
	{/if}

	<!-- Continue Playing + Most Played Instances Row -->
	<div class="grid gap-6 lg:grid-cols-[minmax(0,2fr)_minmax(0,3fr)]">
		<!-- Continue Playing -->
		<div>
			{#if homepageStore.isLoadingData}
				<div class="space-y-2">
					<div class="bg-muted/50 h-4 w-40 animate-pulse rounded"></div>
					<div class="bg-muted/30 h-28 animate-pulse rounded"></div>
				</div>
			{:else if homepageStore.continueInstance}
				<ContinuePlayingSection
					instance={homepageStore.continueInstance}
					launchStatus={continueInstanceStatus}
					onLaunch={handleLaunch}
					onKill={handleKill}
					onCardClick={handleInstanceClick}
				/>
			{:else}
				<div class="space-y-2">
					<div class="flex items-center gap-2">
						<div class="bg-primary h-4 w-4 rounded"></div>
						<h2 class="text-sm font-bold uppercase tracking-wider">Continue Playing</h2>
					</div>
					<div class="border-border bg-muted/30 flex h-28 items-center justify-center border-2 border-dashed text-center">
						<div>
							<p class="text-muted-foreground text-sm">No recent sessions</p>
							<p class="text-muted-foreground mt-1 text-xs">Play an instance to see it here!</p>
						</div>
					</div>
				</div>
			{/if}
		</div>

		<!-- Most Played Instances -->
		<div>
			{#if homepageStore.isLoadingData}
				<div class="space-y-2">
					<div class="bg-muted/50 h-4 w-32 animate-pulse rounded"></div>
					<div class="flex gap-3">
						{#each [0, 1, 2, 3] as i (i)}
							<div class="bg-muted/30 h-48 w-48 flex-shrink-0 animate-pulse rounded"></div>
						{/each}
					</div>
				</div>
			{:else}
				<MostPlayedInstancesSection
					instances={homepageStore.mostPlayedInstances}
					{launchStatuses}
					onLaunch={handleLaunch}
					onKill={handleKill}
					onCardClick={handleInstanceClick}
				/>
			{/if}
		</div>
	</div>

	<!-- Recent Screenshots (full width) -->
	<div>
		{#if homepageStore.isLoadingData}
			<div class="space-y-2">
				<div class="bg-muted/50 h-4 w-40 animate-pulse rounded"></div>
				<div class="bg-muted/30 h-44 animate-pulse rounded"></div>
			</div>
		{:else}
			<RecentScreenshotsSection screenshots={homepageStore.recentScreenshots} />
			{#if homepageStore.recentScreenshots.length === 0}
				<div class="space-y-2">
					<div class="flex items-center gap-2">
						<Camera class="text-primary h-4 w-4" />
						<h2 class="text-sm font-bold uppercase tracking-wider">Recent Screenshots</h2>
					</div>
					<div class="border-border bg-muted/30 border-2 border-dashed p-6 text-center">
						<p class="text-muted-foreground text-sm">No screenshots yet</p>
						<p class="text-muted-foreground mt-1 text-xs">Press F2 in-game to take screenshots!</p>
					</div>
				</div>
			{/if}
		{/if}
	</div>

	<!-- Recent Worlds + Favorite Servers Row -->
	<div class="grid gap-6 lg:grid-cols-[minmax(0,3fr)_minmax(0,2fr)]">
		<!-- Recent Worlds -->
		<div>
			{#if homepageStore.isLoadingData}
				<div class="space-y-2">
					<div class="bg-muted/50 h-4 w-32 animate-pulse rounded"></div>
					<div class="flex gap-3">
						{#each [0, 1, 2] as i (i)}
							<div class="bg-muted/30 h-40 w-44 flex-shrink-0 animate-pulse rounded"></div>
						{/each}
					</div>
				</div>
			{:else}
				<MostPlayedWorldsSection
					worlds={homepageStore.mostPlayedWorlds}
					onLaunch={handleWorldLaunch}
				/>
			{/if}
		</div>

		<!-- Favorite Servers -->
		<div>
			{#if homepageStore.isLoadingData}
				<div class="space-y-2">
					<div class="bg-muted/50 h-4 w-32 animate-pulse rounded"></div>
					<div class="bg-muted/30 h-40 animate-pulse rounded"></div>
				</div>
			{:else}
				<FavoriteServersSection servers={homepageStore.favoriteServers} />
			{/if}
		</div>
	</div>

	<!-- Quick Actions (full width) -->
	<QuickActionsSection />

	<!-- Error display -->
	{#if homepageStore.dataError || homepageStore.newsError}
		<div class="bg-destructive/10 border-destructive text-destructive border-2 p-4 text-sm">
			{#if homepageStore.dataError}
				<p>Failed to load homepage data: {homepageStore.dataError}</p>
			{/if}
			{#if homepageStore.newsError}
				<p>Failed to load news: {homepageStore.newsError}</p>
			{/if}
			<button class="mt-2 underline" onclick={() => homepageStore.clearErrors()}>Dismiss</button>
		</div>
	{/if}
</div>
