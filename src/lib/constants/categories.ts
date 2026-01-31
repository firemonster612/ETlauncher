import type { ContentPlatform, ContentType } from '$lib/types';

export interface CategoryOption {
	value: string;
	label: string;
}

/**
 * Modrinth category slugs by content type
 * These are used as facets: categories:{slug}
 */
export const MODRINTH_CATEGORIES: Record<ContentType, CategoryOption[]> = {
	mod: [
		{ value: 'adventure', label: 'Adventure' },
		{ value: 'cursed', label: 'Cursed' },
		{ value: 'decoration', label: 'Decoration' },
		{ value: 'economy', label: 'Economy' },
		{ value: 'equipment', label: 'Equipment' },
		{ value: 'food', label: 'Food' },
		{ value: 'game-mechanics', label: 'Game Mechanics' },
		{ value: 'library', label: 'Library' },
		{ value: 'magic', label: 'Magic' },
		{ value: 'management', label: 'Management' },
		{ value: 'minigame', label: 'Minigame' },
		{ value: 'mobs', label: 'Mobs' },
		{ value: 'optimization', label: 'Optimization' },
		{ value: 'social', label: 'Social' },
		{ value: 'storage', label: 'Storage' },
		{ value: 'technology', label: 'Technology' },
		{ value: 'transportation', label: 'Transportation' },
		{ value: 'utility', label: 'Utility' },
		{ value: 'world-generation', label: 'World Generation' },
	],
	shader: [
		{ value: 'atmosphere', label: 'Atmosphere' },
		{ value: 'bloom', label: 'Bloom' },
		{ value: 'cartoon', label: 'Cartoon' },
		{ value: 'cursed', label: 'Cursed' },
		{ value: 'fantasy', label: 'Fantasy' },
		{ value: 'foliage', label: 'Foliage' },
		{ value: 'low-end', label: 'Low End' },
		{ value: 'path-tracing', label: 'Path Tracing' },
		{ value: 'pbr', label: 'PBR' },
		{ value: 'realistic', label: 'Realistic' },
		{ value: 'reflections', label: 'Reflections' },
		{ value: 'semi-realistic', label: 'Semi-Realistic' },
		{ value: 'shadows', label: 'Shadows' },
		{ value: 'vanilla-like', label: 'Vanilla-like' },
	],
	resourcepack: [
		{ value: '16x', label: '16x' },
		{ value: '32x', label: '32x' },
		{ value: '48x', label: '48x' },
		{ value: '64x', label: '64x' },
		{ value: '128x', label: '128x' },
		{ value: '256x', label: '256x' },
		{ value: '512x+', label: '512x+' },
		{ value: 'audio', label: 'Audio' },
		{ value: 'blocks', label: 'Blocks' },
		{ value: 'combat', label: 'Combat' },
		{ value: 'core-shaders', label: 'Core Shaders' },
		{ value: 'cursed', label: 'Cursed' },
		{ value: 'decoration', label: 'Decoration' },
		{ value: 'entities', label: 'Entities' },
		{ value: 'environment', label: 'Environment' },
		{ value: 'equipment', label: 'Equipment' },
		{ value: 'fonts', label: 'Fonts' },
		{ value: 'gui', label: 'GUI' },
		{ value: 'items', label: 'Items' },
		{ value: 'locale', label: 'Locale' },
		{ value: 'mobs', label: 'Mobs' },
		{ value: 'modded', label: 'Modded' },
		{ value: 'models', label: 'Models' },
		{ value: 'realistic', label: 'Realistic' },
		{ value: 'simplistic', label: 'Simplistic' },
		{ value: 'themed', label: 'Themed' },
		{ value: 'tweaks', label: 'Tweaks' },
		{ value: 'utility', label: 'Utility' },
		{ value: 'vanilla-like', label: 'Vanilla-like' },
	],
	datapack: [
		{ value: 'adventure', label: 'Adventure' },
		{ value: 'cursed', label: 'Cursed' },
		{ value: 'decoration', label: 'Decoration' },
		{ value: 'economy', label: 'Economy' },
		{ value: 'equipment', label: 'Equipment' },
		{ value: 'food', label: 'Food' },
		{ value: 'game-mechanics', label: 'Game Mechanics' },
		{ value: 'library', label: 'Library' },
		{ value: 'magic', label: 'Magic' },
		{ value: 'management', label: 'Management' },
		{ value: 'minigame', label: 'Minigame' },
		{ value: 'mobs', label: 'Mobs' },
		{ value: 'optimization', label: 'Optimization' },
		{ value: 'social', label: 'Social' },
		{ value: 'storage', label: 'Storage' },
		{ value: 'technology', label: 'Technology' },
		{ value: 'transportation', label: 'Transportation' },
		{ value: 'utility', label: 'Utility' },
		{ value: 'world-generation', label: 'World Generation' },
	],
	world: [],
};

/**
 * CurseForge category IDs by content type
 * These are passed as categoryId={id} parameter
 */
export const CURSEFORGE_CATEGORIES: Record<ContentType, CategoryOption[]> = {
	mod: [
		{ value: '434', label: 'Adventure and RPG' },
		{ value: '425', label: 'Armor, Tools, and Weapons' },
		{ value: '407', label: 'Biomes' },
		{ value: '420', label: 'Cosmetic' },
		{ value: '428', label: 'Food' },
		{ value: '419', label: 'Library & API' },
		{ value: '424', label: 'Magic' },
		{ value: '426', label: 'Mobs' },
		{ value: '408', label: 'Ores and Resources' },
		{ value: '422', label: 'Redstone' },
		{ value: '421', label: 'Server Utility' },
		{ value: '436', label: 'Storage' },
		{ value: '409', label: 'Structures' },
		{ value: '412', label: 'Technology' },
		{ value: '5191', label: 'Utility & QoL' },
		{ value: '406', label: 'World Gen' },
	],
	shader: [
		// CurseForge shaders use classId 6552
		// Categories are limited
	],
	resourcepack: [
		{ value: '393', label: '16x' },
		{ value: '394', label: '32x' },
		{ value: '395', label: '64x' },
		{ value: '396', label: '128x' },
		{ value: '397', label: '256x' },
		{ value: '398', label: '512x and Higher' },
		{ value: '399', label: 'Font' },
		{ value: '400', label: 'GUI' },
		{ value: '401', label: 'Map' },
		{ value: '402', label: 'Misc' },
		{ value: '403', label: 'Mod Support' },
		{ value: '4555', label: 'Photo Realistic' },
		{ value: '404', label: 'Steampunk' },
		{ value: '4556', label: 'Traditional' },
	],
	datapack: [
		{ value: '6948', label: 'Adventure' },
		{ value: '6949', label: 'Fantasy' },
		{ value: '6950', label: 'Library' },
		{ value: '6952', label: 'Magic' },
		{ value: '6947', label: 'Miscellaneous' },
		{ value: '6946', label: 'Mod Support' },
		{ value: '6951', label: 'Tech' },
		{ value: '6953', label: 'Utility' },
	],
	world: [
		{ value: '248', label: 'Adventure' },
		{ value: '249', label: 'Creation' },
		{ value: '250', label: 'Game Map' },
		{ value: '4464', label: 'Modded World' },
		{ value: '251', label: 'Parkour' },
		{ value: '252', label: 'Puzzle' },
		{ value: '253', label: 'Survival' },
	],
};

/**
 * Get available categories for the current platform and content type
 */
export function getCategoriesForContext(
	platform: ContentPlatform,
	contentType: ContentType
): CategoryOption[] {
	if (platform === 'modrinth') {
		return MODRINTH_CATEGORIES[contentType] ?? [];
	}
	return CURSEFORGE_CATEGORIES[contentType] ?? [];
}
