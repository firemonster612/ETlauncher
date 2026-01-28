import { invoke } from '@tauri-apps/api/core';
import type { AppSettings, UpdateSettingsRequest, ResourcePoolStats } from '$lib/types';

/** Get current application settings */
export async function getSettings(): Promise<AppSettings> {
	return invoke<AppSettings>('get_settings');
}

/** Update application settings */
export async function updateSettings(updates: Partial<AppSettings>): Promise<AppSettings> {
	// Convert frontend naming to backend naming (camelCase to snake_case)
	const request: UpdateSettingsRequest = {
		instancesPath: updates.instancesPath,
		memoryMinMb: updates.memoryMinMb,
		memoryMaxMb: updates.memoryMaxMb,
		concurrentDownloads: updates.concurrentDownloads,
		closeLauncherOnGameStart: updates.closeLauncherOnGameStart,
		reopenLauncherOnGameClose: updates.reopenLauncherOnGameClose,
		showSnapshots: updates.showSnapshots,
		showOldVersions: updates.showOldVersions,
		theme: updates.theme,
		colorPreset: updates.colorPreset,
		customColors: updates.customColors,
		disableHoverLift: updates.disableHoverLift,
		fontFamily: updates.fontFamily,
		customFont: updates.customFont,
		sidebarStyle: updates.sidebarStyle,
		customSidebarColor: updates.customSidebarColor,
		setupCompleted: updates.setupCompleted,
		curseforgeApiKey: updates.curseforgeApiKey,
		resourcePool: updates.resourcePool,
		autoUpdate: updates.autoUpdate,
		includePreReleases: updates.includePreReleases,
	};

	return invoke<AppSettings>('update_settings', { updates: request });
}

/** Reset settings to defaults */
export async function resetSettings(): Promise<AppSettings> {
	return invoke<AppSettings>('reset_settings');
}

/** Get the default instances path */
export async function getDefaultInstancesPath(): Promise<string> {
	return invoke<string>('get_default_instances_path');
}

/** Get the system theme (light or dark) from Tauri's native detection */
export async function getSystemTheme(): Promise<'light' | 'dark'> {
	const theme = await invoke<string>('get_system_theme');
	return theme === 'light' ? 'light' : 'dark';
}

// =============================================================================
// Resource Pool Functions
// =============================================================================

/** Get resource pool statistics */
export async function getPoolStats(): Promise<ResourcePoolStats> {
	return invoke<ResourcePoolStats>('get_pool_stats');
}

/** Result of garbage collection */
export interface GarbageCollectResult {
	resourcesRemoved: number;
	bytesFreed: number;
	failedCount: number;
}

/** Run garbage collection on the resource pool */
export async function garbageCollectPool(): Promise<GarbageCollectResult> {
	return invoke<GarbageCollectResult>('garbage_collect_pool');
}

/** Result of pool integrity verification */
export interface PoolIntegrityResult {
	validResources: number;
	missingFiles: number;
	orphanedFiles: number;
}

/** Verify resource pool integrity */
export async function verifyPoolIntegrity(): Promise<PoolIntegrityResult> {
	return invoke<PoolIntegrityResult>('verify_pool_integrity');
}

/** Result of migrating an instance */
export interface InstanceMigrationResult {
	instanceId: string;
	filesMigrated: number;
	spaceSavedBytes: number;
	errorCount: number;
}

/** Check if an instance needs migration to the resource pool */
export async function checkInstanceNeedsMigration(instanceId: string): Promise<boolean> {
	return invoke<boolean>('check_instance_needs_migration', { instanceId });
}

/** Migrate a single instance to use the resource pool */
export async function migrateInstanceToPool(instanceId: string): Promise<InstanceMigrationResult> {
	return invoke<InstanceMigrationResult>('migrate_instance_to_pool', { instanceId });
}

/** Result of migrating all instances */
export interface MigrateAllResult {
	instancesMigrated: number;
	totalFilesMigrated: number;
	totalSpaceSavedBytes: number;
}

/** Migrate all instances to use the resource pool */
export async function migrateAllInstancesToPool(): Promise<MigrateAllResult> {
	return invoke<MigrateAllResult>('migrate_all_instances_to_pool');
}
