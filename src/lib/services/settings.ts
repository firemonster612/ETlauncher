import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, UpdateSettingsRequest } from "$lib/types";

/** Get current application settings */
export async function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

/** Update application settings */
export async function updateSettings(
  updates: Partial<AppSettings>
): Promise<AppSettings> {
  // Convert frontend naming to backend naming (camelCase to snake_case)
  const request: UpdateSettingsRequest = {
    instancesPath: updates.instancesPath,
    javaPath: updates.javaPath,
    memoryMinMb: updates.memoryMinMb,
    memoryMaxMb: updates.memoryMaxMb,
    concurrentDownloads: updates.concurrentDownloads,
    closeLauncherOnGameStart: updates.closeLauncherOnGameStart,
    reopenLauncherOnGameClose: updates.reopenLauncherOnGameClose,
    showSnapshots: updates.showSnapshots,
    showOldVersions: updates.showOldVersions,
    theme: updates.theme,
  };

  return invoke<AppSettings>("update_settings", { updates: request });
}

/** Reset settings to defaults */
export async function resetSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("reset_settings");
}

/** Get the default instances path */
export async function getDefaultInstancesPath(): Promise<string> {
  return invoke<string>("get_default_instances_path");
}

/** Detect Java path on the system */
export async function detectJava(): Promise<string | null> {
  return invoke<string | null>("detect_java");
}
