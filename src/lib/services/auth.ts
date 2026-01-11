import { invoke } from '@tauri-apps/api/core';
import type { DeviceCodeResponse, AuthPollStatus, MinecraftAccount } from '$lib/types';

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
