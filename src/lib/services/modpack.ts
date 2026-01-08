import { invoke } from "@tauri-apps/api/core";
import type {
  Instance,
  Modpack,
  ModpackMod,
  ModpackSearchParams,
  ModpackSearchResult,
  ModpackVersion,
  ModpackPlatform,
} from "$lib/types";

/** Search for modpacks across platforms */
export async function searchModpacks(
  params: ModpackSearchParams
): Promise<ModpackSearchResult> {
  return invoke<ModpackSearchResult>("search_modpacks", { params });
}

/** Get a modpack by ID */
export async function getModpack(
  platform: ModpackPlatform,
  id: string
): Promise<Modpack> {
  return invoke<Modpack>("get_modpack", { platform, id });
}

/** Get versions for a modpack */
export async function getModpackVersions(
  platform: ModpackPlatform,
  id: string
): Promise<ModpackVersion[]> {
  return invoke<ModpackVersion[]>("get_modpack_versions", { platform, id });
}

/** Get mod list for a specific modpack version (best-effort) */
export async function getModpackMods(
  platform: ModpackPlatform,
  modpackId: string,
  versionId: string
): Promise<ModpackMod[]> {
  return invoke<ModpackMod[]>("get_modpack_mods", { platform, modpackId, versionId });
}

/** Install a modpack and create a new instance */
export async function installModpack(
  platform: ModpackPlatform,
  modpackId: string,
  versionId: string,
  instanceName?: string
): Promise<Instance> {
  return invoke<Instance>("install_modpack", {
    platform,
    modpackId,
    versionId,
    instanceName,
  });
}

/** Import an instance from a local .mrpack file */
export async function importModpackFile(
  filePath: string,
  instanceName?: string
): Promise<Instance> {
  return invoke<Instance>("import_modpack_file", {
    filePath,
    instanceName,
  });
}
