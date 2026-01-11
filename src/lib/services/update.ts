import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
	UpdateCheckResult,
	ModpackUpdateInfo,
	UpdatePlan,
	UpdateProgress,
	ModpackInstanceUpdateCheck,
	ModpackUpdatePlan,
	InstanceUpdateCheck,
	InstanceUpdatePlan,
} from '$lib/types/update';
import type { Instance, LoaderType } from '$lib/types/instance';
import type { InstalledContentManifest } from '$lib/types/content';

/**
 * Check for modpack updates for a modpack instance
 * @param instanceId - The instance ID
 * @returns ModpackUpdateInfo if an update is available, null otherwise
 */
export async function checkModpackUpdate(instanceId: string): Promise<ModpackUpdateInfo | null> {
	return invoke('check_modpack_update', { instanceId });
}

/**
 * Check for content updates (same MC version, newer mod versions)
 * @param instanceId - The instance ID
 * @returns UpdateCheckResult with lists of updatable, up-to-date, incompatible content
 */
export async function checkContentUpdates(instanceId: string): Promise<UpdateCheckResult> {
	return invoke('check_content_updates', { instanceId });
}

/**
 * Preview migration to a different Minecraft version
 * Shows content compatibility for the target version
 * @param instanceId - The instance ID
 * @param targetMcVersion - Target Minecraft version
 * @param targetLoader - Target mod loader type
 * @returns UpdateCheckResult with compatibility information
 */
export async function previewVersionMigration(
	instanceId: string,
	targetMcVersion: string,
	targetLoader: LoaderType
): Promise<UpdateCheckResult> {
	return invoke('preview_version_migration', {
		instanceId,
		targetMcVersion,
		targetLoader,
	});
}

/**
 * Execute content updates for an instance
 * @param instanceId - The instance ID
 * @param plan - The update plan
 * @param onProgress - Optional callback for progress updates
 */
export async function updateInstanceContent(
	instanceId: string,
	plan: UpdatePlan,
	onProgress?: (progress: UpdateProgress) => void
): Promise<void> {
	let unlisten: UnlistenFn | undefined;

	if (onProgress) {
		unlisten = await listen<UpdateProgress>('update_progress', (event) => {
			onProgress(event.payload);
		});
	}

	try {
		await invoke('update_instance_content', { instanceId, plan });
	} finally {
		if (unlisten) {
			unlisten();
		}
	}
}

/**
 * Migrate an instance to a different Minecraft version
 * @param instanceId - The instance ID
 * @param targetMcVersion - Target Minecraft version
 * @param targetLoader - Target mod loader type
 * @param targetLoaderVersion - Target mod loader version
 * @param plan - The update plan
 * @param onProgress - Optional callback for progress updates
 * @returns Updated instance
 */
export async function migrateInstanceVersion(
	instanceId: string,
	targetMcVersion: string,
	targetLoader: LoaderType,
	targetLoaderVersion: string,
	plan: UpdatePlan,
	onProgress?: (progress: UpdateProgress) => void
): Promise<Instance> {
	let unlisten: UnlistenFn | undefined;

	if (onProgress) {
		unlisten = await listen<UpdateProgress>('update_progress', (event) => {
			onProgress(event.payload);
		});
	}

	try {
		return await invoke('migrate_instance_version', {
			instanceId,
			targetMcVersion,
			targetLoader,
			targetLoaderVersion,
			plan,
		});
	} finally {
		if (unlisten) {
			unlisten();
		}
	}
}

/**
 * Get the content manifest for an instance
 * @param instanceId - The instance ID
 * @returns The installed content manifest
 */
export async function getContentManifest(instanceId: string): Promise<InstalledContentManifest> {
	return invoke('get_content_manifest', { instanceId });
}

/**
 * Create an update plan from UpdateCheckResult
 * Utility function to help build UpdatePlan from user selections
 */
export function createUpdatePlan(
	instanceId: string,
	checkResult: UpdateCheckResult,
	options: {
		updateMinecraftVersion?: string;
		updateLoaderVersion?: string;
		selectedUpdates?: string[]; // filenames to update, defaults to all updatable
		removeIncompatible?: string[]; // filenames to remove
		keepIncompatible?: string[]; // filenames to keep despite being incompatible
	} = {}
): UpdatePlan {
	const {
		updateMinecraftVersion,
		updateLoaderVersion,
		selectedUpdates,
		removeIncompatible = [],
		keepIncompatible = [],
	} = options;

	// Default to updating all updatable content if not specified
	const contentToUpdate = selectedUpdates ?? checkResult.updatable.map((c) => c.filename);

	return {
		instanceId,
		updateMinecraftVersion,
		updateLoaderVersion,
		contentToUpdate,
		contentToRemove: removeIncompatible,
		contentToKeep: keepIncompatible,
	};
}

// =============================================================================
// NEW UPDATE SYSTEM FUNCTIONS
// =============================================================================

/**
 * Check for modpack instance updates (returns all available versions)
 * Use this for modpack instances to see available modpack versions
 * @param instanceId - The instance ID
 * @returns ModpackInstanceUpdateCheck with all available versions
 */
export async function checkModpackInstanceUpdates(
	instanceId: string
): Promise<ModpackInstanceUpdateCheck> {
	return invoke('check_modpack_instance_updates', { instanceId });
}

/**
 * Execute a modpack update
 * @param instanceId - The instance ID
 * @param plan - The modpack update plan
 * @param onProgress - Optional callback for progress updates
 * @returns Updated instance
 */
export async function executeModpackUpdate(
	instanceId: string,
	plan: ModpackUpdatePlan,
	onProgress?: (progress: UpdateProgress) => void
): Promise<Instance> {
	let unlisten: UnlistenFn | undefined;

	if (onProgress) {
		unlisten = await listen<UpdateProgress>('update_progress', (event) => {
			onProgress(event.payload);
		});
	}

	try {
		return await invoke('execute_modpack_update', { instanceId, plan });
	} finally {
		if (unlisten) {
			unlisten();
		}
	}
}

/**
 * Check for non-modpack instance updates (targets latest MC version)
 * Use this for non-modpack instances to check for Minecraft version updates
 * @param instanceId - The instance ID
 * @returns InstanceUpdateCheck with compatibility information
 */
export async function checkInstanceUpdates(instanceId: string): Promise<InstanceUpdateCheck> {
	return invoke('check_instance_updates', { instanceId });
}

/**
 * Execute a non-modpack instance update
 * @param instanceId - The instance ID
 * @param plan - The instance update plan
 * @param onProgress - Optional callback for progress updates
 * @returns Updated instance
 */
export async function executeInstanceUpdate(
	instanceId: string,
	plan: InstanceUpdatePlan,
	onProgress?: (progress: UpdateProgress) => void
): Promise<Instance> {
	let unlisten: UnlistenFn | undefined;

	if (onProgress) {
		unlisten = await listen<UpdateProgress>('update_progress', (event) => {
			onProgress(event.payload);
		});
	}

	try {
		return await invoke('execute_instance_update', { instanceId, plan });
	} finally {
		if (unlisten) {
			unlisten();
		}
	}
}
