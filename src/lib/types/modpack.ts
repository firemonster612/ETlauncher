import type { LoaderType, ModpackPlatform } from "./instance";

/** Sort order for modpack search */
export type ModpackSortBy = "downloads" | "recentlyUpdated" | "name" | "relevance";

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
