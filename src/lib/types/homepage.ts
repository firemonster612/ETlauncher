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
	/** Game mode (survival, creative, adventure, spectator) */
	gameMode: string | null;
	/** Whether this world supports quick play (MC >= 1.20) */
	supportsQuickPlay: boolean;
}

/** Server data with instance context for homepage display */
export interface HomepageServer {
	instanceId: string;
	instanceName: string;
	name: string;
	ip: string;
	iconBase64: string | null;
}

/** Aggregated stats for homepage display */
export interface HomepageStats {
	/** Total play time across all instances in seconds */
	totalPlayTime: number;
	/** Total number of instances */
	instanceCount: number;
	/** Total number of worlds across all instances */
	worldCount: number;
	/** Total number of screenshots across all instances */
	screenshotCount: number;
}

/** Aggregated homepage data */
export interface HomepageData {
	recentScreenshots: HomepageScreenshot[];
	mostPlayedInstances: Instance[];
	mostPlayedWorlds: HomepageWorld[];
	/** Last played instance for "Continue Playing" section */
	continueInstance: Instance | null;
	/** Aggregated servers from all instances */
	favoriteServers: HomepageServer[];
	/** Aggregated stats */
	stats: HomepageStats;
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
