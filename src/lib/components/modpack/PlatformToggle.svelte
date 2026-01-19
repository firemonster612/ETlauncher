<script lang="ts">
	import type { ModpackPlatform } from '$lib/types';

	interface Props {
		platform: ModpackPlatform | 'all';
		active?: boolean;
		onclick?: () => void;
	}

	let { platform, active = false, onclick }: Props = $props();

	function getPlatformLabel(p: ModpackPlatform | 'all'): string {
		switch (p) {
			case 'all':
				return 'All';
			case 'modrinth':
				return 'Modrinth';
			case 'curseforge':
				return 'CurseForge';
			case 'ftb':
				return 'FTB';
			case 'technic':
				return 'Technic';
			case 'atlauncher':
				return 'ATLauncher';
			default:
				return (p as string).charAt(0).toUpperCase() + (p as string).slice(1);
		}
	}

	function getPlatformColor(p: ModpackPlatform | 'all'): string {
		if (!active) return 'border-border bg-card hover:border-muted-foreground';
		switch (p) {
			case 'all':
				return 'border-primary bg-primary/20 text-primary';
			case 'modrinth':
				return 'border-green-500 bg-green-500/20 text-green-400';
			case 'curseforge':
				return 'border-orange-500 bg-orange-500/20 text-orange-400';
			case 'ftb':
				return 'border-blue-500 bg-blue-500/20 text-blue-400';
			case 'technic':
				return 'border-yellow-500 bg-yellow-500/20 text-yellow-400';
			case 'atlauncher':
				return 'border-purple-500 bg-purple-500/20 text-purple-400';
			default:
				return 'border-primary bg-primary/20 text-primary';
		}
	}
</script>

<button
	type="button"
	class="platform-toggle-btn h-9 border-2 px-4 text-xs font-bold transition-all {getPlatformColor(
		platform
	)}"
	data-active={active}
	{onclick}
>
	{getPlatformLabel(platform)}
</button>

<style>
	.platform-toggle-btn {
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.platform-toggle-btn:hover {
		transform: translateY(-1px);
	}

	.platform-toggle-btn:active {
		transform: translateY(0);
	}
</style>
