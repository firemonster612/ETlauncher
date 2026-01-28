<script lang="ts">
	import { Clock, Box, Globe, Camera } from '@lucide/svelte';
	import type { HomepageStats } from '$lib/types';

	interface Props {
		stats: HomepageStats;
	}

	let { stats }: Props = $props();

	function formatPlayTime(seconds: number): string {
		if (seconds < 60) return '< 1 min';
		const hours = Math.floor(seconds / 3600);
		const minutes = Math.floor((seconds % 3600) / 60);
		if (hours === 0) return `${minutes}m`;
		if (hours < 24) return `${hours}h ${minutes}m`;
		const days = Math.floor(hours / 24);
		const remainingHours = hours % 24;
		return `${days}d ${remainingHours}h`;
	}
</script>

<div class="border-border bg-card/50 flex items-center justify-between border-2 px-6 py-3">
	<div class="flex items-center gap-2">
		<Clock class="text-primary h-4 w-4" />
		<span class="text-muted-foreground text-xs uppercase tracking-wider">Play Time</span>
		<span class="font-bold">{formatPlayTime(stats.totalPlayTime)}</span>
	</div>

	<div class="bg-border h-6 w-px"></div>

	<div class="flex items-center gap-2">
		<Box class="text-primary h-4 w-4" />
		<span class="text-muted-foreground text-xs uppercase tracking-wider">Instances</span>
		<span class="font-bold">{stats.instanceCount}</span>
	</div>

	<div class="bg-border h-6 w-px"></div>

	<div class="flex items-center gap-2">
		<Globe class="text-primary h-4 w-4" />
		<span class="text-muted-foreground text-xs uppercase tracking-wider">Worlds</span>
		<span class="font-bold">{stats.worldCount}</span>
	</div>

	<div class="bg-border h-6 w-px"></div>

	<div class="flex items-center gap-2">
		<Camera class="text-primary h-4 w-4" />
		<span class="text-muted-foreground text-xs uppercase tracking-wider">Screenshots</span>
		<span class="font-bold">{stats.screenshotCount}</span>
	</div>
</div>
