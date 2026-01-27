import type { Instance } from './instance';

/** Screenshot data with instance context for homepage display */
export interface HomepageScreenshot {
	instanceId: string;
	instanceName: string;
	filename: string;
	path: string;
	takenAt: number;
}

/** World data with instance context for homepage display */
export interface HomepageWorld {
	instanceId: string;
	instanceName: string;
	minecraftVersion: string;
	folderName: string;
	name: string;
	lastPlayed: number | null;
	iconBase64: string | null;
	/** Whether this world supports quick play (MC >= 1.20) */
	supportsQuickPlay: boolean;
}

/** Aggregated homepage data */
export interface HomepageData {
	recentScreenshots: HomepageScreenshot[];
	mostPlayedInstances: Instance[];
	mostPlayedWorlds: HomepageWorld[];
}

/** A news article from Minecraft launcher news API */
export interface NewsArticle {
	id: string;
	title: string;
	description: string;
	imageUrl: string;
	articleUrl: string;
	date: string;
	category: string;
}

/** Response from the news API */
export interface NewsResponse {
	articles: NewsArticle[];
}
