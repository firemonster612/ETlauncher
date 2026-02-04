import { invoke } from '@tauri-apps/api/core';
import type { SavedSkin, MinecraftProfile } from '$lib/types';

/** Save a skin to the local library */
export async function saveSkinToLibrary(
	name: string,
	variant: 'classic' | 'slim',
	skinData: Uint8Array
): Promise<SavedSkin> {
	return invoke<SavedSkin>('save_skin_to_library', {
		name,
		variant,
		skinData: Array.from(skinData),
	});
}

/** Get all saved skins from the library */
export async function getSkinLibrary(): Promise<SavedSkin[]> {
	return invoke<SavedSkin[]>('get_skin_library');
}

/** Delete a skin from the library */
export async function deleteSkinFromLibrary(skinId: string): Promise<void> {
	return invoke('delete_skin_from_library', { skinId });
}

/** Apply a skin from the library to a Minecraft account */
export async function applySkinFromLibrary(
	accountId: string,
	skinId: string
): Promise<MinecraftProfile> {
	return invoke<MinecraftProfile>('apply_skin_from_library', { accountId, skinId });
}

/** Get skin data from the library (for preview) */
export async function getSkinData(skinId: string): Promise<Uint8Array> {
	const data = await invoke<number[]>('get_skin_data', { skinId });
	return new Uint8Array(data);
}

/** Read a skin file from a path */
export async function readSkinFile(filePath: string): Promise<Uint8Array> {
	const data = await invoke<number[]>('read_skin_file', { filePath });
	return new Uint8Array(data);
}

/** Convert Uint8Array to data URL for use in img elements or skinview3d */
export function skinDataToUrl(data: Uint8Array): string {
	// Create a new Uint8Array copy to ensure proper BlobPart compatibility
	const copy = new Uint8Array(data);
	const blob = new Blob([copy], { type: 'image/png' });
	return URL.createObjectURL(blob);
}

/** Validate skin image dimensions */
export function validateSkinImage(
	width: number,
	height: number
): { valid: boolean; error?: string } {
	if (width === 64 && height === 64) {
		return { valid: true };
	}
	if (width === 64 && height === 32) {
		return { valid: true };
	}
	return {
		valid: false,
		error: `Invalid skin dimensions: ${width}x${height}. Must be 64x64 or 64x32.`,
	};
}
