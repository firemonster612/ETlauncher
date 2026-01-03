/** Application theme */
export type Theme = "dark" | "light" | "system";

/** Global application settings */
export interface AppSettings {
  /** Path to instances directory */
  instancesPath: string;
  /** Path to Java executable (auto-detected or user-set) */
  javaPath?: string;
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
}

/** Request to update settings (partial update) */
export interface UpdateSettingsRequest {
  instancesPath?: string;
  javaPath?: string;
  memoryMinMb?: number;
  memoryMaxMb?: number;
  concurrentDownloads?: number;
  closeLauncherOnGameStart?: boolean;
  reopenLauncherOnGameClose?: boolean;
  showSnapshots?: boolean;
  showOldVersions?: boolean;
  theme?: Theme;
}
