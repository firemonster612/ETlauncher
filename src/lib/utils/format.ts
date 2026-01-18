import type { LoaderType, ContentPlatform, ModpackPlatform } from '$lib/types';

/**
 * Format download count to human-readable string (e.g., 1.5M, 10K)
 */
export function formatDownloads(downloads: number): string {
	if (downloads >= 1_000_000) {
		return `${(downloads / 1_000_000).toFixed(1)}M`;
	}
	if (downloads >= 1_000) {
		return `${(downloads / 1_000).toFixed(1)}K`;
	}
	return downloads.toString();
}

/**
 * Format bytes to human-readable string (e.g., 1.5 MB, 10 KB)
 */
export function formatBytes(bytes: number): string {
	if (bytes >= 1_000_000_000) {
		return `${(bytes / 1_000_000_000).toFixed(1)} GB`;
	}
	if (bytes >= 1_000_000) {
		return `${(bytes / 1_000_000).toFixed(1)} MB`;
	}
	if (bytes >= 1_000) {
		return `${(bytes / 1_000).toFixed(1)} KB`;
	}
	return `${bytes} B`;
}

/**
 * Get CSS classes for platform badge styling
 */
export function getPlatformColor(platform: ContentPlatform | ModpackPlatform | string): string {
	switch (platform) {
		case 'modrinth':
			return 'bg-green-500/20 text-green-500 border-green-500/50';
		case 'curseforge':
			return 'bg-orange-500/20 text-orange-500 border-orange-500/50';
		case 'ftb':
			return 'bg-blue-500/20 text-blue-500 border-blue-500/50';
		case 'technic':
			return 'bg-yellow-500/20 text-yellow-500 border-yellow-500/50';
		case 'atlauncher':
			return 'bg-purple-500/20 text-purple-500 border-purple-500/50';
		default:
			return 'bg-muted text-muted-foreground border-muted';
	}
}

/**
 * Get CSS classes for loader badge styling
 */
export function getLoaderColor(loader: LoaderType | string): string {
	switch (loader) {
		case 'fabric':
			return 'bg-amber-500/20 text-amber-500';
		case 'forge':
			return 'bg-orange-500/20 text-orange-500';
		case 'neoforge':
			return 'bg-red-500/20 text-red-500';
		case 'quilt':
			return 'bg-purple-500/20 text-purple-500';
		default:
			return 'bg-muted/50 text-muted-foreground';
	}
}
