import type { LoaderType, ModpackPlatform } from './instance';
import type { ContentGalleryImage } from './content';

/** Sort order for modpack search */
export type ModpackSortBy = 'downloads' | 'recentlyUpdated' | 'name' | 'relevance' | 'newest';

/** Side filter for client/server support */
export type SideFilter = 'client' | 'server' | 'both';

/** Search parameters for modpack queries */
export interface ModpackSearchParams {
	/** Search query (by name) */
	query?: string;
	/** Filter by Minecraft version */
	mcVersion?: string;
	/** Filter by mod loader */
	loader?: LoaderType;
	/** Filter by category */
	category?: string;
	/** Sort order */
	sortBy?: ModpackSortBy;
	/** Page number (0-indexed) */
	page?: number;
	/** Number of results per page */
	pageSize?: number;
	/** Filter by platform (undefined = search all) */
	platform?: ModpackPlatform;
	/** Filter by client/server side support */
	side?: SideFilter;
}

/** A file within a modpack version */
export interface ModpackFile {
	/** Download URL */
	url: string;
	/** File hash (SHA1 or SHA512) */
	hash?: string;
	/** Hash algorithm used */
	hashAlgorithm?: string;
	/** File size in bytes */
	size: number;
	/** Relative path within the instance */
	path: string;
	/** Whether this is a required file */
	required: boolean;
}

/** A specific version of a modpack */
export interface ModpackVersion {
	/** Version ID on the platform */
	id: string;
	/** Version name/number */
	name: string;
	/** Minecraft version this is for */
	mcVersion: string;
	/** Mod loader type */
	loaderType: LoaderType;
	/** Mod loader version */
	loaderVersion?: string;
	/** Changelog for this version */
	changelog?: string;
	/** Release date (Unix timestamp) */
	releasedAt?: number;
	/** Download count for this version */
	downloads?: number;
	/** Files included in this version */
	files: ModpackFile[];
}

/** External links for a modpack (Discord, Wiki, GitHub, etc.) */
export interface ModpackExternalLinks {
	discordUrl?: string;
	wikiUrl?: string;
	issuesUrl?: string;
	sourceUrl?: string;
}

/** A team member of a modpack project */
export interface ModpackTeamMember {
	username: string;
	name?: string;
	avatarUrl?: string;
	role: string;
}

/** A modpack from any platform */
export interface Modpack {
	/** Unique ID on the platform */
	id: string;
	/** Slug/URL-friendly name */
	slug: string;
	/** Display name */
	name: string;
	/** Author/team name */
	author: string;
	/** Short description */
	description: string;
	/** Full description (may be markdown) */
	body?: string;
	/** Icon URL */
	iconUrl?: string;
	/** Banner/cover image URL */
	bannerUrl?: string;
	/** Total download count */
	downloads: number;
	/** Platform this modpack is from */
	platform: ModpackPlatform;
	/** Categories/tags */
	categories: string[];
	/** Gallery images */
	gallery?: ContentGalleryImage[];
	/** Available Minecraft versions */
	mcVersions: string[];
	/** Available mod loaders */
	loaders: LoaderType[];
	/** Latest version info (if available) */
	latestVersion?: ModpackVersion;
	/** External URL to modpack page */
	url?: string;
	/** Last updated timestamp */
	updatedAt?: number;
	/** Created timestamp */
	createdAt?: number;
	/** Client-side support: 'required', 'optional', or 'unsupported' */
	clientSide?: 'required' | 'optional' | 'unsupported';
	/** Server-side support: 'required', 'optional', or 'unsupported' */
	serverSide?: 'required' | 'optional' | 'unsupported';
	/** External links (Discord, Wiki, Issues, Source) */
	externalLinks?: ModpackExternalLinks;
	/** Team members */
	teamMembers?: ModpackTeamMember[];
	/** Follower count */
	followers?: number;
}

/** Content type for items within a modpack */
export type ModpackContentType = 'mod' | 'shader' | 'resourcePack' | 'dataPack' | 'other';

/** A content entry within a modpack (mod, shader, resource pack, etc.) */
export interface ModpackMod {
	/** Platform-specific project ID */
	id: string;
	/** Display name */
	name: string;
	/** Optional icon URL */
	iconUrl?: string;
	/** Optional author */
	author?: string;
	/** Optional external URL */
	url?: string;
	/** Content type (mod, shader, resource pack, etc.) */
	contentType?: ModpackContentType;
}

/** Search results from a platform */
export interface ModpackSearchResult {
	/** List of modpacks */
	modpacks: Modpack[];
	/** Total number of results (for pagination) */
	totalCount: number;
	/** Current page */
	page: number;
	/** Page size */
	pageSize: number;
}

/** Progress of modpack installation */
export interface ModpackInstallProgress {
	/** Current stage of installation */
	stage: string;
	/** Progress percentage (0-100) */
	progress: number;
	/** Current item being processed */
	currentItem?: string;
	/** Total items to process */
	totalItems: number;
	/** Items processed so far */
	completedItems: number;
	/** Total bytes to download (0 if not tracking bytes) */
	totalBytes?: number;
	/** Bytes downloaded so far */
	downloadedBytes?: number;
	/** Download speed in bytes per second */
	speedBytesPerSec?: number;
}

/** Request to install a modpack */
export interface InstallModpackRequest {
	/** Platform the modpack is from */
	platform: ModpackPlatform;
	/** Modpack ID on the platform */
	modpackId: string;
	/** Version ID to install */
	versionId: string;
	/** Custom name for the instance (optional) */
	instanceName?: string;
}
