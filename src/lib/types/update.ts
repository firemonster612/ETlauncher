import type { LoaderType, ModpackPlatform } from './instance';
import type { ContentPlatform, ContentSource, InstalledContent } from './content';
import type { LoaderVersion } from './loader';

/** Status of a content item's update availability */
export type ContentUpdateStatus =
	| {
			type: 'updateAvailable';
			currentVersion: string;
			availableVersion: string;
			availableVersionId: string;
	  }
	| { type: 'upToDate' }
	| { type: 'noCompatibleVersion' }
	| { type: 'unidentified' }
	| { type: 'unavailable' };

/** Information about a content item's update status */
export interface ContentUpdateInfo {
	/** Filename of the content */
	filename: string;
	/** Display name */
	name: string;
	/** Source of the content (modpack original, user added, etc.) */
	source: ContentSource;
	/** Platform the content is from */
	platform?: ContentPlatform;
	/** Project ID on the platform */
	projectId?: string;
	/** Current installed version ID */
	currentVersionId?: string;
	/** Update status */
	status: ContentUpdateStatus;
}

/** Information about an available modpack update */
export interface ModpackUpdateInfo {
	/** Platform the modpack is from */
	platform: ModpackPlatform;
	/** Modpack ID on the platform */
	modpackId: string;
	/** Currently installed version ID */
	currentVersionId: string;
	/** Currently installed version name */
	currentVersionName: string;
	/** Available update version ID */
	availableVersionId: string;
	/** Available update version name */
	availableVersionName: string;
	/** Minecraft version of the available update */
	availableMcVersion: string;
	/** Changelog for the update */
	changelog?: string;
}

/** Result of checking for updates */
export interface UpdateCheckResult {
	/** Instance ID */
	instanceId: string;
	/** Current Minecraft version */
	currentMcVersion: string;
	/** Current mod loader version */
	currentLoaderVersion?: string;
	/** Target Minecraft version (same as current for content updates) */
	targetMcVersion: string;
	/** Target mod loader type */
	targetLoader: LoaderType;
	/** Target mod loader version */
	targetLoaderVersion?: string;
	/** Available loader versions for target MC version */
	availableLoaderVersions: LoaderVersion[];
	/** Content with available updates */
	updatable: ContentUpdateInfo[];
	/** Content that is up to date */
	upToDate: ContentUpdateInfo[];
	/** Content without compatible versions for target */
	incompatible: ContentUpdateInfo[];
	/** Content that couldn't be identified */
	unidentified: ContentUpdateInfo[];
	/** Modpack update info (if this is a modpack instance) */
	modpackUpdate?: ModpackUpdateInfo;
}

/** Action to take for incompatible content */
export type IncompatibleContentAction = 'keep' | 'remove';

/** Plan for executing an update */
export interface UpdatePlan {
	/** Instance ID */
	instanceId: string;
	/** New Minecraft version (if changing) */
	updateMinecraftVersion?: string;
	/** New loader version (if changing) */
	updateLoaderVersion?: string;
	/** Filenames of content to update */
	contentToUpdate: string[];
	/** Filenames of content to remove */
	contentToRemove: string[];
	/** Filenames of incompatible content to keep (may cause issues) */
	contentToKeep: string[];
}

/** Progress of an update operation */
export interface UpdateProgress {
	/** Current stage of the update */
	stage: string;
	/** Progress percentage (0-100) */
	progress: number;
	/** Current item being processed */
	currentItem?: string;
	/** Total number of items */
	totalItems: number;
	/** Number of completed items */
	completedItems: number;
}

// =============================================================================
// NEW UPDATE SYSTEM TYPES
// =============================================================================

/** Available modpack version for update selection */
export interface ModpackVersionOption {
	/** Version ID on the platform */
	versionId: string;
	/** Version name/number */
	versionName: string;
	/** Minecraft version */
	mcVersion: string;
	/** Loader type */
	loaderType: LoaderType;
	/** Loader version (if available) */
	loaderVersion?: string;
	/** Release timestamp */
	releasedAt?: number;
	/** Changelog for this version */
	changelog?: string;
	/** Whether this is the currently installed version */
	isCurrent: boolean;
}

/** Result of checking modpack instance for updates */
export interface ModpackInstanceUpdateCheck {
	/** Instance ID */
	instanceId: string;
	/** Modpack display name */
	modpackName: string;
	/** Platform the modpack is from */
	platform: ModpackPlatform;
	/** Modpack ID on the platform */
	modpackId: string;
	/** Currently installed version */
	currentVersion: ModpackVersionOption;
	/** All available versions (newest first) */
	availableVersions: ModpackVersionOption[];
	/** User-added content that needs decisions */
	userAddedContent: InstalledContent[];
	/** Whether newer versions are available */
	hasUpdate: boolean;
}

/** User decision for content during update (no default allowed) */
export type UserContentDecision = 'pending' | 'keep' | 'remove';

/** Plan for executing a modpack update */
export interface ModpackUpdatePlan {
	/** Instance ID */
	instanceId: string;
	/** Target version ID to update to */
	targetVersionId: string;
	/** Decisions for user-added content (filename -> decision) */
	userContentDecisions: Record<string, UserContentDecision>;
}

/** Result of checking non-modpack instance for updates */
export interface InstanceUpdateCheck {
	/** Instance ID */
	instanceId: string;
	/** Current Minecraft version */
	currentMcVersion: string;
	/** Current loader type */
	currentLoaderType: LoaderType;
	/** Current loader version */
	currentLoaderVersion?: string;
	/** Latest available Minecraft version */
	latestMcVersion: string;
	/** Whether MC update is available */
	hasMcUpdate: boolean;
	/** Target loader version for the update */
	targetLoaderVersion?: string;
	/** Content compatible with target version */
	compatibleContent: ContentUpdateInfo[];
	/** Content incompatible with target version */
	incompatibleContent: ContentUpdateInfo[];
	/** Content that couldn't be identified */
	unidentifiedContent: ContentUpdateInfo[];
}

/** Plan for executing a non-modpack instance update */
export interface InstanceUpdatePlan {
	/** Instance ID */
	instanceId: string;
	/** Target Minecraft version */
	targetMcVersion: string;
	/** Target loader type */
	targetLoaderType: LoaderType;
	/** Target loader version */
	targetLoaderVersion?: string;
	/** Decisions for incompatible content (filename -> decision) */
	incompatibleDecisions: Record<string, UserContentDecision>;
}
