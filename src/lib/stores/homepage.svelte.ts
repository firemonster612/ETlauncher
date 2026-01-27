import type { HomepageData, NewsArticle, HomepageScreenshot, HomepageWorld, Instance } from '$lib/types';
import * as homepageService from '$lib/services/homepage';

/** Create the homepage store */
function createHomepageStore() {
	// Homepage data state
	let recentScreenshots = $state<HomepageScreenshot[]>([]);
	let mostPlayedInstances = $state<Instance[]>([]);
	let mostPlayedWorlds = $state<HomepageWorld[]>([]);
	let isLoadingData = $state(false);
	let dataError = $state<string | null>(null);

	// News state
	let newsArticles = $state<NewsArticle[]>([]);
	let isLoadingNews = $state(false);
	let newsError = $state<string | null>(null);

	return {
		// Getters for homepage data
		get recentScreenshots() {
			return recentScreenshots;
		},
		get mostPlayedInstances() {
			return mostPlayedInstances;
		},
		get mostPlayedWorlds() {
			return mostPlayedWorlds;
		},
		get isLoadingData() {
			return isLoadingData;
		},
		get dataError() {
			return dataError;
		},

		// Getters for news
		get newsArticles() {
			return newsArticles;
		},
		get isLoadingNews() {
			return isLoadingNews;
		},
		get newsError() {
			return newsError;
		},

		/** Load homepage data (screenshots, instances, worlds) */
		async loadData() {
			isLoadingData = true;
			dataError = null;

			try {
				const data: HomepageData = await homepageService.getHomepageData();
				recentScreenshots = data.recentScreenshots;
				mostPlayedInstances = data.mostPlayedInstances;
				mostPlayedWorlds = data.mostPlayedWorlds;
			} catch (e: unknown) {
				dataError = e instanceof Error ? e.message : typeof e === 'string' ? e : JSON.stringify(e);
				console.error('Failed to load homepage data:', e);
			} finally {
				isLoadingData = false;
			}
		},

		/** Load Minecraft news */
		async loadNews() {
			isLoadingNews = true;
			newsError = null;

			try {
				const response = await homepageService.getMinecraftNews();
				newsArticles = response.articles;
			} catch (e: unknown) {
				newsError = e instanceof Error ? e.message : typeof e === 'string' ? e : JSON.stringify(e);
				console.error('Failed to load news:', e);
			} finally {
				isLoadingNews = false;
			}
		},

		/** Load all homepage content (data + news) */
		async loadAll() {
			// Load both in parallel
			await Promise.all([this.loadData(), this.loadNews()]);
		},

		/** Refresh homepage data */
		async refresh() {
			await this.loadData();
		},

		/** Clear errors */
		clearErrors() {
			dataError = null;
			newsError = null;
		},
	};
}

/** Global homepage store instance */
export const homepageStore = createHomepageStore();
