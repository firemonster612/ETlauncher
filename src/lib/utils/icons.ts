/**
 * Entity icons organized by category
 * Icons from Entity-Icons (CC0 license): https://github.com/Simplexity-Development/Entity-Icons
 */

export interface EntityIcon {
	id: string; // e.g., "creeper" or "cat/black"
	name: string; // Display name e.g., "Creeper" or "Black Cat"
	path: string; // Path relative to /icons/entities/
	category: string; // Category for grouping
}

/** Icon categories */
export const ICON_CATEGORIES = ['Hostile', 'Passive', 'Neutral', 'Aquatic', 'Other'] as const;

export type IconCategory = (typeof ICON_CATEGORIES)[number];

/** All available entity icons */
export const ENTITY_ICONS: EntityIcon[] = [
	// Hostile mobs
	{ id: 'blaze', name: 'Blaze', path: 'blaze/blaze.png', category: 'Hostile' },
	{ id: 'breeze', name: 'Breeze', path: 'breeze/breeze.png', category: 'Hostile' },
	{ id: 'creaking', name: 'Creaking', path: 'creaking/creaking.png', category: 'Hostile' },
	{ id: 'creeper', name: 'Creeper', path: 'creeper/creeper.png', category: 'Hostile' },
	{
		id: 'elder_guardian',
		name: 'Elder Guardian',
		path: 'elder_guardian/elder_guardian.png',
		category: 'Hostile',
	},
	{ id: 'endermite', name: 'Endermite', path: 'endermite/endermite.png', category: 'Hostile' },
	{ id: 'ghast', name: 'Ghast', path: 'ghast/ghast.png', category: 'Hostile' },
	{ id: 'happy_ghast', name: 'Happy Ghast', path: 'ghast/happy_ghast.png', category: 'Hostile' },
	{ id: 'guardian', name: 'Guardian', path: 'guardian/guardian.png', category: 'Hostile' },
	{ id: 'hoglin', name: 'Hoglin', path: 'hoglin/hoglin.png', category: 'Hostile' },
	{ id: 'zoglin', name: 'Zoglin', path: 'hoglin/zombified_hoglin.png', category: 'Hostile' },
	{ id: 'illager_evoker', name: 'Evoker', path: 'illager/evoker.png', category: 'Hostile' },
	{
		id: 'illager_illusioner',
		name: 'Illusioner',
		path: 'illager/illusioner.png',
		category: 'Hostile',
	},
	{ id: 'illager_pillager', name: 'Pillager', path: 'illager/pillager.png', category: 'Hostile' },
	{ id: 'illager_ravager', name: 'Ravager', path: 'illager/ravager.png', category: 'Hostile' },
	{ id: 'illager_vex', name: 'Vex', path: 'illager/vex.png', category: 'Hostile' },
	{
		id: 'illager_vindicator',
		name: 'Vindicator',
		path: 'illager/vindicator.png',
		category: 'Hostile',
	},
	{ id: 'phantom', name: 'Phantom', path: 'phantom/phantom.png', category: 'Hostile' },
	{
		id: 'piglin_brute',
		name: 'Piglin Brute',
		path: 'piglin/piglin_brute.png',
		category: 'Hostile',
	},
	{ id: 'shulker', name: 'Shulker', path: 'shulker/shulker.png', category: 'Hostile' },
	{ id: 'silverfish', name: 'Silverfish', path: 'silverfish/silver_fish.png', category: 'Hostile' },
	{ id: 'skeleton', name: 'Skeleton', path: 'skeleton/skeleton.png', category: 'Hostile' },
	{ id: 'skeleton_stray', name: 'Stray', path: 'skeleton/stray.png', category: 'Hostile' },
	{
		id: 'skeleton_wither',
		name: 'Wither Skeleton',
		path: 'skeleton/wither_skeleton.png',
		category: 'Hostile',
	},
	{ id: 'slime', name: 'Slime', path: 'slime/slime.png', category: 'Hostile' },
	{ id: 'magma_cube', name: 'Magma Cube', path: 'slime/magma_cube.png', category: 'Hostile' },
	{ id: 'spider', name: 'Spider', path: 'spider/spider.png', category: 'Hostile' },
	{ id: 'cave_spider', name: 'Cave Spider', path: 'spider/cave_spider.png', category: 'Hostile' },
	{ id: 'warden', name: 'Warden', path: 'warden/warden.png', category: 'Hostile' },
	{ id: 'witch', name: 'Witch', path: 'witch/witch.png', category: 'Hostile' },
	{ id: 'wither', name: 'Wither', path: 'wither/wither.png', category: 'Hostile' },
	{ id: 'zombie', name: 'Zombie', path: 'zombie/zombie.png', category: 'Hostile' },
	{ id: 'zombie_drowned', name: 'Drowned', path: 'zombie/drowned.png', category: 'Hostile' },
	{ id: 'zombie_husk', name: 'Husk', path: 'zombie/husk.png', category: 'Hostile' },
	{
		id: 'zombie_villager',
		name: 'Zombie Villager',
		path: 'zombie_villager/zombie_villager.png',
		category: 'Hostile',
	},

	// Passive mobs
	{ id: 'allay', name: 'Allay', path: 'allay/allay.png', category: 'Passive' },
	{ id: 'armadillo', name: 'Armadillo', path: 'armadillo/armadillo.png', category: 'Passive' },
	{ id: 'axolotl_blue', name: 'Blue Axolotl', path: 'axolotl/blue.png', category: 'Passive' },
	{ id: 'axolotl_cyan', name: 'Cyan Axolotl', path: 'axolotl/cyan.png', category: 'Passive' },
	{ id: 'axolotl_gold', name: 'Gold Axolotl', path: 'axolotl/gold.png', category: 'Passive' },
	{ id: 'axolotl_pink', name: 'Pink Axolotl', path: 'axolotl/pink.png', category: 'Passive' },
	{ id: 'axolotl_wild', name: 'Wild Axolotl', path: 'axolotl/wild.png', category: 'Passive' },
	{ id: 'bat', name: 'Bat', path: 'bat/bat.png', category: 'Passive' },
	{ id: 'camel', name: 'Camel', path: 'camel/camel.png', category: 'Passive' },
	{ id: 'cat_black', name: 'Black Cat', path: 'cat/all_black.png', category: 'Passive' },
	{
		id: 'cat_british',
		name: 'British Shorthair',
		path: 'cat/british_shorthair.png',
		category: 'Passive',
	},
	{ id: 'cat_calico', name: 'Calico Cat', path: 'cat/calico.png', category: 'Passive' },
	{ id: 'cat_jellie', name: 'Jellie Cat', path: 'cat/jellie.png', category: 'Passive' },
	{ id: 'cat_persian', name: 'Persian Cat', path: 'cat/persian.png', category: 'Passive' },
	{ id: 'cat_ragdoll', name: 'Ragdoll Cat', path: 'cat/ragdoll.png', category: 'Passive' },
	{ id: 'cat_red', name: 'Red Cat', path: 'cat/red.png', category: 'Passive' },
	{ id: 'cat_siamese', name: 'Siamese Cat', path: 'cat/siamese.png', category: 'Passive' },
	{ id: 'cat_tabby', name: 'Tabby Cat', path: 'cat/tabby.png', category: 'Passive' },
	{ id: 'cat_default', name: 'Default Cat', path: 'cat/default.png', category: 'Passive' },
	{ id: 'cat_white', name: 'White Cat', path: 'cat/white.png', category: 'Passive' },
	{ id: 'chicken', name: 'Chicken', path: 'chicken/temperate_chicken.png', category: 'Passive' },
	{ id: 'cow', name: 'Cow', path: 'cow/temperate.png', category: 'Passive' },
	{ id: 'mooshroom', name: 'Mooshroom', path: 'cow/mooshroom.png', category: 'Passive' },
	{
		id: 'mooshroom_brown',
		name: 'Brown Mooshroom',
		path: 'cow/brown_mooshroom.png',
		category: 'Passive',
	},
	{ id: 'donkey', name: 'Donkey', path: 'horse/donkey.png', category: 'Passive' },
	{ id: 'horse_black', name: 'Black Horse', path: 'horse/black.png', category: 'Passive' },
	{ id: 'horse_brown', name: 'Brown Horse', path: 'horse/brown.png', category: 'Passive' },
	{ id: 'horse_chestnut', name: 'Chestnut Horse', path: 'horse/chestnut.png', category: 'Passive' },
	{ id: 'horse_creamy', name: 'Creamy Horse', path: 'horse/creamy.png', category: 'Passive' },
	{ id: 'horse_gray', name: 'Gray Horse', path: 'horse/gray.png', category: 'Passive' },
	{ id: 'horse_white', name: 'White Horse', path: 'horse/white.png', category: 'Passive' },
	{ id: 'mule', name: 'Mule', path: 'horse/mule.png', category: 'Passive' },
	{ id: 'skeleton_horse', name: 'Skeleton Horse', path: 'horse/skeleton.png', category: 'Passive' },
	{ id: 'zombie_horse', name: 'Zombie Horse', path: 'horse/zombie.png', category: 'Passive' },
	{ id: 'llama_brown', name: 'Brown Llama', path: 'llama/brown.png', category: 'Passive' },
	{ id: 'llama_cream', name: 'Cream Llama', path: 'llama/creamy.png', category: 'Passive' },
	{ id: 'llama_gray', name: 'Gray Llama', path: 'llama/gray.png', category: 'Passive' },
	{ id: 'llama_white', name: 'White Llama', path: 'llama/white.png', category: 'Passive' },
	{ id: 'ocelot', name: 'Ocelot', path: 'cat/ocelot.png', category: 'Passive' },
	{ id: 'panda', name: 'Panda', path: 'panda/panda.png', category: 'Passive' },
	{ id: 'parrot_blue', name: 'Blue Parrot', path: 'parrot/blue.png', category: 'Passive' },
	{
		id: 'parrot_red_blue',
		name: 'Red & Blue Parrot',
		path: 'parrot/red_blue.png',
		category: 'Passive',
	},
	{ id: 'parrot_gray', name: 'Gray Parrot', path: 'parrot/gray.png', category: 'Passive' },
	{ id: 'parrot_green', name: 'Green Parrot', path: 'parrot/green.png', category: 'Passive' },
	{
		id: 'parrot_yellow_blue',
		name: 'Yellow & Blue Parrot',
		path: 'parrot/yellow_blue.png',
		category: 'Passive',
	},
	{ id: 'pig', name: 'Pig', path: 'pig/temperate_pig.png', category: 'Passive' },
	{ id: 'rabbit_brown', name: 'Brown Rabbit', path: 'rabbit/brown.png', category: 'Passive' },
	{ id: 'rabbit_white', name: 'White Rabbit', path: 'rabbit/white.png', category: 'Passive' },
	{ id: 'rabbit_black', name: 'Black Rabbit', path: 'rabbit/black.png', category: 'Passive' },
	{ id: 'rabbit_gold', name: 'Gold Rabbit', path: 'rabbit/gold.png', category: 'Passive' },
	{ id: 'sheep_white', name: 'White Sheep', path: 'sheep/white.png', category: 'Passive' },
	{ id: 'sheep_black', name: 'Black Sheep', path: 'sheep/black.png', category: 'Passive' },
	{ id: 'sheep_brown', name: 'Brown Sheep', path: 'sheep/brown.png', category: 'Passive' },
	{ id: 'sheep_pink', name: 'Pink Sheep', path: 'sheep/pink.png', category: 'Passive' },
	{ id: 'sniffer', name: 'Sniffer', path: 'sniffer/sniffer.png', category: 'Passive' },
	{ id: 'strider', name: 'Strider', path: 'strider/warm_strider.png', category: 'Passive' },
	{ id: 'villager', name: 'Villager', path: 'villager/villager.png', category: 'Passive' },
	{
		id: 'wandering_trader',
		name: 'Wandering Trader',
		path: 'wandering_trader/wandering_trader.png',
		category: 'Passive',
	},

	// Neutral mobs
	{ id: 'bee', name: 'Bee', path: 'bee/bee.png', category: 'Neutral' },
	{ id: 'enderman', name: 'Enderman', path: 'enderman/enderman.png', category: 'Neutral' },
	{ id: 'fox', name: 'Fox', path: 'fox/fox.png', category: 'Neutral' },
	{ id: 'fox_snow', name: 'Snow Fox', path: 'fox/snow_fox.png', category: 'Neutral' },
	{ id: 'goat', name: 'Goat', path: 'goat/goat.png', category: 'Neutral' },
	{ id: 'iron_golem', name: 'Iron Golem', path: 'iron_golem/iron_golem.png', category: 'Neutral' },
	{ id: 'piglin', name: 'Piglin', path: 'piglin/piglin.png', category: 'Neutral' },
	{
		id: 'zombified_piglin',
		name: 'Zombified Piglin',
		path: 'piglin/zombified_piglin.png',
		category: 'Neutral',
	},
	{ id: 'polar_bear', name: 'Polar Bear', path: 'bear/polar.png', category: 'Neutral' },
	{
		id: 'snow_golem',
		name: 'Snow Golem',
		path: 'snow_golem/pumpkin_golem.png',
		category: 'Neutral',
	},
	{ id: 'wolf', name: 'Wolf', path: 'wolf/pale.png', category: 'Neutral' },
	{ id: 'wolf_black', name: 'Black Wolf', path: 'wolf/black.png', category: 'Neutral' },
	{ id: 'wolf_snowy', name: 'Snowy Wolf', path: 'wolf/snowy.png', category: 'Neutral' },
	{ id: 'wolf_spotted', name: 'Spotted Wolf', path: 'wolf/spotted.png', category: 'Neutral' },

	// Aquatic mobs
	{ id: 'dolphin', name: 'Dolphin', path: 'dolphin/dolphin.png', category: 'Aquatic' },
	{ id: 'fish_cod', name: 'Cod', path: 'fish/cod.png', category: 'Aquatic' },
	{ id: 'fish_salmon', name: 'Salmon', path: 'fish/salmon.png', category: 'Aquatic' },
	{ id: 'fish_pufferfish', name: 'Pufferfish', path: 'fish/pufferfish.png', category: 'Aquatic' },
	{
		id: 'fish_tropical',
		name: 'Tropical Fish',
		path: 'fish/tropical_fish.png',
		category: 'Aquatic',
	},
	{ id: 'frog_cold', name: 'Cold Frog', path: 'frog/cold.png', category: 'Aquatic' },
	{ id: 'frog_temperate', name: 'Temperate Frog', path: 'frog/temperate.png', category: 'Aquatic' },
	{ id: 'frog_warm', name: 'Warm Frog', path: 'frog/warm.png', category: 'Aquatic' },
	{ id: 'squid', name: 'Squid', path: 'squid/squid.png', category: 'Aquatic' },
	{ id: 'glow_squid', name: 'Glow Squid', path: 'squid/glow_squid.png', category: 'Aquatic' },
	{ id: 'tadpole', name: 'Tadpole', path: 'tadpole/tadpole.png', category: 'Aquatic' },
	{ id: 'turtle', name: 'Turtle', path: 'turtle/turtle.png', category: 'Aquatic' },

	// Other (bosses, special entities)
	{
		id: 'enderdragon',
		name: 'Ender Dragon',
		path: 'enderdragon/ender_dragon.png',
		category: 'Other',
	},
	{
		id: 'end_crystal',
		name: 'End Crystal',
		path: 'end_crystal/end_crystal.png',
		category: 'Other',
	},
	{ id: 'armorstand', name: 'Armor Stand', path: 'armorstand/armorstand.png', category: 'Other' },
];

/** Get icons filtered by category */
export function getIconsByCategory(category: IconCategory): EntityIcon[] {
	return ENTITY_ICONS.filter((icon) => icon.category === category);
}

/** Get all unique categories that have icons */
export function getCategories(): IconCategory[] {
	return ICON_CATEGORIES.filter((cat) => ENTITY_ICONS.some((icon) => icon.category === cat));
}

/** Search icons by name */
export function searchIcons(query: string): EntityIcon[] {
	const lower = query.toLowerCase();
	return ENTITY_ICONS.filter(
		(icon) => icon.name.toLowerCase().includes(lower) || icon.id.toLowerCase().includes(lower)
	);
}

/** Get a random icon */
export function getRandomIcon(): EntityIcon {
	const index = Math.floor(Math.random() * ENTITY_ICONS.length);
	return ENTITY_ICONS[index];
}

/** Get icon by ID */
export function getIconById(id: string): EntityIcon | undefined {
	return ENTITY_ICONS.find((icon) => icon.id === id);
}

/** Get the full URL path for an icon */
export function getIconUrl(icon: EntityIcon | string): string {
	if (typeof icon === 'string') {
		const found = getIconById(icon);
		if (found) return `/icons/entities/${found.path}`;
		return `/icons/entities/creeper/creeper.png`;
	}
	return `/icons/entities/${icon.path}`;
}

/**
 * Parse an icon path stored in instance config
 * Format: "entity:iconId" e.g., "entity:creeper" or "entity:cat_black"
 */
export function parseIconPath(iconPath: string | undefined): EntityIcon | undefined {
	if (!iconPath) return undefined;
	if (!iconPath.startsWith('entity:')) return undefined;
	const id = iconPath.replace('entity:', '');
	return getIconById(id);
}

/** Create an icon path for storage */
export function makeIconPath(icon: EntityIcon): string {
	return `entity:${icon.id}`;
}

/** Check if a path is an entity icon */
export function isEntityIcon(iconPath: string | undefined): boolean {
	return iconPath?.startsWith('entity:') ?? false;
}
