import { invoke } from '@tauri-apps/api/core';
import type { VersionManifest, VersionEntry, VersionInfo } from '$lib/types';

/** Fetch the version manifest from Mojang */
export async function fetchVersionManifest(
	forceRefresh: boolean = false
): Promise<VersionManifest> {
	return invoke<VersionManifest>('fetch_version_manifest', { forceRefresh });
}

/** Get filtered versions */
export async function getVersions(
	showSnapshots: boolean = false,
	showOldVersions: boolean = false
): Promise<VersionEntry[]> {
	return invoke<VersionEntry[]>('get_versions', { showSnapshots, showOldVersions });
}

/** Get detailed version info */
export async function getVersionInfo(versionId: string): Promise<VersionInfo> {
	return invoke<VersionInfo>('get_version_info', { versionId });
}

/** Download all game files for an instance */
export async function downloadGameFiles(instanceId: string, versionId: string): Promise<void> {
	return invoke('download_game_files', { instanceId, versionId });
}
