import { invoke } from '@tauri-apps/api/core';
import type { AuthPollStatus, DeviceCodeResponse, MinecraftAccount } from '$lib/types';

/** Start device code authentication flow */
export async function startDeviceAuth(): Promise<DeviceCodeResponse> {
	return invoke<DeviceCodeResponse>('start_device_auth');
}

/** Poll for device code authentication status */
export async function pollDeviceAuth(deviceCode: string): Promise<AuthPollStatus> {
	return invoke<AuthPollStatus>('poll_device_auth', { deviceCode });
}

/** Refresh account token */
export async function refreshAccountToken(accountId: string): Promise<MinecraftAccount> {
	return invoke<MinecraftAccount>('refresh_account_token', { accountId });
}

/** Create a new offline account */
export async function createOfflineAccount(username: string): Promise<MinecraftAccount> {
	return invoke<MinecraftAccount>('create_offline_account', { username });
}

/** Set skin for an offline account */
export async function setOfflineSkin(
	accountId: string,
	skinData: Uint8Array,
	variant: 'classic' | 'slim'
): Promise<string> {
	return invoke<string>('set_offline_skin', {
		accountId,
		skinData: Array.from(skinData),
		variant,
	});
}

/** Set cape for an offline account */
export async function setOfflineCape(accountId: string, capeData: Uint8Array): Promise<string> {
	return invoke<string>('set_offline_cape', {
		accountId,
		capeData: Array.from(capeData),
	});
}

/** Remove skin from an offline account */
export async function removeOfflineSkin(accountId: string): Promise<void> {
	return invoke<void>('remove_offline_skin', { accountId });
}

/** Remove cape from an offline account */
export async function removeOfflineCape(accountId: string): Promise<void> {
	return invoke<void>('remove_offline_cape', { accountId });
}

/** Get offline skin texture as a data URL */
export async function getOfflineSkinData(accountId: string): Promise<string | null> {
	return invoke<string | null>('get_offline_skin_data', { accountId });
}

/** Get offline cape texture as a data URL */
export async function getOfflineCapeData(accountId: string): Promise<string | null> {
	return invoke<string | null>('get_offline_cape_data', { accountId });
}

/** Check if the OS keyring is available for secure token storage */
export async function isKeyringAvailable(): Promise<boolean> {
	return invoke<boolean>('is_keyring_available');
}

/** Get a bundled default skin (steve or alex) as raw PNG bytes */
export async function getDefaultSkin(name: 'steve' | 'alex'): Promise<Uint8Array> {
	const data = await invoke<number[]>('get_default_skin', { name });
	return new Uint8Array(data);
}
