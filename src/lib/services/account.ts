import { invoke } from '@tauri-apps/api/core';
import type { MinecraftAccount, MinecraftProfile } from '$lib/types';

/** Get all accounts */
export async function getAccounts(): Promise<MinecraftAccount[]> {
	return invoke<MinecraftAccount[]>('get_accounts');
}

/** Get account by ID */
export async function getAccount(accountId: string): Promise<MinecraftAccount> {
	return invoke<MinecraftAccount>('get_account', { accountId });
}

/** Get the active account */
export async function getActiveAccount(): Promise<MinecraftAccount | null> {
	return invoke<MinecraftAccount | null>('get_active_account');
}

/** Set account as active */
export async function setActiveAccount(accountId: string): Promise<MinecraftAccount[]> {
	return invoke<MinecraftAccount[]>('set_active_account', { accountId });
}

/** Delete an account (logout) */
export async function deleteAccount(accountId: string): Promise<MinecraftAccount[]> {
	return invoke<MinecraftAccount[]>('delete_account', { accountId });
}

/** Get Minecraft profile with skins and capes */
export async function getMinecraftProfile(accountId: string): Promise<MinecraftProfile> {
	return invoke<MinecraftProfile>('get_minecraft_profile', { accountId });
}

/** Upload a new skin */
export async function uploadSkin(
	accountId: string,
	variant: 'classic' | 'slim',
	skinData: Uint8Array
): Promise<MinecraftProfile> {
	return invoke<MinecraftProfile>('upload_skin', {
		accountId,
		variant,
		skinData: Array.from(skinData),
	});
}

/** Set active cape */
export async function setCape(accountId: string, capeId: string): Promise<MinecraftProfile> {
	return invoke<MinecraftProfile>('set_cape', { accountId, capeId });
}

/** Hide cape (remove active cape) */
export async function hideCape(accountId: string): Promise<MinecraftProfile> {
	return invoke<MinecraftProfile>('hide_cape', { accountId });
}

/** Get avatar URL for a player */
export function getAvatarUrl(username: string, size: number = 64): string {
	return `https://minotar.net/avatar/${username}/${size}`;
}

/** Get skin render URL for a player */
export function getSkinRenderUrl(username: string, size: number = 128): string {
	return `https://minotar.net/body/${username}/${size}`;
}
