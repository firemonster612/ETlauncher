import { invoke } from '@tauri-apps/api/core';
import type {
	InstanceDetail,
	ScreenshotsResponse,
	ServersResponse,
	WorldsResponse,
} from '$lib/types';

export async function getInstanceDetail(instanceId: string): Promise<InstanceDetail> {
	return invoke<InstanceDetail>('get_instance_detail', { instanceId });
}

export async function getInstanceScreenshots(instanceId: string): Promise<ScreenshotsResponse> {
	return invoke<ScreenshotsResponse>('get_instance_screenshots', { instanceId });
}

export async function getInstanceWorlds(instanceId: string): Promise<WorldsResponse> {
	return invoke<WorldsResponse>('get_instance_worlds', { instanceId });
}

export async function getInstanceServers(instanceId: string): Promise<ServersResponse> {
	return invoke<ServersResponse>('get_instance_servers', { instanceId });
}

export async function getScreenshotData(instanceId: string, filename: string): Promise<string> {
	return invoke<string>('get_screenshot_data', { instanceId, filename });
}

export async function launchIntoWorld(
	instanceId: string,
	accountId: string,
	worldFolder: string
): Promise<number> {
	return invoke<number>('launch_into_world', { instanceId, accountId, worldFolder });
}

export async function launchIntoServer(
	instanceId: string,
	accountId: string,
	serverIp: string
): Promise<number> {
	return invoke<number>('launch_into_server', { instanceId, accountId, serverIp });
}

export async function openWorldFolder(instanceId: string, worldFolder: string): Promise<void> {
	return invoke<void>('open_world_folder', { instanceId, worldFolder });
}

export async function deleteScreenshot(instanceId: string, filename: string): Promise<void> {
	return invoke<void>('delete_screenshot', { instanceId, filename });
}

export async function deleteWorld(instanceId: string, worldFolder: string): Promise<void> {
	return invoke<void>('delete_world', { instanceId, worldFolder });
}
