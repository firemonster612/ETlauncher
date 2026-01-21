/** Application theme */
export type Theme = 'dark' | 'light' | 'system';

/** Link strategy for resource pool */
export type LinkStrategy = 'auto' | 'hardLink' | 'symlink' | 'copy';

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
	/** Whether the first-launch setup/tutorial has been completed */
	setupCompleted: boolean;
	/** CurseForge API key for accessing CurseForge content */
	curseforgeApiKey?: string;
	/** Resource pool configuration for shared content management */
	resourcePool: ResourcePoolConfig;
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
	setupCompleted?: boolean;
	curseforgeApiKey?: string;
	resourcePool?: ResourcePoolConfig;
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
}
