import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { LoaderVersion, LoaderInstallProgress } from '$lib/types/loader';
import type { LoaderType } from '$lib/types/instance';

/**
 * Get available loader versions for a specific loader type and Minecraft version
 */
export async function getLoaderVersions(
	loaderType: LoaderType,
	minecraftVersion: string
): Promise<LoaderVersion[]> {
	return invoke('get_loader_versions', {
		loaderType,
		minecraftVersion,
	});
}

/**
 * Install a mod loader to an instance
 * @param instanceId - The instance ID
 * @param loaderType - Type of loader to install
 * @param loaderVersion - Loader version to install
 * @param onProgress - Optional callback for progress updates
 */
export async function installLoader(
	instanceId: string,
	loaderType: LoaderType,
	loaderVersion: string,
	onProgress?: (progress: LoaderInstallProgress) => void
): Promise<void> {
	if (onProgress) {
		// Set up event listener for progress updates
		const unlisten = await listen<LoaderInstallProgress>('loader-install-progress', (event) => {
			onProgress(event.payload);
		});

		// Set up listener for completion
		const unlistenComplete = await listen('loader-install-complete', () => {
			unlisten();
			unlistenComplete();
		});
	}

	return invoke('install_loader', {
		instanceId,
		loaderType,
		loaderVersion,
	});
}

/**
 * Check if a loader is installed for an instance
 */
export async function checkLoaderInstalled(
	instanceId: string,
	loaderType: LoaderType,
	loaderVersion: string
): Promise<boolean> {
	return invoke('check_loader_installed', {
		instanceId,
		loaderType,
		loaderVersion,
	});
}

/**
 * Get the latest stable loader version for a given loader type
 */
export async function getLatestStableLoaderVersion(
	loaderType: LoaderType,
	minecraftVersion: string
): Promise<LoaderVersion | null> {
	const versions = await getLoaderVersions(loaderType, minecraftVersion);
	const stable = versions.find((v) => v.stable);
	return stable || null;
}

/**
 * Get recommended loader version for a Minecraft version
 * (This is a placeholder - logic can be expanded later)
 */
export async function getRecommendedLoaderVersion(
	loaderType: LoaderType,
	minecraftVersion: string
): Promise<LoaderVersion | null> {
	// For now, just return latest stable
	// In the future, this could consider:
	// - Modpack compatibility
	// - Known issues with specific versions
	// - User preferences
	return getLatestStableLoaderVersion(loaderType, minecraftVersion);
}
