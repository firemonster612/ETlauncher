/** Application theme */
export type Theme = "dark" | "light" | "system";

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
}
