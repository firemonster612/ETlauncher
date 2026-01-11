import { invoke } from '@tauri-apps/api/core';
import type { Instance, CreateInstanceRequest, UpdateInstanceRequest } from '$lib/types';

/** Get all instances */
export async function getInstances(): Promise<Instance[]> {
	return invoke<Instance[]>('get_instances');
}

/** Get a single instance by ID */
export async function getInstance(instanceId: string): Promise<Instance> {
	return invoke<Instance>('get_instance', { instanceId });
}

/** Create a new instance */
export async function createInstance(request: CreateInstanceRequest): Promise<Instance> {
	return invoke<Instance>('create_instance', { request });
}

/** Update an existing instance */
export async function updateInstance(
	instanceId: string,
	updates: UpdateInstanceRequest
): Promise<Instance> {
	return invoke<Instance>('update_instance', { instanceId, updates });
}

/** Delete an instance */
export async function deleteInstance(instanceId: string, deleteFiles: boolean): Promise<void> {
	return invoke('delete_instance', { instanceId, deleteFiles });
}

/** Duplicate an instance with a new name */
export async function duplicateInstance(instanceId: string, newName: string): Promise<Instance> {
	return invoke<Instance>('duplicate_instance', { instanceId, newName });
}

/** Open the game folder for an instance in the file explorer */
export async function openInstanceFolder(instanceId: string): Promise<void> {
	return invoke('open_instance_folder', { instanceId });
}

/** Export an instance to Modrinth .mrpack format */
export async function exportInstance(instanceId: string, outputPath: string): Promise<string> {
	return invoke<string>('export_instance', { instanceId, outputPath });
}
