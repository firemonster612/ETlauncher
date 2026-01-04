import type { LoaderType } from "./instance";

/** Information about a mod loader version */
export interface LoaderVersion {
  /** Version string (e.g., "0.18.4" for Fabric, "61.0.5" for Forge) */
  version: string;
  /** Maven coordinates for downloading */
  maven: string;
  /** Whether this version is marked as stable */
  stable: boolean;
  /** Build number (for some loaders) */
  build?: number;
  /** Separator character used in version strings */
  separator?: string;
}

/** Request to install a mod loader */
export interface LoaderInstallRequest {
  /** Type of loader to install */
  loaderType: LoaderType;
  /** Minecraft version */
  minecraftVersion: string;
  /** Loader version to install */
  loaderVersion: string;
}

/** Progress of a loader installation */
export interface LoaderInstallProgress {
  /** Current stage of installation */
  stage: string;
  /** Progress percentage (0-100) */
  progress: number;
  /** Current file being downloaded/processed */
  currentFile?: string;
  /** Total bytes to download (if applicable) */
  totalBytes?: number;
  /** Bytes downloaded so far (if applicable) */
  downloadedBytes?: number;
}

/** Installation stages */
export type LoaderInstallStage =
  | "downloading-installer"
  | "running-installer"
  | "verifying-installation"
  | "complete"
  | "failed";
