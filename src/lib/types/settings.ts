/** Application theme */
export type Theme = 'dark' | 'light' | 'system';

/** Color preset for accent color */
export type ColorPreset = 'default' | 'purple' | 'green' | 'orange' | 'pink' | 'blue' | 'custom';

/** Custom theme colors */
export interface ThemeColors {
	/** Primary hue (0-360) */
	primaryHue?: number;
	/** Primary chroma (0-0.4) */
	primaryChroma?: number;
}

/** Font family preference - preset name or custom font string */
export type FontFamily = 'pixel' | 'system' | 'custom';

/** Custom font configuration */
export interface CustomFont {
	/** Font family name (e.g., "Arial", "Roboto") */
	family: string;
}

/** Sidebar/titlebar style preset */
export type SidebarStyle = 'default' | 'accent' | 'custom';

/** Custom sidebar color configuration */
export interface CustomSidebarColor {
	/** Sidebar hue (0-360) */
	hue: number;
	/** Sidebar chroma (0-0.35) */
	chroma: number;
}

/** Link strategy for resource pool */
export type LinkStrategy = 'auto' | 'hardLink' | 'symlink' | 'copy';

/** Background type for app customization */
export type BackgroundType = 'none' | 'color' | 'image' | 'video' | 'gif';

/** Background configuration */
export interface BackgroundConfig {
	/** Type of background */
	type: BackgroundType;
	/** Hex color (for type='color') */
	color?: string;
	/** Stored filename in app data (for media types) */
	filename?: string;
	/** Opacity 0-1 (for media types) */
	opacity?: number;
	/** Blur 0-20px (for media types) */
	blur?: number;
}

/** Resource pool configuration */
export interface ResourcePoolConfig {
	/** Whether the resource pool is enabled */
	enabled: boolean;
	/** Preferred link strategy */
	linkStrategy: LinkStrategy;
}

/** Global application settings */
export interface AppSettings {
	/** Path to instances directory */
	instancesPath: string;
	/** Minimum memory allocation (MB) */
	memoryMinMb: number;
	/** Maximum memory allocation (MB) */
	memoryMaxMb: number;
	/** Number of concurrent downloads */
	concurrentDownloads: number;
	/** Close launcher when game starts */
	closeLauncherOnGameStart: boolean;
	/** Reopen launcher when game closes */
	reopenLauncherOnGameClose: boolean;
	/** Show snapshot versions in version list */
	showSnapshots: boolean;
	/** Show old_alpha and old_beta versions */
	showOldVersions: boolean;
	/** UI theme */
	theme: Theme;
	/** Color preset for accent color */
	colorPreset?: ColorPreset;
	/** Custom theme colors (used when colorPreset is 'custom') */
	customColors?: ThemeColors;
	/** Disable hover lift effect on buttons */
	disableHoverLift?: boolean;
	/** Font family preference */
	fontFamily?: FontFamily;
	/** Custom font configuration (used when fontFamily is 'custom') */
	customFont?: CustomFont;
	/** Sidebar/titlebar style */
	sidebarStyle?: SidebarStyle;
	/** Custom sidebar color (used when sidebarStyle is 'custom') */
	customSidebarColor?: CustomSidebarColor;
	/** Whether the first-launch setup/tutorial has been completed */
	setupCompleted: boolean;
	/** CurseForge API key for accessing CurseForge content */
	curseforgeApiKey?: string;
	/** Resource pool configuration for shared content management */
	resourcePool: ResourcePoolConfig;
	/** Whether auto-updates are enabled (default: true) */
	autoUpdate: boolean;
	/** Whether to include pre-release versions in updates */
	includePreReleases: boolean;
	/** Background customization configuration */
	background?: BackgroundConfig;
}

/** Request to update settings (partial update) */
export interface UpdateSettingsRequest {
	instancesPath?: string;
	memoryMinMb?: number;
	memoryMaxMb?: number;
	concurrentDownloads?: number;
	closeLauncherOnGameStart?: boolean;
	reopenLauncherOnGameClose?: boolean;
	showSnapshots?: boolean;
	showOldVersions?: boolean;
	theme?: Theme;
	colorPreset?: ColorPreset;
	customColors?: ThemeColors;
	disableHoverLift?: boolean;
	fontFamily?: FontFamily;
	customFont?: CustomFont;
	sidebarStyle?: SidebarStyle;
	customSidebarColor?: CustomSidebarColor;
	setupCompleted?: boolean;
	curseforgeApiKey?: string;
	resourcePool?: ResourcePoolConfig;
	autoUpdate?: boolean;
	includePreReleases?: boolean;
	background?: BackgroundConfig;
}

/** Resource pool statistics */
export interface ResourcePoolStats {
	/** Total number of resources in the pool */
	totalResources: number;
	/** Number of mods in the pool */
	modCount: number;
	/** Number of shaders in the pool */
	shaderCount: number;
	/** Number of resource packs in the pool */
	resourcePackCount: number;
	/** Total size of the pool in bytes */
	totalSizeBytes: number;
	/** Estimated space saved by deduplication (bytes) */
	spaceSavedBytes: number;
	/** Number of unused resources (candidates for GC) */
	unusedCount: number;
	/** Last garbage collection timestamp */
	lastGcAt: number | null;
	/** Size of cached assets (textures, sounds, etc.) in bytes */
	assetsCacheSize: number;
	/** Size of cached libraries (Java libraries) in bytes */
	librariesCacheSize: number;
}
