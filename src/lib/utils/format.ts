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
 * Uses binary units (1024) for accurate file size representation
 */
export function formatBytes(bytes: number): string {
	if (bytes < 1024) return `${bytes} B`;
	if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
	if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
	return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

/**
 * Format download speed to human-readable string (e.g., 1.5 MB/s)
 */
export function formatSpeed(bytesPerSec: number): string {
	if (bytesPerSec < 1024) return `${bytesPerSec.toFixed(0)} B/s`;
	if (bytesPerSec < 1024 * 1024) return `${(bytesPerSec / 1024).toFixed(1)} KB/s`;
	return `${(bytesPerSec / (1024 * 1024)).toFixed(1)} MB/s`;
}

/**
 * Format seconds to human-readable duration (e.g., 5m 30s)
 */
export function formatEta(seconds: number): string {
	if (!isFinite(seconds) || seconds <= 0) return '';
	if (seconds < 60) return `${Math.ceil(seconds)}s`;
	if (seconds < 3600) {
		const mins = Math.floor(seconds / 60);
		const secs = Math.ceil(seconds % 60);
		return secs > 0 ? `${mins}m ${secs}s` : `${mins}m`;
	}
	const hours = Math.floor(seconds / 3600);
	const mins = Math.floor((seconds % 3600) / 60);
	return mins > 0 ? `${hours}h ${mins}m` : `${hours}h`;
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
