import type { LoaderType } from './instance';

/** Platform that hosts individual content (mods, shaders, resourcepacks) */
export type ContentPlatform = 'modrinth' | 'curseforge';

/** Type of content */
export type ContentType = 'mod' | 'shader' | 'resourcepack' | 'datapack' | 'world';

/** Source of content (how it was added to the instance) */
export type ContentSource = 'modpackOriginal' | 'userAdded' | 'userDependency';

/** Sort order for content search */
export type ContentSortBy = 'downloads' | 'recentlyUpdated' | 'name' | 'relevance';

/** Dependency type */
export type DependencyType = 'required' | 'optional' | 'incompatible' | 'embedded';

/** A dependency of a content version */
export interface ContentDependency {
	/** ID of the dependency on the same platform */
	id: string;
	/** Name of the dependency (for display) */
	name?: string;
	/** Type of dependency */
	dependencyType: DependencyType;
	/** Version requirement (if any) */
	versionReq?: string;
}

/** Search parameters for content queries */
export interface ContentSearchParams {
	/** Search query (by name) */
	query?: string;
	/** Filter by Minecraft version */
	mcVersion?: string;
	/** Filter by mod loader */
	loader?: LoaderType;
	/** Filter by content type */
	contentType?: ContentType;
	/** Filter by category */
	category?: string;
	/** Sort order */
	sortBy?: ContentSortBy;
	/** Page number (0-indexed) */
	page?: number;
	/** Number of results per page */
	pageSize?: number;
	/** Filter by platform (undefined = search all) */
	platform?: ContentPlatform;
}

/** A file within a content version */
export interface ContentFile {
	/** Download URL */
	url: string;
	/** File hash (SHA1 or SHA512) */
	hash?: string;
	/** Hash algorithm used */
	hashAlgorithm?: string;
	/** File size in bytes */
	size: number;
	/** Filename */
	filename: string;
	/** Whether this is the primary file */
	primary: boolean;
}

/** A specific version of content */
export interface ContentVersion {
	/** Version ID on the platform */
	id: string;
	/** Project/content ID this version belongs to */
	projectId: string;
	/** Version name/number */
	name: string;
	/** Version number (semantic if available) */
	versionNumber: string;
	/** Minecraft versions this is for */
	mcVersions: string[];
	/** Mod loaders this is for */
	loaders: LoaderType[];
	/** Release date (Unix timestamp) */
	releasedAt?: number;
	/** Download count for this version */
	downloads?: number;
	/** Files included in this version */
	files: ContentFile[];
	/** Dependencies */
	dependencies: ContentDependency[];
	/** Changelog */
	changelog?: string;
}

/** Gallery image for content */
export interface ContentGalleryImage {
	/** Processed image URL */
	url: string;
	/** Original image URL if available */
	rawUrl?: string;
	/** Optional title */
	title?: string;
	/** Optional description */
	description?: string;
	/** Whether the platform marks this as featured */
	featured?: boolean;
}

/** Content item from any platform (mod, shader, resourcepack) */
export interface Content {
	/** Unique ID on the platform */
	id: string;
	/** Slug/URL-friendly name */
	slug: string;
	/** Display name */
	name: string;
	/** Author name */
	author: string;
	/** Short description */
	description: string;
	/** Full description (may be markdown) */
	body?: string;
	/** Icon URL */
	iconUrl?: string;
	/** Total download count */
	downloads: number;
	/** Platform this content is from */
	platform: ContentPlatform;
	/** Type of content */
	contentType: ContentType;
	/** Categories/tags */
	categories: string[];
	/** Gallery images */
	gallery?: ContentGalleryImage[];
	/** Available Minecraft versions */
	mcVersions: string[];
	/** Available mod loaders */
	loaders: LoaderType[];
	/** Latest version info (if available) */
	latestVersion?: ContentVersion;
	/** External URL to content page */
	url?: string;
	/** Last updated timestamp */
	updatedAt?: number;
	/** Created timestamp */
	createdAt?: number;
}

/** Search results from a platform */
export interface ContentSearchResult {
	/** List of content items */
	items: Content[];
	/** Total number of results (for pagination) */
	totalCount: number;
	/** Current page */
	page: number;
	/** Page size */
	pageSize: number;
}

/** Installed content tracking for an instance */
export interface InstalledContent {
	/** Display name */
	name: string;
	/** Slug for matching across platforms */
	slug: string;
	/** Modrinth project ID (if known) */
	modrinthId?: string;
	/** CurseForge project ID (if known) */
	curseforgeId?: number;
	/** Platform the content was installed from */
	installedFrom: ContentPlatform;
	/** Installed version string */
	version: string;
	/** Installed version ID */
	versionId: string;
	/** Filename of the installed file */
	filename: string;
	/** Content type */
	contentType: ContentType;
	/** Install timestamp */
	installedAt: number;
	/** Whether this was installed as a dependency */
	isDependency: boolean;
	/** Filenames of content this is a dependency of (parent mods) */
	dependencyOf: string[];
	/** IDs of content this mod depends on (for reverse lookup) */
	dependencyIds: string[];
	/** Source of the content (modpack original, user added, etc.) */
	source: ContentSource;
	/** SHA512 hash for Modrinth identification */
	sha512Hash?: string;
	/** Murmur2 fingerprint for CurseForge identification */
	murmur2Fingerprint?: number;
}

/** Manifest of all installed content in an instance */
export interface InstalledContentManifest {
	/** Manifest version for migration support */
	manifestVersion: number;
	/** Installed mods */
	mods: InstalledContent[];
	/** Installed shaders */
	shaders: InstalledContent[];
	/** Installed resource packs */
	resourcePacks: InstalledContent[];
	/** Installed datapacks */
	datapacks: InstalledContent[];
	/** Installed worlds */
	worlds: InstalledContent[];
}

/** Request to install content to an instance */
export interface InstallContentRequest {
	/** Instance to install to */
	instanceId: string;
	/** Platform the content is from */
	platform: ContentPlatform;
	/** Content ID on the platform */
	contentId: string;
	/** Version ID to install */
	versionId: string;
	/** Whether to also install dependencies */
	installDependencies: boolean;
}

/** Progress of content installation */
export interface ContentInstallProgress {
	/** Current stage of installation */
	stage: string;
	/** Progress percentage (0-100) */
	progress: number;
	/** Current file being downloaded */
	currentFile?: string;
	/** Total files to download */
	totalFiles: number;
	/** Files downloaded so far */
	completedFiles: number;
}

/** Progress of a single file download (for real-time UI updates) */
export interface ContentDownloadProgress {
	/** Filename being downloaded */
	filename: string;
	/** Bytes downloaded so far */
	downloadedBytes: number;
	/** Total file size in bytes */
	totalBytes: number;
	/** Progress percentage (0-100) */
	progressPercent: number;
}

/** Progress with queue identification for parallel downloads */
export interface ContentDownloadProgressWithId extends ContentDownloadProgress {
	/** Queue entry ID */
	queueId: string;
	/** Content ID for matching */
	contentId: string;
}

/** Status of a queued content download */
export type QueueItemStatus = 'pending' | 'downloading' | 'completed' | 'failed';

/** A single item in the download queue */
export interface QueuedDownload {
	/** Unique queue entry ID */
	queueId: string;
	/** Content being downloaded */
	content: Content;
	/** Version to install */
	version: ContentVersion;
	/** Instance ID installing to */
	instanceId: string;
	/** Current status */
	status: QueueItemStatus;
	/** Download progress (when downloading) */
	progress?: ContentDownloadProgress;
	/** Error message (when failed) */
	error?: string;
	/** Timestamp when queued */
	queuedAt: number;
}

/** Request to queue a content installation */
export interface QueueInstallRequest {
	queueId: string;
	instanceId: string;
	platform: ContentPlatform;
	contentId: string;
	contentName: string;
	contentSlug: string;
	contentType: ContentType;
	versionId: string;
	versionName: string;
	mcVersion: string;
	loader?: LoaderType;
	/** Whether this is a dependency (auto-resolved) vs user-requested */
	isDependency?: boolean;
}

/** Resolved dependency with install info */
export interface ResolvedDependency {
	/** The content item */
	content: Content;
	/** The version to install */
	version: ContentVersion;
	/** Whether this is already installed */
	alreadyInstalled: boolean;
}

/** Modrinth project info from hash lookup */
export interface DetectedModrinthProject {
	/** Project ID */
	projectId: string;
	/** Project slug */
	slug: string;
	/** Display name */
	name: string;
	/** Version ID that was matched */
	versionId: string;
	/** Version number string */
	versionNumber: string;
}

/** CurseForge project info from fingerprint lookup */
export interface DetectedCurseForgeProject {
	/** Project ID */
	projectId: number;
	/** File ID */
	fileId: number;
	/** Display name */
	name: string;
	/** Filename */
	filename: string;
	/** Project slug for cross-platform matching */
	slug: string;
}

/** A detected mod file from scanning the mods folder */
export interface DetectedMod {
	/** Filename of the mod */
	filename: string;
	/** File size in bytes */
	size: number;
	/** SHA512 hash of the file */
	sha512: string;
	/** Murmur2 fingerprint for CurseForge */
	murmur2Fingerprint: number;
	/** Modrinth project info (if identified) */
	modrinthProject?: DetectedModrinthProject;
	/** CurseForge project info (if identified) */
	curseforgeProject?: DetectedCurseForgeProject;
	/** Whether this mod was identified */
	isIdentified: boolean;
	/** Whether this item is disabled (in disabled subfolder) */
	isDisabled: boolean;
	/** Whether this was installed as a dependency */
	isDependency: boolean;
	/** Filenames of content this is a dependency of (parent mods) */
	dependencyOf: string[];
}

/** Result of scanning an instance's content folder */
export interface ScanResult {
	/** Content folder was found */
	folderExists: boolean;
	/** All detected content items */
	items: DetectedMod[];
	/** Count of identified items (matched via Modrinth or CurseForge) */
	identifiedCount: number;
	/** Count of unidentified items */
	unidentifiedCount: number;
	/** Timestamp of scan */
	scannedAt: number;
}
