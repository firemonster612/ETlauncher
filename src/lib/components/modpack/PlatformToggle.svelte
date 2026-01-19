<script lang="ts">
	import type { ModpackPlatform } from '$lib/types';

	interface Props {
		platform: ModpackPlatform;
		active?: boolean;
		omitted?: boolean;
		onclick?: () => void;
	}

	let { platform, active = false, omitted = false, onclick }: Props = $props();

	function getPlatformLabel(p: ModpackPlatform): string {
		switch (p) {
			case 'modrinth':
				return 'M';
			case 'curseforge':
				return 'CF';
			case 'ftb':
				return 'FTB';
			case 'technic':
				return 'T';
			case 'atlauncher':
				return 'AT';
			default:
				return p.charAt(0).toUpperCase();
		}
	}

	function getPlatformFullName(p: ModpackPlatform): string {
		switch (p) {
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
				return p;
		}
	}

	function getPlatformColor(p: ModpackPlatform): string {
		switch (p) {
			case 'modrinth':
				return 'text-green-500';
			case 'curseforge':
				return 'text-orange-500';
			case 'ftb':
				return 'text-blue-500';
			case 'technic':
				return 'text-yellow-500';
			case 'atlauncher':
				return 'text-purple-500';
			default:
				return 'text-muted-foreground';
		}
	}
</script>

<button
	type="button"
	class="platform-toggle flex h-9 w-9 items-center justify-center text-xs font-bold {getPlatformColor(
		platform
	)}"
	data-active={active}
	data-omitted={omitted}
	title={getPlatformFullName(platform)}
	{onclick}
>
	{getPlatformLabel(platform)}
</button>
