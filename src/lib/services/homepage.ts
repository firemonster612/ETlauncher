import { invoke } from '@tauri-apps/api/core';
import type { HomepageData, NewsResponse } from '$lib/types';

/**
 * Get aggregated homepage data (screenshots, instances, worlds)
 */
export async function getHomepageData(): Promise<HomepageData> {
	return invoke<HomepageData>('get_homepage_data');
}

/**
 * Get Minecraft news articles
 */
export async function getMinecraftNews(): Promise<NewsResponse> {
	return invoke<NewsResponse>('get_minecraft_news');
}
